//! Platform information (`system_info`, `system/system_info.h`): what the TV is and runs, by the
//! documented Tizen 8.0 keys. [`platform_string`], [`platform_int`] and [`platform_bool`] read one
//! key; [`product_info`] reads the keys an app shows as product information. A key the TV does
//! not know, a failed read, an empty string or a key with a NUL byte yields `None`, never an
//! error. Off Tizen every read is `None`. The functions may be called from any thread.

/// `http://tizen.org/system/model_name` (string): the model, such as `QE55Q80C`.
pub const MODEL_NAME: &str = "http://tizen.org/system/model_name";
/// `http://tizen.org/system/manufacturer` (string).
pub const MANUFACTURER: &str = "http://tizen.org/system/manufacturer";
/// `http://tizen.org/system/platform.name` (string): `Tizen`.
pub const PLATFORM_NAME: &str = "http://tizen.org/system/platform.name";
/// `http://tizen.org/feature/platform.version` (string): the platform version, such as `8.0`.
pub const PLATFORM_VERSION: &str = "http://tizen.org/feature/platform.version";
/// `http://tizen.org/feature/platform.version.name` (string): the platform version's name.
pub const PLATFORM_VERSION_NAME: &str = "http://tizen.org/feature/platform.version.name";
/// `http://tizen.org/feature/platform.native.api.version` (string): the native API version.
pub const NATIVE_API_VERSION: &str = "http://tizen.org/feature/platform.native.api.version";
/// `http://tizen.org/feature/profile` (string): the profile, `tv` on a TV.
pub const PROFILE: &str = "http://tizen.org/feature/profile";
/// `http://tizen.org/system/build.string` (string): the platform build.
pub const BUILD_STRING: &str = "http://tizen.org/system/build.string";
/// `http://tizen.org/system/build.date` (string).
pub const BUILD_DATE: &str = "http://tizen.org/system/build.date";
/// `http://tizen.org/feature/platform.core.cpu.arch` (string): `armv7`, `armv8`, `x86_64`, ...
pub const CPU_ARCH: &str = "http://tizen.org/feature/platform.core.cpu.arch";
/// `http://tizen.org/feature/screen.width` (int): the screen width in pixels.
pub const SCREEN_WIDTH: &str = "http://tizen.org/feature/screen.width";
/// `http://tizen.org/feature/screen.height` (int): the screen height in pixels.
pub const SCREEN_HEIGHT: &str = "http://tizen.org/feature/screen.height";

/// The string keys [`product_info`] reads, in its fields' order.
pub const PRODUCT_STRING_KEYS: &[&str] = &[
    MODEL_NAME,
    MANUFACTURER,
    PLATFORM_NAME,
    PLATFORM_VERSION,
    PLATFORM_VERSION_NAME,
    NATIVE_API_VERSION,
    PROFILE,
    BUILD_STRING,
    BUILD_DATE,
    CPU_ARCH,
];

/// The int keys [`product_info`] reads.
pub const PRODUCT_INT_KEYS: &[&str] = &[SCREEN_WIDTH, SCREEN_HEIGHT];

/// The product information an app shows; `None` where the TV does not report it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProductInfo {
    /// [`MODEL_NAME`].
    pub model_name: Option<String>,
    /// [`MANUFACTURER`].
    pub manufacturer: Option<String>,
    /// [`PLATFORM_NAME`].
    pub platform_name: Option<String>,
    /// [`PLATFORM_VERSION`].
    pub platform_version: Option<String>,
    /// [`PLATFORM_VERSION_NAME`].
    pub platform_version_name: Option<String>,
    /// [`NATIVE_API_VERSION`].
    pub native_api_version: Option<String>,
    /// [`PROFILE`].
    pub profile: Option<String>,
    /// [`BUILD_STRING`].
    pub build_string: Option<String>,
    /// [`BUILD_DATE`].
    pub build_date: Option<String>,
    /// [`CPU_ARCH`].
    pub cpu_arch: Option<String>,
    /// [`SCREEN_WIDTH`].
    pub screen_width: Option<i32>,
    /// [`SCREEN_HEIGHT`].
    pub screen_height: Option<i32>,
}

impl ProductInfo {
    /// Reads every product key through `string` and `int` (the platform's readers, or fakes).
    pub fn read(
        string: impl Fn(&str) -> Option<String>,
        int: impl Fn(&str) -> Option<i32>,
    ) -> Self {
        Self {
            model_name: string(MODEL_NAME),
            manufacturer: string(MANUFACTURER),
            platform_name: string(PLATFORM_NAME),
            platform_version: string(PLATFORM_VERSION),
            platform_version_name: string(PLATFORM_VERSION_NAME),
            native_api_version: string(NATIVE_API_VERSION),
            profile: string(PROFILE),
            build_string: string(BUILD_STRING),
            build_date: string(BUILD_DATE),
            cpu_arch: string(CPU_ARCH),
            screen_width: int(SCREEN_WIDTH),
            screen_height: int(SCREEN_HEIGHT),
        }
    }
}

/// A key as `system_info` takes it: `None` for a key with a NUL byte.
pub fn c_key(key: &str) -> Option<std::ffi::CString> {
    std::ffi::CString::new(key).ok()
}

/// A string as `system_info` returned it (lossy UTF-8), trimmed; `None` when empty.
pub fn string_value(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The platform string for `key` (`system_info_get_platform_string`, its value freed here).
pub fn platform_string(key: &str) -> Option<String> {
    #[cfg(target_vendor = "tizen")]
    {
        crate::tizen::system_info::platform_string(&c_key(key)?)
    }
    #[cfg(not(target_vendor = "tizen"))]
    {
        let _ = key;
        None
    }
}

/// The platform int for `key` (`system_info_get_platform_int`).
pub fn platform_int(key: &str) -> Option<i32> {
    #[cfg(target_vendor = "tizen")]
    {
        crate::tizen::system_info::platform_int(&c_key(key)?)
    }
    #[cfg(not(target_vendor = "tizen"))]
    {
        let _ = key;
        None
    }
}

/// The platform bool for `key` (`system_info_get_platform_bool`), such as a feature key.
pub fn platform_bool(key: &str) -> Option<bool> {
    #[cfg(target_vendor = "tizen")]
    {
        crate::tizen::system_info::platform_bool(&c_key(key)?)
    }
    #[cfg(not(target_vendor = "tizen"))]
    {
        let _ = key;
        None
    }
}

/// The TV's product information (every field `None` off Tizen).
pub fn product_info() -> ProductInfo {
    ProductInfo::read(platform_string, platform_int)
}

#[cfg(test)]
mod tests;
