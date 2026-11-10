use super::*;

#[test]
fn the_keys_are_the_documented_tizen_keys() {
    for key in PRODUCT_STRING_KEYS.iter().chain(PRODUCT_INT_KEYS) {
        assert!(
            key.starts_with("http://tizen.org/system/")
                || key.starts_with("http://tizen.org/feature/"),
            "{key}"
        );
        assert!(c_key(key).is_some(), "{key}");
    }
    assert_eq!(MODEL_NAME, "http://tizen.org/system/model_name");
    assert_eq!(PLATFORM_NAME, "http://tizen.org/system/platform.name");
    assert_eq!(
        PLATFORM_VERSION,
        "http://tizen.org/feature/platform.version"
    );
    let mut all: Vec<_> = PRODUCT_STRING_KEYS.iter().chain(PRODUCT_INT_KEYS).collect();
    all.sort();
    all.dedup();
    assert_eq!(
        all.len(),
        PRODUCT_STRING_KEYS.len() + PRODUCT_INT_KEYS.len()
    );
}

#[test]
fn product_info_reads_each_key_into_its_field() {
    let info = ProductInfo::read(
        |key| Some(key.rsplit('/').next().unwrap().to_owned()),
        |key| match key {
            SCREEN_WIDTH => Some(1920),
            SCREEN_HEIGHT => Some(1080),
            _ => None,
        },
    );
    assert_eq!(info.model_name.as_deref(), Some("model_name"));
    assert_eq!(info.manufacturer.as_deref(), Some("manufacturer"));
    assert_eq!(info.platform_name.as_deref(), Some("platform.name"));
    assert_eq!(info.platform_version.as_deref(), Some("platform.version"));
    assert_eq!(
        info.platform_version_name.as_deref(),
        Some("platform.version.name")
    );
    assert_eq!(
        info.native_api_version.as_deref(),
        Some("platform.native.api.version")
    );
    assert_eq!(info.profile.as_deref(), Some("profile"));
    assert_eq!(info.build_string.as_deref(), Some("build.string"));
    assert_eq!(info.build_date.as_deref(), Some("build.date"));
    assert_eq!(info.cpu_arch.as_deref(), Some("platform.core.cpu.arch"));
    assert_eq!(
        (info.screen_width, info.screen_height),
        (Some(1920), Some(1080))
    );
}

#[test]
fn unknown_keys_read_as_none() {
    assert_eq!(
        ProductInfo::read(|_| None, |_| None),
        ProductInfo::default()
    );
}

#[test]
fn values_and_keys_are_converted() {
    assert_eq!(string_value(b"  QE55Q80C\n").as_deref(), Some("QE55Q80C"));
    assert_eq!(string_value(b""), None);
    assert_eq!(string_value(b"   "), None);
    assert_eq!(
        string_value(b"Tizen \xff").as_deref(),
        Some("Tizen \u{fffd}")
    );
    assert!(c_key("http://tizen.org/system/model_name").is_some());
    assert!(c_key("bad\0key").is_none());
}

#[cfg(not(target_vendor = "tizen"))]
#[test]
fn off_tizen_every_read_is_none() {
    assert_eq!(platform_string(MODEL_NAME), None);
    assert_eq!(platform_int(SCREEN_WIDTH), None);
    assert_eq!(platform_bool("http://tizen.org/feature/screen"), None);
    assert_eq!(product_info(), ProductInfo::default());
}
