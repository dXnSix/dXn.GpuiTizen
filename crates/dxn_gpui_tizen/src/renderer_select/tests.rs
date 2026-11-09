use super::*;

#[test]
fn auto_is_the_default_and_the_environment_overrides_the_options() {
    assert_eq!(Renderer::default(), Renderer::Auto);
    assert_eq!(requested(Renderer::Auto, None), (Renderer::Auto, None));
    assert_eq!(
        requested(Renderer::Auto, Some("wgpu")),
        (Renderer::WgpuSurface, None)
    );
    assert_eq!(
        requested(Renderer::WgpuSurface, Some(" EGL ")),
        (Renderer::PlatformEgl, None)
    );
    assert_eq!(
        requested(Renderer::PlatformEgl, Some("auto")).0,
        Renderer::Auto
    );
    let (renderer, warning) = requested(Renderer::WgpuSurface, Some("vulkan"));
    assert_eq!(
        renderer,
        Renderer::WgpuSurface,
        "an unknown value keeps the options'"
    );
    assert!(warning.unwrap().contains("vulkan"));
    for (value, renderer) in [
        ("platform-egl", Renderer::PlatformEgl),
        ("wgpu-surface", Renderer::WgpuSurface),
    ] {
        assert_eq!(Renderer::parse(value), Some(renderer));
    }
}

#[test]
fn auto_falls_back_to_the_wgpu_surface_with_its_reason() {
    assert!(tries_egl(Renderer::Auto));
    assert_eq!(choose(Renderer::Auto, Some(Ok(()))), Choice::PlatformEgl);
    assert_eq!(
        choose(Renderer::Auto, Some(Err("no GL ES 3 window config".into()))),
        Choice::WgpuSurface {
            fallback: Some("no GL ES 3 window config".into())
        }
    );
}

#[test]
fn an_explicit_choice_is_kept() {
    assert!(!tries_egl(Renderer::WgpuSurface));
    assert_eq!(
        choose(Renderer::WgpuSurface, None),
        Choice::WgpuSurface { fallback: None }
    );
    assert_eq!(
        choose(Renderer::PlatformEgl, Some(Ok(()))),
        Choice::PlatformEgl
    );
    assert_eq!(
        choose(
            Renderer::PlatformEgl,
            Some(Err("loading libEGL.so.1".into()))
        ),
        Choice::Failed("loading libEGL.so.1".into()),
        "a requested EGL path does not fall back silently"
    );
}

#[test]
fn the_fallback_is_logged_once() {
    let mut log = FallbackLog::default();
    let fallback = choose(
        Renderer::Auto,
        Some(Err("eglInitialize: BadDisplay".into())),
    );
    let first = log.message(&fallback).unwrap();
    assert!(first.contains("BadDisplay"));
    assert_eq!(log.message(&fallback), None, "not again after a resume");
    assert_eq!(log.message(&Choice::PlatformEgl), None);
}
