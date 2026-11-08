//! Thin, hand-written `extern "C"` bindings to the documented Tizen 8.0 native C APIs that
//! `dxn_gpui_tizen` and the diagnostic app use, and nothing else.
//!
//! Every function and global goes through [`tizen_extern!`], which also records it in
//! [`DECLARED_SYMBOLS`]; `symbols.allow` lists each allowed symbol with the public API module it is
//! documented in, and the `symbols::tests` group keeps the two in step. Link directives apply only
//! when `target_vendor = "tizen"`, so the crate builds and its tests run on any host.
//!
//! Layouts follow the Tizen 8.0 common rootstrap's headers; each item names its header. EGL and
//! GL ES are never declared here: wgpu and the diagnostic app load them at run time.

#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals)]

use core::ffi::{c_char, c_double, c_int, c_uint, c_void};

pub mod symbols;

/// Declares Tizen functions and globals per shared library and records every name, with its
/// library, in [`DECLARED_SYMBOLS`]. A trailing `; ...` after the arguments declares a variadic
/// function.
macro_rules! tizen_extern {
    ($(
        $lib:literal {
            $( $(#[$fattr:meta])* fn $f:ident ( $($a:ident : $t:ty),* $(; $dots:tt)? ) $(-> $r:ty)? ; )*
            $( $(#[$sattr:meta])* static $s:ident : $st:ty ; )*
        }
    )*) => {
        $(
            #[cfg_attr(target_vendor = "tizen", link(name = $lib))]
            unsafe extern "C" {
                $( $(#[$fattr])* pub fn $f( $($a : $t),* $(, $dots)? ) $(-> $r)?; )*
                $( $(#[$sattr])* pub static $s: $st; )*
            }
        )*

        /// Every declared Tizen symbol with the shared library (`lib<name>.so`) it is linked from.
        pub const DECLARED_SYMBOLS: &[(&str, &str)] = &[
            $( $( (stringify!($f), $lib), )* $( (stringify!($s), $lib), )* )*
        ];
    };
}

// ---------------------------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------------------------

/// `Eina_Bool` (`eina-1/eina/eina_types.h`): `unsigned char`.
pub type Eina_Bool = u8;
pub const EINA_TRUE: Eina_Bool = 1;
pub const EINA_FALSE: Eina_Bool = 0;
/// `ECORE_CALLBACK_RENEW` / `ECORE_CALLBACK_CANCEL` (`ecore-1/Ecore_Common.h`).
pub const ECORE_CALLBACK_RENEW: Eina_Bool = EINA_TRUE;
pub const ECORE_CALLBACK_CANCEL: Eina_Bool = EINA_FALSE;
/// `ECORE_CALLBACK_PASS_ON` / `ECORE_CALLBACK_DONE` for event handlers (`ecore-1/Ecore_Common.h`).
pub const ECORE_CALLBACK_PASS_ON: Eina_Bool = EINA_TRUE;
pub const ECORE_CALLBACK_DONE: Eina_Bool = EINA_FALSE;

macro_rules! opaque {
    ($($(#[$m:meta])* $name:ident),* $(,)?) => {$(
        $(#[$m])*
        #[repr(C)]
        pub struct $name {
            _private: [u8; 0],
            _not_send_sync: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    )*};
}

opaque!(
    /// `app_control_s` (`appfw/app_control.h`); used through [`app_control_h`].
    app_control_s,
    /// `Ecore_Timer` (`ecore-1/Ecore_Common.h`).
    Ecore_Timer,
    /// `Ecore_Event_Handler` (`ecore-1/Ecore_Common.h`).
    Ecore_Event_Handler,
    /// `Ecore_Wl2_Display` (`ecore-wl2-1/Ecore_Wayland2.h`).
    Ecore_Wl2_Display,
    /// `Ecore_Wl2_Window` (`ecore-wl2-1/Ecore_Wayland2.h`).
    Ecore_Wl2_Window,
    /// `Ecore_Wl2_Frame_Cb_Handle` (`ecore-wl2-1/Ecore_Wayland2.h`).
    Ecore_Wl2_Frame_Cb_Handle,
    /// `Ecore_IMF_Context` (`ecore-imf-1/Ecore_IMF.h`).
    Ecore_IMF_Context,
    /// `Eo` (the input device of a key event; never dereferenced).
    Eo,
);

/// `app_control_h` (`appfw/app_control.h`).
pub type app_control_h = *mut app_control_s;

/// `app_create_cb` and friends (`appfw/app.h`).
pub type app_create_cb = Option<unsafe extern "C" fn(user_data: *mut c_void) -> bool>;
pub type app_terminate_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type app_pause_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type app_resume_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type app_control_cb =
    Option<unsafe extern "C" fn(app_control: app_control_h, user_data: *mut c_void)>;

/// `ui_app_lifecycle_callback_s` (`appfw/app.h`), in header order.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ui_app_lifecycle_callback_s {
    pub create: app_create_cb,
    pub terminate: app_terminate_cb,
    pub pause: app_pause_cb,
    pub resume: app_resume_cb,
    pub app_control: app_control_cb,
}

/// `APP_ERROR_NONE` (`appfw/app_types.h`, `TIZEN_ERROR_NONE`).
pub const APP_ERROR_NONE: c_int = 0;

/// `Ecore_Cb`, `Ecore_Task_Cb`, `Ecore_Event_Handler_Cb` (`ecore-1/Ecore_Common.h`).
pub type Ecore_Cb = Option<unsafe extern "C" fn(data: *mut c_void)>;
pub type Ecore_Task_Cb = Option<unsafe extern "C" fn(data: *mut c_void) -> Eina_Bool>;
pub type Ecore_Event_Handler_Cb =
    Option<unsafe extern "C" fn(data: *mut c_void, kind: c_int, event: *mut c_void) -> Eina_Bool>;

/// `Ecore_Window` (`ecore-input-1/Ecore_Input.h`): `uintptr_t`.
pub type Ecore_Window = usize;

/// `struct _Ecore_Event_Key` (`ecore-input-1/Ecore_Input.h`), in header order.
#[repr(C)]
pub struct Ecore_Event_Key {
    pub keyname: *const c_char,
    pub key: *const c_char,
    pub string: *const c_char,
    pub compose: *const c_char,
    pub window: Ecore_Window,
    pub root_window: Ecore_Window,
    pub event_window: Ecore_Window,
    pub timestamp: c_uint,
    pub modifiers: c_uint,
    pub same_screen: c_int,
    pub keycode: c_uint,
    pub data: *mut c_void,
    pub dev: *mut Eo,
    /// `Ecore_Event_Flags` (an enum).
    pub event_flags: c_int,
}

/// `ECORE_EVENT_MODIFIER_*` (`ecore-input-1/Ecore_Input.h`).
pub const ECORE_EVENT_MODIFIER_SHIFT: c_uint = 0x0001;
pub const ECORE_EVENT_MODIFIER_CTRL: c_uint = 0x0002;
pub const ECORE_EVENT_MODIFIER_ALT: c_uint = 0x0004;
pub const ECORE_EVENT_MODIFIER_WIN: c_uint = 0x0008;

/// `Ecore_Wl2_Frame_Cb` (`ecore-wl2-1/Ecore_Wayland2.h`).
pub type Ecore_Wl2_Frame_Cb =
    Option<unsafe extern "C" fn(win: *mut Ecore_Wl2_Window, timestamp: u32, data: *mut c_void)>;

/// `Ecore_Wl2_Window_Keygrab_Mode` (`ecore-wl2-1/Ecore_Wayland2.h`); third-party apps may use only
/// `TOPMOST`.
pub type Ecore_Wl2_Window_Keygrab_Mode = c_int;
pub const ECORE_WL2_WINDOW_KEYGRAB_TOPMOST: Ecore_Wl2_Window_Keygrab_Mode = 2;

/// `Ecore_IMF_Event_Cb` (`ecore-imf-1/Ecore_IMF.h`).
pub type Ecore_IMF_Event_Cb = Option<
    unsafe extern "C" fn(data: *mut c_void, ctx: *mut Ecore_IMF_Context, event_info: *mut c_void),
>;

/// `Ecore_IMF_Callback_Type` (`ecore-imf-1/Ecore_IMF.h`), in header order from 0.
pub type Ecore_IMF_Callback_Type = c_int;
pub const ECORE_IMF_CALLBACK_PREEDIT_START: Ecore_IMF_Callback_Type = 0;
pub const ECORE_IMF_CALLBACK_PREEDIT_END: Ecore_IMF_Callback_Type = 1;
pub const ECORE_IMF_CALLBACK_PREEDIT_CHANGED: Ecore_IMF_Callback_Type = 2;
pub const ECORE_IMF_CALLBACK_COMMIT: Ecore_IMF_Callback_Type = 3;
pub const ECORE_IMF_CALLBACK_DELETE_SURROUNDING: Ecore_IMF_Callback_Type = 4;

/// `struct _Ecore_IMF_Event_Delete_Surrounding` (`ecore-imf-1/Ecore_IMF.h`).
#[repr(C)]
pub struct Ecore_IMF_Event_Delete_Surrounding {
    pub ctx: *mut Ecore_IMF_Context,
    pub offset: c_int,
    pub n_chars: c_int,
}

/// `system_settings_key_e` (`system/system_settings.h`); values computed from the header's enum.
pub type system_settings_key_e = c_int;
pub const SYSTEM_SETTINGS_KEY_FONT_SIZE: system_settings_key_e = 3;
pub const SYSTEM_SETTINGS_KEY_FONT_TYPE: system_settings_key_e = 4;
pub const SYSTEM_SETTINGS_KEY_LOCALE_COUNTRY: system_settings_key_e = 12;
pub const SYSTEM_SETTINGS_KEY_LOCALE_LANGUAGE: system_settings_key_e = 13;
pub const SYSTEM_SETTINGS_KEY_LOCALE_TIMEFORMAT_24HOUR: system_settings_key_e = 14;
pub const SYSTEM_SETTINGS_KEY_LOCALE_TIMEZONE: system_settings_key_e = 15;
pub const SYSTEM_SETTINGS_KEY_ACCESSIBILITY_TTS: system_settings_key_e = 32;
/// `SYSTEM_SETTINGS_ERROR_NONE` (`TIZEN_ERROR_NONE`).
pub const SYSTEM_SETTINGS_ERROR_NONE: c_int = 0;

/// `system_settings_changed_cb` (`system/system_settings.h`).
pub type system_settings_changed_cb =
    Option<unsafe extern "C" fn(key: system_settings_key_e, user_data: *mut c_void)>;

/// `log_priority` (`dlog/dlog.h`).
pub type log_priority = c_int;
pub const DLOG_DEBUG: log_priority = 3;
pub const DLOG_INFO: log_priority = 4;
pub const DLOG_WARN: log_priority = 5;
pub const DLOG_ERROR: log_priority = 6;

// ---------------------------------------------------------------------------------------------
// Functions and globals
// ---------------------------------------------------------------------------------------------

tizen_extern! {
    // appfw/app.h (Application Framework > Application)
    "capi-appfw-application" {
        fn ui_app_main(
            argc: c_int,
            argv: *mut *mut c_char,
            callback: *mut ui_app_lifecycle_callback_s,
            user_data: *mut c_void
        ) -> c_int;
        fn ui_app_exit();
    }
    // appfw/app_common.h (Application Framework > Application > Common); the returned strings are
    // malloc'd and freed with libc's `free`.
    "capi-appfw-app-common" {
        fn app_get_cache_path() -> *mut c_char;
        fn app_get_resource_path() -> *mut c_char;
        fn app_get_data_path() -> *mut c_char;
    }
    // appfw/app_control.h (Application Framework > Application > App Control)
    "capi-appfw-app-control" {
        fn app_control_get_extra_data(
            app_control: app_control_h,
            key: *const c_char,
            value: *mut *mut c_char
        ) -> c_int;
    }
    // ecore-1/Ecore_Common.h, Ecore_Legacy.h (EFL > Ecore)
    "ecore" {
        fn ecore_main_loop_thread_safe_call_async(callback: Ecore_Cb, data: *mut c_void);
        fn ecore_timer_add(seconds: c_double, func: Ecore_Task_Cb, data: *const c_void) -> *mut Ecore_Timer;
        fn ecore_timer_del(timer: *mut Ecore_Timer) -> *mut c_void;
        fn ecore_event_handler_add(
            kind: c_int,
            func: Ecore_Event_Handler_Cb,
            data: *const c_void
        ) -> *mut Ecore_Event_Handler;
        fn ecore_event_handler_del(handler: *mut Ecore_Event_Handler) -> *mut c_void;
    }
    // ecore-input-1/Ecore_Input.h (EFL > Ecore > Ecore Input)
    "ecore_input" {
        static ECORE_EVENT_KEY_DOWN: c_int;
        static ECORE_EVENT_KEY_UP: c_int;
    }
    // ecore-wl2-1/Ecore_Wayland2.h (EFL > Ecore > Ecore_Wl2)
    "ecore_wl2" {
        fn ecore_wl2_init() -> c_int;
        fn ecore_wl2_shutdown() -> c_int;
        fn ecore_wl2_connected_display_get(name: *const c_char) -> *mut Ecore_Wl2_Display;
        fn ecore_wl2_display_connect(name: *const c_char) -> *mut Ecore_Wl2_Display;
        fn ecore_wl2_display_native_get(display: *mut Ecore_Wl2_Display) -> *mut c_void;
        fn ecore_wl2_display_screen_size_get(display: *mut Ecore_Wl2_Display, w: *mut c_int, h: *mut c_int);
        fn ecore_wl2_window_new(
            display: *mut Ecore_Wl2_Display,
            parent: *mut Ecore_Wl2_Window,
            x: c_int,
            y: c_int,
            w: c_int,
            h: c_int
        ) -> *mut Ecore_Wl2_Window;
        fn ecore_wl2_window_free(window: *mut Ecore_Wl2_Window);
        fn ecore_wl2_window_id_get(window: *mut Ecore_Wl2_Window) -> c_int;
        fn ecore_wl2_window_alpha_set(window: *mut Ecore_Wl2_Window, alpha: Eina_Bool);
        fn ecore_wl2_window_show(window: *mut Ecore_Wl2_Window);
        fn ecore_wl2_window_hide(window: *mut Ecore_Wl2_Window);
        fn ecore_wl2_window_activate(window: *mut Ecore_Wl2_Window);
        fn ecore_wl2_window_commit(window: *mut Ecore_Wl2_Window, flush: Eina_Bool);
        fn ecore_wl2_window_native_surface_get(window: *mut Ecore_Wl2_Window) -> *mut c_void;
        fn ecore_wl2_window_frame_callback_add(
            window: *mut Ecore_Wl2_Window,
            cb: Ecore_Wl2_Frame_Cb,
            data: *mut c_void
        ) -> *mut Ecore_Wl2_Frame_Cb_Handle;
        fn ecore_wl2_window_frame_callback_del(handle: *mut Ecore_Wl2_Frame_Cb_Handle);
        fn ecore_wl2_window_keygrab_set(
            window: *mut Ecore_Wl2_Window,
            key: *const c_char,
            modifiers: c_int,
            not_modifiers: c_int,
            priority: c_int,
            grab_mode: Ecore_Wl2_Window_Keygrab_Mode
        ) -> Eina_Bool;
        fn ecore_wl2_window_keygrab_unset(
            window: *mut Ecore_Wl2_Window,
            key: *const c_char,
            modifiers: c_int,
            any_modifiers: c_int
        ) -> Eina_Bool;
    }
    // ecore-imf-1/Ecore_IMF.h (EFL > Ecore > Ecore IMF)
    "ecore_imf" {
        fn ecore_imf_init() -> c_int;
        fn ecore_imf_shutdown() -> c_int;
        fn ecore_imf_context_default_id_get() -> *const c_char;
        fn ecore_imf_context_add(id: *const c_char) -> *mut Ecore_IMF_Context;
        fn ecore_imf_context_del(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_client_window_set(ctx: *mut Ecore_IMF_Context, window: *mut c_void);
        fn ecore_imf_context_focus_in(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_focus_out(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_reset(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_cursor_position_set(ctx: *mut Ecore_IMF_Context, cursor_pos: c_int);
        fn ecore_imf_context_input_panel_show(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_input_panel_hide(ctx: *mut Ecore_IMF_Context);
        fn ecore_imf_context_preedit_string_get(
            ctx: *mut Ecore_IMF_Context,
            text: *mut *mut c_char,
            cursor_pos: *mut c_int
        );
        fn ecore_imf_context_event_callback_add(
            ctx: *mut Ecore_IMF_Context,
            kind: Ecore_IMF_Callback_Type,
            func: Ecore_IMF_Event_Cb,
            data: *const c_void
        );
        fn ecore_imf_context_event_callback_del(
            ctx: *mut Ecore_IMF_Context,
            kind: Ecore_IMF_Callback_Type,
            func: Ecore_IMF_Event_Cb
        ) -> *mut c_void;
    }
    // system/system_settings.h (System > System Settings)
    "capi-system-system-settings" {
        fn system_settings_get_value_int(key: system_settings_key_e, value: *mut c_int) -> c_int;
        fn system_settings_get_value_bool(key: system_settings_key_e, value: *mut bool) -> c_int;
        fn system_settings_get_value_string(key: system_settings_key_e, value: *mut *mut c_char) -> c_int;
        fn system_settings_add_changed_cb(
            key: system_settings_key_e,
            callback: system_settings_changed_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn system_settings_remove_changed_cb(
            key: system_settings_key_e,
            callback: system_settings_changed_cb
        ) -> c_int;
    }
    // system/system_info.h (System > System Information)
    "capi-system-info" {
        fn system_info_get_platform_string(key: *const c_char, value: *mut *mut c_char) -> c_int;
    }
    // dlog/dlog.h (System > dlog)
    "dlog" {
        fn dlog_print(prio: log_priority, tag: *const c_char, fmt: *const c_char; ...) -> c_int;
    }
}
