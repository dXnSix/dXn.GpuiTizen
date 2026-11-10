//! `system_info` for `crate::system_info`: one documented platform key per call. A failed read
//! (an unknown key, an I/O or permission error) is `None`.

use std::ffi::{CStr, CString, c_char, c_int};
use std::ptr;

use dxn_gpui_tizen_sys as sys;

use crate::system_info::string_value;

pub(crate) fn platform_string(key: &CString) -> Option<String> {
    let mut value: *mut c_char = ptr::null_mut();
    // SAFETY: a NUL-terminated key and an out pointer; on success the value is a malloc'd
    // NUL-terminated string (`system_info.h`: "must be released using free()"), freed here.
    unsafe {
        if sys::system_info_get_platform_string(key.as_ptr(), &mut value)
            != sys::SYSTEM_INFO_ERROR_NONE
            || value.is_null()
        {
            return None;
        }
        let text = string_value(CStr::from_ptr(value).to_bytes());
        libc::free(value.cast());
        text
    }
}

pub(crate) fn platform_int(key: &CString) -> Option<i32> {
    let mut value: c_int = 0;
    // SAFETY: a NUL-terminated key and an out pointer.
    let result = unsafe { sys::system_info_get_platform_int(key.as_ptr(), &mut value) };
    (result == sys::SYSTEM_INFO_ERROR_NONE).then_some(value)
}

pub(crate) fn platform_bool(key: &CString) -> Option<bool> {
    let mut value = false;
    // SAFETY: a NUL-terminated key and an out pointer.
    let result = unsafe { sys::system_info_get_platform_bool(key.as_ptr(), &mut value) };
    (result == sys::SYSTEM_INFO_ERROR_NONE).then_some(value)
}
