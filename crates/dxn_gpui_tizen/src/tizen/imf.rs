//! The window's Ecore_IMF context (design `d16-ime`): IME events go to GPUI's input handler
//! through `crate::ime`, `ImfCall`s are performed on the context, and key events are offered to
//! the IME first while a text field is focused.

use std::cell::RefCell;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr;

use dxn_gpui_tizen_sys as sys;
use gpui::{Bounds, Pixels, TextInputStateChange};

use crate::ime::{self, ImeEvent, ImeFocus, ImfCall};

/// Text the IME may see after the cursor (`retrieve_surrounding`), in UTF-16 units.
const SURROUNDING_AFTER: usize = 256;

pub(crate) struct Imf {
    ctx: *mut sys::Ecore_IMF_Context,
    focus: RefCell<ImeFocus>,
}

type EventCb = unsafe extern "C" fn(*mut c_void, *mut sys::Ecore_IMF_Context, *mut c_void);

const CALLBACKS: [(sys::Ecore_IMF_Callback_Type, EventCb); 5] = [
    (sys::ECORE_IMF_CALLBACK_PREEDIT_START, on_preedit_start),
    (sys::ECORE_IMF_CALLBACK_PREEDIT_END, on_preedit_end),
    (sys::ECORE_IMF_CALLBACK_PREEDIT_CHANGED, on_preedit_changed),
    (sys::ECORE_IMF_CALLBACK_COMMIT, on_commit),
    (
        sys::ECORE_IMF_CALLBACK_DELETE_SURROUNDING,
        on_delete_surrounding,
    ),
];

impl Imf {
    /// A context for `window`, or `None` when the TV has no input method module.
    pub(crate) fn new(window: *mut sys::Ecore_Wl2_Window) -> Option<Self> {
        // SAFETY: Ecore is initialized inside ui_app_main; init and shutdown are counted.
        let ctx = unsafe {
            sys::ecore_imf_init();
            let id = sys::ecore_imf_context_default_id_get();
            if id.is_null() {
                ptr::null_mut()
            } else {
                sys::ecore_imf_context_add(id)
            }
        };
        if ctx.is_null() {
            log::warn!("no Ecore_IMF context: text fields get no on-screen keyboard");
            // SAFETY: balances the init above.
            unsafe { sys::ecore_imf_shutdown() };
            return None;
        }
        // SAFETY: a live context and window; the callbacks reach the window through `shared()`
        // and are removed before the context is deleted. Ecore_IMF's Wayland module takes the
        // Ecore_Wl2 window id as the client window.
        unsafe {
            let id = sys::ecore_wl2_window_id_get(window) as usize;
            sys::ecore_imf_context_client_window_set(ctx, id as *mut c_void);
            for (kind, cb) in CALLBACKS {
                sys::ecore_imf_context_event_callback_add(ctx, kind, Some(cb), ptr::null());
            }
            sys::ecore_imf_context_retrieve_surrounding_callback_set(
                ctx,
                Some(on_retrieve_surrounding),
                ptr::null(),
            );
        }
        Some(Self {
            ctx,
            focus: RefCell::new(ImeFocus::default()),
        })
    }

    /// GPUI's `text_input_state_changed`: the calls to perform once GPUI's frame is done.
    pub(crate) fn state_changed(&self, change: TextInputStateChange) -> Vec<ImfCall> {
        self.focus.borrow_mut().state_changed(change)
    }

    /// GPUI installed an input handler for `field`: `true` when [`Imf::settle`] is to be posted.
    pub(crate) fn input_handler(&self, field: Option<gpui::FocusId>) -> bool {
        self.focus.borrow_mut().input_handler(field)
    }

    /// Whether IME events reach the field now (see `ImfCall::BeginDiscard`).
    pub(crate) fn accepts_events(&self) -> bool {
        self.focus.borrow().accepts_events()
    }

    /// After a frame that switched text fields.
    pub(crate) fn settle(&self) -> Vec<ImfCall> {
        self.focus.borrow_mut().settle()
    }

    pub(crate) fn show_keyboard(&self) -> Vec<ImfCall> {
        self.focus.borrow().show_keyboard()
    }

    pub(crate) fn hide_keyboard(&self) -> Vec<ImfCall> {
        self.focus.borrow().hide_keyboard()
    }

    pub(crate) fn window_focus(&self, active: bool) -> Vec<ImfCall> {
        self.focus.borrow().window_focus(active)
    }

    /// Performs `calls`; `cursor` gives the cursor in characters for `CursorPosition`.
    pub(crate) fn perform(&self, calls: &[ImfCall], cursor: impl Fn() -> Option<usize>) {
        for call in calls {
            // SAFETY: a live context, on the main thread.
            unsafe {
                match call {
                    ImfCall::BeginDiscard | ImfCall::EndDiscard => {
                        self.focus.borrow_mut().marker(*call)
                    }
                    ImfCall::FocusIn => sys::ecore_imf_context_focus_in(self.ctx),
                    ImfCall::FocusOut => sys::ecore_imf_context_focus_out(self.ctx),
                    ImfCall::Reset => sys::ecore_imf_context_reset(self.ctx),
                    ImfCall::ShowPanel => sys::ecore_imf_context_input_panel_show(self.ctx),
                    ImfCall::HidePanel => sys::ecore_imf_context_input_panel_hide(self.ctx),
                    ImfCall::CursorPosition => {
                        if let Some(chars) = cursor() {
                            let chars = c_int::try_from(chars).unwrap_or(c_int::MAX);
                            sys::ecore_imf_context_cursor_position_set(self.ctx, chars);
                        }
                    }
                }
            }
        }
    }

    /// The caret's bounds (GPUI's `update_ime_position`), for the IME's candidate window.
    pub(crate) fn cursor_location(&self, bounds: Bounds<Pixels>, scale: f32) {
        let (x, y, w, h) = ime::cursor_location(bounds, scale);
        // SAFETY: a live context.
        unsafe { sys::ecore_imf_context_cursor_location_set(self.ctx, x, y, w, h) };
    }

    /// Offers a key event to the IME while a field is focused; `true` when the IME consumed it.
    pub(crate) fn filter_key(&self, event: &sys::Ecore_Event_Key, down: bool) -> bool {
        if !self.focus.borrow().editable() {
            return false;
        }
        let key = sys::Ecore_IMF_Event_Key {
            keyname: event.keyname,
            modifiers: ime::imf_modifiers(event.modifiers) as c_int,
            locks: ime::imf_locks(event.modifiers) as c_int,
            key: event.key,
            string: event.string,
            compose: event.compose,
            timestamp: event.timestamp,
            dev_name: ptr::null(),
            dev_class: 0,
            dev_subclass: 0,
            keycode: event.keycode,
        };
        let (kind, mut imf_event) = if down {
            (
                sys::ECORE_IMF_EVENT_KEY_DOWN,
                sys::Ecore_IMF_Event { key_down: key },
            )
        } else {
            (
                sys::ECORE_IMF_EVENT_KEY_UP,
                sys::Ecore_IMF_Event { key_up: key },
            )
        };
        // SAFETY: a live context; the event and the strings it borrows from Ecore's key event
        // outlive the call.
        unsafe { sys::ecore_imf_context_filter_event(self.ctx, kind, &mut imf_event) != 0 }
    }
}

impl Drop for Imf {
    fn drop(&mut self) {
        // SAFETY: the context from `new`, its callbacks as added; then the balancing shutdown.
        unsafe {
            for (kind, cb) in CALLBACKS {
                sys::ecore_imf_context_event_callback_del(self.ctx, kind, Some(cb));
            }
            sys::ecore_imf_context_del(self.ctx);
            sys::ecore_imf_shutdown();
        }
    }
}

/// Applies an IME event to the window's input handler, unless the context is leaving a field.
fn deliver(event: ImeEvent) {
    let window = super::app::shared().and_then(|shared| shared.window.borrow().clone());
    if let Some(window) = window
        && window.imf_accepts_events()
    {
        window.with_input_handler(|handler| ime::apply(&event, handler));
    }
}

unsafe extern "C" fn on_preedit_start(
    _data: *mut c_void,
    _ctx: *mut sys::Ecore_IMF_Context,
    _info: *mut c_void,
) {
    deliver(ImeEvent::PreeditStart);
}

unsafe extern "C" fn on_preedit_end(
    _data: *mut c_void,
    _ctx: *mut sys::Ecore_IMF_Context,
    _info: *mut c_void,
) {
    deliver(ImeEvent::PreeditEnd);
}

unsafe extern "C" fn on_preedit_changed(
    _data: *mut c_void,
    ctx: *mut sys::Ecore_IMF_Context,
    _info: *mut c_void,
) {
    let mut text: *mut c_char = ptr::null_mut();
    let mut cursor: c_int = 0;
    // SAFETY: the context of this callback; the string is malloc'd and freed here.
    let text = unsafe {
        sys::ecore_imf_context_preedit_string_get(ctx, &mut text, &mut cursor);
        if text.is_null() {
            String::new()
        } else {
            let owned = CStr::from_ptr(text).to_string_lossy().into_owned();
            libc::free(text.cast());
            owned
        }
    };
    deliver(ImeEvent::PreeditChanged {
        text,
        cursor: usize::try_from(cursor).unwrap_or(0),
    });
}

unsafe extern "C" fn on_commit(
    _data: *mut c_void,
    _ctx: *mut sys::Ecore_IMF_Context,
    info: *mut c_void,
) {
    if info.is_null() {
        return;
    }
    // SAFETY: COMMIT's event info is the committed NUL-terminated UTF-8 string.
    let text = unsafe { CStr::from_ptr(info.cast::<c_char>()) }
        .to_string_lossy()
        .into_owned();
    deliver(ImeEvent::Commit(text));
}

unsafe extern "C" fn on_delete_surrounding(
    _data: *mut c_void,
    _ctx: *mut sys::Ecore_IMF_Context,
    info: *mut c_void,
) {
    if info.is_null() {
        return;
    }
    // SAFETY: DELETE_SURROUNDING's event info is an `Ecore_IMF_Event_Delete_Surrounding`.
    let event = unsafe { &*info.cast::<sys::Ecore_IMF_Event_Delete_Surrounding>() };
    deliver(ImeEvent::DeleteSurrounding {
        offset: event.offset,
        n_chars: event.n_chars,
    });
}

unsafe extern "C" fn on_retrieve_surrounding(
    _data: *mut c_void,
    _ctx: *mut sys::Ecore_IMF_Context,
    text: *mut *mut c_char,
    cursor_pos: *mut c_int,
) -> sys::Eina_Bool {
    let window = super::app::shared()
        .and_then(|shared| shared.window.borrow().clone())
        .filter(|window| window.imf_accepts_events());
    let surrounding = window
        .and_then(|window| window.with_input_handler(|h| ime::surrounding(h, SURROUNDING_AFTER)))
        .flatten();
    let Some((surrounding, cursor)) = surrounding else {
        return sys::EINA_FALSE;
    };
    if !text.is_null() {
        let bytes = surrounding.replace('\0', " ");
        // SAFETY: Ecore_IMF frees the string with free(); it is NUL-terminated here.
        unsafe {
            let copy = libc::malloc(bytes.len() + 1).cast::<u8>();
            if copy.is_null() {
                return sys::EINA_FALSE;
            }
            ptr::copy_nonoverlapping(bytes.as_ptr(), copy, bytes.len());
            *copy.add(bytes.len()) = 0;
            *text = copy.cast();
        }
    }
    if !cursor_pos.is_null() {
        // SAFETY: an out pointer from Ecore_IMF.
        unsafe { *cursor_pos = c_int::try_from(cursor).unwrap_or(c_int::MAX) };
    }
    sys::EINA_TRUE
}
