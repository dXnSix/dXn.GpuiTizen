//! `system_settings` for `crate::settings` (design `d17-settings-fonts`): the settings are read at
//! start and again after change callbacks, a burst of callbacks coalesced into one read on the
//! main loop. A key the TV does not support (or any error) reads as `None`.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr;
use std::sync::{Arc, OnceLock};

use dxn_gpui_tizen_sys as sys;

use super::main_loop::EcoreLoop;
use crate::dispatcher::MainLoop;
use crate::settings::{self, Coalescer, RawSettings, SystemSettings};

/// The keys whose changes are watched.
const WATCHED: [sys::system_settings_key_e; 5] = [
    sys::SYSTEM_SETTINGS_KEY_LOCALE_LANGUAGE,
    sys::SYSTEM_SETTINGS_KEY_LOCALE_COUNTRY,
    sys::SYSTEM_SETTINGS_KEY_LOCALE_TIMEZONE,
    sys::SYSTEM_SETTINGS_KEY_FONT_SIZE,
    sys::SYSTEM_SETTINGS_KEY_ACCESSIBILITY_TTS,
];

static COALESCER: Coalescer = Coalescer::new();
static MAIN_LOOP: OnceLock<Arc<EcoreLoop>> = OnceLock::new();

fn string(key: sys::system_settings_key_e) -> Option<String> {
    let mut value: *mut c_char = ptr::null_mut();
    // SAFETY: an out pointer; on success the value is a malloc'd string, freed here.
    unsafe {
        if sys::system_settings_get_value_string(key, &mut value) != sys::SYSTEM_SETTINGS_ERROR_NONE
            || value.is_null()
        {
            return None;
        }
        let text = CStr::from_ptr(value).to_string_lossy().into_owned();
        libc::free(value.cast());
        Some(text)
    }
}

fn int(key: sys::system_settings_key_e) -> Option<i32> {
    let mut value: c_int = 0;
    // SAFETY: an out pointer.
    let result = unsafe { sys::system_settings_get_value_int(key, &mut value) };
    (result == sys::SYSTEM_SETTINGS_ERROR_NONE).then_some(value)
}

fn boolean(key: sys::system_settings_key_e) -> Option<bool> {
    let mut value = false;
    // SAFETY: an out pointer.
    let result = unsafe { sys::system_settings_get_value_bool(key, &mut value) };
    (result == sys::SYSTEM_SETTINGS_ERROR_NONE).then_some(value)
}

fn read() -> RawSettings {
    RawSettings {
        language: string(sys::SYSTEM_SETTINGS_KEY_LOCALE_LANGUAGE),
        country: string(sys::SYSTEM_SETTINGS_KEY_LOCALE_COUNTRY),
        time_zone: string(sys::SYSTEM_SETTINGS_KEY_LOCALE_TIMEZONE),
        font_size: int(sys::SYSTEM_SETTINGS_KEY_FONT_SIZE),
        tts: boolean(sys::SYSTEM_SETTINGS_KEY_ACCESSIBILITY_TTS),
    }
}

fn publish() {
    COALESCER.reading();
    let settings = SystemSettings::from_raw(&read());
    log::info!("system settings: {settings:?}");
    settings::publish(settings);
}

unsafe extern "C" fn on_changed(_key: sys::system_settings_key_e, _data: *mut c_void) {
    if COALESCER.changed()
        && let Some(main_loop) = MAIN_LOOP.get()
    {
        main_loop.post(Box::new(publish));
    }
}

/// Reads the settings and watches them (main thread, before the app launches).
pub(crate) fn start(main_loop: &Arc<EcoreLoop>) {
    let _ = MAIN_LOOP.set(Arc::clone(main_loop));
    publish();
    for key in WATCHED {
        // SAFETY: a static callback that only posts to the main loop.
        let result =
            unsafe { sys::system_settings_add_changed_cb(key, Some(on_changed), ptr::null_mut()) };
        if result != sys::SYSTEM_SETTINGS_ERROR_NONE {
            log::info!("system setting {key} is not watched ({result})");
        }
    }
}

/// Stops watching (terminate).
pub(crate) fn stop() {
    for key in WATCHED {
        // SAFETY: removes the callback `start` added (an error when it was never added).
        unsafe { sys::system_settings_remove_changed_cb(key, Some(on_changed)) };
    }
}
