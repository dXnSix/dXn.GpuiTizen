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
    /// `Ecore_Job` (`ecore-1/Ecore_Legacy.h`).
    Ecore_Job,
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

/// `app_control_extra_data_cb` (`appfw/app_control.h`): return `true` to continue.
pub type app_control_extra_data_cb = Option<
    unsafe extern "C" fn(
        app_control: app_control_h,
        key: *const c_char,
        user_data: *mut c_void,
    ) -> bool,
>;

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

/// `Ecore_IMF_Event_Type` (`ecore-imf-1/Ecore_IMF.h`), in header order from 0.
pub type Ecore_IMF_Event_Type = c_int;
pub const ECORE_IMF_EVENT_KEY_DOWN: Ecore_IMF_Event_Type = 6;
pub const ECORE_IMF_EVENT_KEY_UP: Ecore_IMF_Event_Type = 7;

/// `struct _Ecore_IMF_Event_Key_Down` and `_Key_Up` (`ecore-imf-1/Ecore_IMF.h`), which share a
/// layout, in header order.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ecore_IMF_Event_Key {
    pub keyname: *const c_char,
    /// `Ecore_IMF_Keyboard_Modifiers` (an enum of bits).
    pub modifiers: c_int,
    /// `Ecore_IMF_Keyboard_Locks` (an enum of bits).
    pub locks: c_int,
    pub key: *const c_char,
    pub string: *const c_char,
    pub compose: *const c_char,
    pub timestamp: c_uint,
    pub dev_name: *const c_char,
    /// `Ecore_IMF_Device_Class` (an enum; 0 is none).
    pub dev_class: c_int,
    /// `Ecore_IMF_Device_Subclass` (an enum; 0 is none).
    pub dev_subclass: c_int,
    pub keycode: c_uint,
}

/// `union _Ecore_IMF_Event` (`ecore-imf-1/Ecore_IMF.h`): the key members, padded to the size of
/// its largest member (`Ecore_IMF_Event_Mouse_Move`, twelve `int`s).
#[repr(C)]
pub union Ecore_IMF_Event {
    pub key_down: Ecore_IMF_Event_Key,
    pub key_up: Ecore_IMF_Event_Key,
    pub mouse_move: [c_int; 12],
}

/// The `retrieve_surrounding` callback of `ecore_imf_context_retrieve_surrounding_callback_set`
/// (`ecore-imf-1/Ecore_IMF.h`): `text` is malloc'd by the callback and freed by Ecore_IMF.
pub type Ecore_IMF_Retrieve_Surrounding_Cb = Option<
    unsafe extern "C" fn(
        data: *mut c_void,
        ctx: *mut Ecore_IMF_Context,
        text: *mut *mut c_char,
        cursor_pos: *mut c_int,
    ) -> Eina_Bool,
>;

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
/// `SYSTEM_INFO_ERROR_NONE` (`system/system_info_type.h`, `TIZEN_ERROR_NONE`).
pub const SYSTEM_INFO_ERROR_NONE: c_int = 0;

/// `system_settings_changed_cb` (`system/system_settings.h`).
pub type system_settings_changed_cb =
    Option<unsafe extern "C" fn(key: system_settings_key_e, user_data: *mut c_void)>;

/// `log_priority` (`dlog/dlog.h`).
pub type log_priority = c_int;
pub const DLOG_DEBUG: log_priority = 3;
pub const DLOG_INFO: log_priority = 4;
pub const DLOG_WARN: log_priority = 5;
pub const DLOG_ERROR: log_priority = 6;

/// `player_h` (`media/player.h`).
#[repr(C)]
pub struct player_s {
    _private: [u8; 0],
}
pub type player_h = *mut player_s;
/// `player_display_h` (`media/player.h`): for `PLAYER_DISPLAY_TYPE_OVERLAY_SYNC_UI`, the exported
/// shell handle of an `Ecore_Wl2_Subsurface` (`ecore_wl2_subsurface_exported_surface_handle_get`);
/// the plain overlay type takes an Evas object, which a native Ecore_Wl2 window does not have.
pub type player_display_h = *mut c_void;
/// `player_state_e`.
pub type player_state_e = c_int;
pub const PLAYER_STATE_NONE: player_state_e = 0;
pub const PLAYER_STATE_IDLE: player_state_e = 1;
pub const PLAYER_STATE_READY: player_state_e = 2;
pub const PLAYER_STATE_PLAYING: player_state_e = 3;
pub const PLAYER_STATE_PAUSED: player_state_e = 4;
/// `player_display_type_e`.
pub type player_display_type_e = c_int;
pub const PLAYER_DISPLAY_TYPE_OVERLAY: player_display_type_e = 0;
pub const PLAYER_DISPLAY_TYPE_NONE: player_display_type_e = 4;
pub const PLAYER_DISPLAY_TYPE_OVERLAY_SYNC_UI: player_display_type_e = 5;
/// `player_interrupted_code_e`.
pub type player_interrupted_code_e = c_int;
pub const PLAYER_INTERRUPTED_BY_RESOURCE_CONFLICT: player_interrupted_code_e = 4;

/// `Ecore_Wl2_Subsurface` (`ecore-wl2-1/Ecore_Wayland2.h`).
#[repr(C)]
pub struct Ecore_Wl2_Subsurface {
    _private: [u8; 0],
}

/// `media_packet_h` (`media/media_packet.h`).
#[repr(C)]
pub struct media_packet_s {
    _private: [u8; 0],
}
pub type media_packet_h = *mut media_packet_s;
/// `media_format_h` (`media/media_format.h`).
#[repr(C)]
pub struct media_format_s {
    _private: [u8; 0],
}
pub type media_format_h = *mut media_format_s;
/// `media_format_mimetype_e`.
pub type media_format_mimetype_e = c_int;
const MEDIA_FORMAT_VIDEO: media_format_mimetype_e = 0x0200_0000;
const MEDIA_FORMAT_RAW: media_format_mimetype_e = 0x2000_0000;
pub const MEDIA_FORMAT_I420: media_format_mimetype_e =
    MEDIA_FORMAT_VIDEO | MEDIA_FORMAT_RAW | 0x2510;
pub const MEDIA_FORMAT_NV12: media_format_mimetype_e =
    MEDIA_FORMAT_VIDEO | MEDIA_FORMAT_RAW | 0x2520;
pub const MEDIA_FORMAT_NV21: media_format_mimetype_e =
    MEDIA_FORMAT_VIDEO | MEDIA_FORMAT_RAW | 0x2550;
pub type player_media_packet_video_decoded_cb =
    Option<unsafe extern "C" fn(packet: media_packet_h, user_data: *mut c_void)>;

/// `TIZEN_ERROR_PLAYER` (`tizen_error.h`).
pub const TIZEN_ERROR_PLAYER: c_int = -0x0194_0000;
/// `PLAYER_ERROR_CLASS`: the player's own error codes are this ORed with 0x01..=0x12.
pub const PLAYER_ERROR_CLASS: c_int = TIZEN_ERROR_PLAYER | 0x20;
/// `TIZEN_ERROR_NOT_SUPPORTED`: `TIZEN_ERROR_MIN_PLATFORM_ERROR` + 2.
pub const TIZEN_ERROR_NOT_SUPPORTED: c_int = -1_073_741_824 + 2;

pub type player_prepared_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type player_completed_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type player_seek_completed_cb = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
pub type player_interrupted_cb =
    Option<unsafe extern "C" fn(code: player_interrupted_code_e, user_data: *mut c_void)>;
pub type player_error_cb = Option<unsafe extern "C" fn(error_code: c_int, user_data: *mut c_void)>;
pub type player_buffering_cb = Option<unsafe extern "C" fn(percent: c_int, user_data: *mut c_void)>;

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
        fn app_control_foreach_extra_data(
            app_control: app_control_h,
            callback: app_control_extra_data_cb,
            user_data: *mut c_void
        ) -> c_int;
    }
    // ecore-1/Ecore_Common.h, Ecore_Legacy.h (EFL > Ecore)
    "ecore" {
        fn ecore_main_loop_thread_safe_call_async(callback: Ecore_Cb, data: *mut c_void);
        fn ecore_timer_add(seconds: c_double, func: Ecore_Task_Cb, data: *const c_void) -> *mut Ecore_Timer;
        fn ecore_timer_del(timer: *mut Ecore_Timer) -> *mut c_void;
        fn ecore_job_add(func: Ecore_Cb, data: *const c_void) -> *mut Ecore_Job;
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
        fn ecore_wl2_subsurface_new(window: *mut Ecore_Wl2_Window) -> *mut Ecore_Wl2_Subsurface;
        fn ecore_wl2_subsurface_del(subsurface: *mut Ecore_Wl2_Subsurface);
        fn ecore_wl2_subsurface_export(subsurface: *mut Ecore_Wl2_Subsurface) -> Eina_Bool;
        fn ecore_wl2_subsurface_exported_surface_handle_get(
            subsurface: *mut Ecore_Wl2_Subsurface
        ) -> *const c_char;
        fn ecore_wl2_subsurface_exported_surface_show(subsurface: *mut Ecore_Wl2_Subsurface);
        fn ecore_wl2_subsurface_exported_surface_hide(subsurface: *mut Ecore_Wl2_Subsurface);
        fn ecore_wl2_subsurface_exported_surface_place_below(
            subsurface: *mut Ecore_Wl2_Subsurface,
            other: *mut Ecore_Wl2_Subsurface
        );
        fn ecore_wl2_subsurface_exported_surface_move(
            subsurface: *mut Ecore_Wl2_Subsurface,
            x: c_int,
            y: c_int
        );
        fn ecore_wl2_subsurface_exported_surface_resize(
            subsurface: *mut Ecore_Wl2_Subsurface,
            w: c_int,
            h: c_int
        );
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
        fn ecore_imf_context_cursor_location_set(
            ctx: *mut Ecore_IMF_Context,
            x: c_int,
            y: c_int,
            w: c_int,
            h: c_int
        );
        fn ecore_imf_context_retrieve_surrounding_callback_set(
            ctx: *mut Ecore_IMF_Context,
            func: Ecore_IMF_Retrieve_Surrounding_Cb,
            data: *const c_void
        );
        fn ecore_imf_context_filter_event(
            ctx: *mut Ecore_IMF_Context,
            kind: Ecore_IMF_Event_Type,
            event: *mut Ecore_IMF_Event
        ) -> Eina_Bool;
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
        fn system_info_get_platform_bool(key: *const c_char, value: *mut bool) -> c_int;
        fn system_info_get_platform_int(key: *const c_char, value: *mut c_int) -> c_int;
        fn system_info_get_platform_string(key: *const c_char, value: *mut *mut c_char) -> c_int;
    }
    // media/player.h (Multimedia > Media Player)
    "capi-media-player" {
        fn player_create(player: *mut player_h) -> c_int;
        fn player_destroy(player: player_h) -> c_int;
        fn player_set_uri(player: player_h, uri: *const c_char) -> c_int;
        fn player_prepare_async(
            player: player_h,
            callback: player_prepared_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn player_unprepare(player: player_h) -> c_int;
        fn player_start(player: player_h) -> c_int;
        fn player_pause(player: player_h) -> c_int;
        fn player_stop(player: player_h) -> c_int;
        fn player_get_state(player: player_h, state: *mut player_state_e) -> c_int;
        fn player_set_play_position(
            player: player_h,
            milliseconds: c_int,
            accurate: bool,
            callback: player_seek_completed_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn player_get_play_position(player: player_h, milliseconds: *mut c_int) -> c_int;
        fn player_get_duration(player: player_h, milliseconds: *mut c_int) -> c_int;
        fn player_set_volume(player: player_h, left: f32, right: f32) -> c_int;
        fn player_set_mute(player: player_h, muted: bool) -> c_int;
        fn player_set_looping(player: player_h, looping: bool) -> c_int;
        fn player_set_completed_cb(
            player: player_h,
            callback: player_completed_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn player_set_error_cb(player: player_h, callback: player_error_cb, user_data: *mut c_void) -> c_int;
        fn player_set_buffering_cb(
            player: player_h,
            callback: player_buffering_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn player_set_display(
            player: player_h,
            kind: player_display_type_e,
            display: player_display_h
        ) -> c_int;
        fn player_set_media_packet_video_frame_decoded_cb(
            player: player_h,
            callback: player_media_packet_video_decoded_cb,
            user_data: *mut c_void
        ) -> c_int;
        fn player_set_interrupted_cb(
            player: player_h,
            callback: player_interrupted_cb,
            user_data: *mut c_void
        ) -> c_int;
    }
    // media/media_packet.h, media/media_format.h (Multimedia > Media Tool)
    "capi-media-tool" {
        fn media_packet_get_format(packet: media_packet_h, fmt: *mut media_format_h) -> c_int;
        fn media_packet_get_number_of_video_planes(packet: media_packet_h, num: *mut u32) -> c_int;
        fn media_packet_get_video_plane_data_ptr(
            packet: media_packet_h,
            plane_idx: c_int,
            plane_data_ptr: *mut *mut c_void
        ) -> c_int;
        fn media_packet_get_video_stride_width(
            packet: media_packet_h,
            plane_idx: c_int,
            stride_width: *mut c_int
        ) -> c_int;
        fn media_packet_get_video_stride_height(
            packet: media_packet_h,
            plane_idx: c_int,
            stride_height: *mut c_int
        ) -> c_int;
        fn media_packet_unref(packet: media_packet_h) -> c_int;
        fn media_format_get_video_info(
            fmt: media_format_h,
            mimetype: *mut media_format_mimetype_e,
            width: *mut c_int,
            height: *mut c_int,
            avg_bps: *mut c_int,
            max_bps: *mut c_int
        ) -> c_int;
        fn media_format_unref(fmt: media_format_h) -> c_int;
    }
    // dlog/dlog.h (System > dlog)
    "dlog" {
        fn dlog_print(prio: log_priority, tag: *const c_char, fmt: *const c_char; ...) -> c_int;
    }
}
