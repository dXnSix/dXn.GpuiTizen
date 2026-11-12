# egl14-wayland

| | |
|---|---|
| Upstream PR | ready, not opened (the owner opens it; [../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Issue | [gfx-rs/wgpu#10708](https://github.com/gfx-rs/wgpu/issues/10708) (open, by zhangzhaohong-dev, 2026-10-22): Mali-G76, EGL 1.4 on Wayland, adapter found but not presentable |
| Branch | `egl14-wayland` on `dXnSix/wgpu`, head `ef3af726c`, pushed 2026-11-12 (rebased from `01f93a569`) |
| Base | `upstream/trunk` `c92258fca` (#10889, fetched 2026-11-12); the 13 commits since `b672a5400` touch neither `egl.rs` nor the changelog entry, and the rebase was clean |
| Size | 2 files, +406 −97: `wgpu-hal/src/gles/egl.rs` (code and its tests), `CHANGELOG.md` (+1) |
| Its commits | `ef3af726c` [gles] Support EGL 1.4 drivers on Wayland |
| Text | [PR.md](PR.md) |
| From v30 | `66e825217` on `v30-egl14`, merged into `v30-tizen` |
| Needs | nothing; no public API |

The issue names three gaps on an EGL 1.4 driver on Wayland, and the commit closes all three, each decided in a pure
function with unit tests:

- **The display.** The Wayland arm needed EGL 1.5 (`upcast::<EGL1_5>()`, which needs the `eglGetPlatformDisplay`
  symbol), so EGL 1.4 fell back to `eglGetDisplay(EGL_DEFAULT_DISPLAY)` with `WindowKind::Unknown`, and
  `Surface::configure` refused the Wayland window. `wayland_display_path` now picks EGL 1.5's
  `eglGetPlatformDisplay`, else `eglGetPlatformDisplayEXT` (EGL 1.4 with `EGL_EXT_platform_base` and
  `EGL_EXT_platform_wayland`, extension names matched whole), else `eglGetDisplay` on the `wl_display` (as
  flutter-tizen and DALi do), all three as `WindowKind::Wayland`.
- **The config tier.** On Wayland the "presentation" tier (`EGL_WINDOW_BIT`) is enough to present
  (`presentable_tier`): `wl_egl_window` buffers are the platform's, and the driver reports
  `EGL_NATIVE_RENDERABLE = 0` on every config. X11 and the rest keep requiring native rendering. `choose_config`'s
  tier loop becomes `choose_tier`, tested with fake configs.
- **The render buffer.** When `eglCreateWindowSurface` refuses `EGL_SINGLE_BUFFER` with `EGL_BAD_ATTRIBUTE` or
  `EGL_BAD_MATCH`, the surface is created once more with `EGL_BACK_BUFFER` (`render_buffer_retry`).

Not covered: the surfaceless arm's `egl1_5.expect(...)` on EGL 1.4 that the issue mentions (its `TODO` stays as it
is); it is only reached without a window system.

### Ported from v30

Cherry-picked from `66e825217`; adapted to `trunk`:

- `choose_tier` keeps #10048 (one `SURFACE_TYPE` attribute holding the tiers' bits; duplicate keys made Mesa return
  no configs). The test helper asserts every attribute list has exactly one `SURFACE_TYPE`.
- The display match keeps #9422's ANGLE D3D11 arm ahead of the new Wayland arm.
- `config.format.is_srgb()` is `has_srgb_suffix()` (#9758).
- The `CHANGELOG.md` entry is under `## Unreleased` → Bug Fixes → GLES. It links the issue; replace that with the PR
  link once the PR exists (`By @dXnSix in [#NNNNN](https://github.com/gfx-rs/wgpu/pull/NNNNN)`).

## Checks

Run 2026-11-12 on `ef3af726c`, in the Lima VM (see [../README.md](../README.md#checks)), from the worktree:

```bash
cd /Users/dinko/work-private/projects/dXn/wgpu-wt/trunk      # egl14-wayland checked out
/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/scripts/lima-run.sh cargo test -p wgpu-hal --features gles --lib
#   test result: ok. 9 passed; 0 failed  (gles::egl::tests::display_path, ::config_tiers, ::render_buffer, and six existing)
/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/scripts/lima-run.sh cargo clippy -p wgpu-hal --features gles --all-targets -- -D warnings
#   passed
cargo fmt --check -p wgpu-hal                                # on the Mac: clean
```

Logs: `artifacts-temp/agentx/tizen-gpui-platform/t25-wgpu-egl-lib.log`, `t25-wgpu-egl-clippy.log`,
`t25-wgpu-egl-fmt.log`. The first run (2026-11-10, on `01f93a569` over `b672a5400`) passed the same: 6 tests then.

Not run: on an EGL 1.4 driver (Mali on Wayland, a Tizen TV) from this branch. Mesa in the VM is EGL 1.5, so a Wayland
run there (`scripts/lima-run.sh --weston`) takes the `eglGetPlatformDisplay` path and does not reach the EGL 1.4
paths; the unit tests cover the choices, not the driver calls. The full `cargo xtask test` and other platforms were
not run.

## Updating and opening

Rebase onto the latest `upstream/trunk`, run the checks, fill in the PR text's "LLM Use?" section and rewrite its
description in your own words (wgpu's policy, see [../README.md](../README.md#rules-for-this-area)), tick the
checklist. The owner runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/wgpu
git fetch upstream origin
git worktree add ../wgpu-wt/trunk egl14-wayland 2>/dev/null || git -C ../wgpu-wt/trunk switch egl14-wayland
cd ../wgpu-wt/trunk
git rebase upstream/trunk          # only if trunk moved again since c92258fca; then the checks above and
git push --force-with-lease origin egl14-wayland
# owner runs this
gh pr create --repo gfx-rs/wgpu --base trunk --head dXnSix:egl14-wayland \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/wgpu/egl14-wayland/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/wgpu/egl14-wayland/PR.md)
cd /Users/dinko/work-private/projects/dXn/wgpu && git worktree remove ../wgpu-wt/trunk
```

Then put the PR number into the `CHANGELOG.md` entry (a commit on the branch), here and in
[../README.md](../README.md).
