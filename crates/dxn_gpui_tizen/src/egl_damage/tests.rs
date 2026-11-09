use gpui::{Bounds, DevicePixels, point, size};

use super::*;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Bounds<DevicePixels> {
    Bounds {
        origin: point(DevicePixels(x), DevicePixels(y)),
        size: size(DevicePixels(w), DevicePixels(h)),
    }
}

const MESA: &str =
    "EGL_EXT_buffer_age EGL_KHR_partial_update EGL_KHR_swap_buffers_with_damage EGL_KHR_image_base";

#[test]
fn support_comes_from_whole_extension_names() {
    assert_eq!(
        DamageSupport::from_extensions(MESA),
        DamageSupport {
            buffer_age: true,
            partial_update: true,
            swap_with_damage_khr: true,
            swap_with_damage_ext: false,
        }
    );
    let ext_only = DamageSupport::from_extensions("EGL_EXT_swap_buffers_with_damage");
    assert_eq!(
        ext_only.swap_with_damage_name(),
        Some("eglSwapBuffersWithDamageEXT")
    );
    assert_eq!(
        DamageSupport::from_extensions(MESA).swap_with_damage_name(),
        Some("eglSwapBuffersWithDamageKHR")
    );
    assert_eq!(
        DamageSupport::from_extensions("EGL_EXT_buffer_age_x"),
        DamageSupport::default()
    );
}

#[test]
fn the_age_is_zero_unless_the_driver_reports_one() {
    let support = DamageSupport::from_extensions(MESA);
    assert_eq!(support.age(Some(2)), 2);
    assert_eq!(support.age(Some(0)), 0, "undefined contents");
    assert_eq!(support.age(Some(-1)), 0);
    assert_eq!(support.age(None), 0, "the query failed");
    assert_eq!(
        DamageSupport::default().age(Some(3)),
        0,
        "no EGL_EXT_buffer_age"
    );
}

#[test]
fn a_partial_frame_declares_its_region_and_swaps_its_damage() {
    let support = DamageSupport::from_extensions(MESA);
    let damage = [rect(10, 20, 30, 40)];
    let redraw = [rect(5, 20, 35, 50)];
    assert_eq!(
        plan(&support, &damage, &redraw),
        Plan {
            set_damage_region: Some(vec![5, 20, 35, 50]),
            swap: Swap::WithDamage(vec![10, 20, 30, 40]),
        }
    );
    assert_eq!(
        egl_rects(&[rect(1, 2, 3, 4), rect(5, 6, 7, 8)]),
        [1, 2, 3, 4, 5, 6, 7, 8]
    );
}

#[test]
fn without_the_extensions_the_frame_is_swapped_whole() {
    let damage = [rect(10, 20, 30, 40)];
    assert_eq!(
        plan(&DamageSupport::default(), &damage, &damage),
        Plan {
            set_damage_region: None,
            swap: Swap::Full,
        }
    );
}

#[test]
fn nothing_drawn_means_no_swap() {
    let support = DamageSupport::from_extensions(MESA);
    assert_eq!(plan(&support, &[], &[]).swap, Swap::None);
    // Redrawn for an older buffer, though this frame changed nothing.
    let redraw = [rect(0, 0, 8, 8)];
    assert_eq!(
        plan(&support, &[], &redraw),
        Plan {
            set_damage_region: Some(vec![0, 0, 8, 8]),
            swap: Swap::Full,
        }
    );
}

#[test]
fn a_frame_failing_with_its_region_in_force_makes_the_surface_again() {
    let mut frame = FrameState::default();
    assert_eq!(frame.failed(), Recovery::None, "no region: nothing to undo");

    frame.region_set();
    assert!(frame.region_open());
    frame.swapped();
    assert_eq!(frame.failed(), Recovery::None, "the swap ended the region");

    frame.region_set();
    assert_eq!(frame.failed(), Recovery::NewSurface);
    assert!(!frame.region_open(), "the new surface has no region");
    assert_eq!(frame.failed(), Recovery::None);
}
