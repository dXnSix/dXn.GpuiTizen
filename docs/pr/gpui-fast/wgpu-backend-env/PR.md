# fast: let WGPU_BACKEND choose the native backends

## Description

Part of #(issue): GPUI on Samsung TVs (Tizen), change 2.

`WgpuContext::instance()` asks wgpu for `Backends::VULKAN | Backends::GL`, and Vulkan adapters rank first. A desktop
or CI machine with a Vulkan driver (Mesa's lavapipe included) therefore never draws through GL, which is the only API
on GL ES devices such as Samsung TVs. There is no way to ask for GL short of removing the Vulkan driver.

With this change the native instance asks for `wgpu::Backends::from_env()` when `WGPU_BACKEND` is set (wgpu's own
reader: `gl`, `vulkan`, `gl,vulkan`, ...), and for `VULKAN | GL` as before when it is not. `WGPU_BACKEND` is the
variable wgpu documents for this, so nothing new is introduced.

- `crates/gpui_wgpu/src/fast/gles.rs`: `backends()` and its tests.
- `wgpu_context.rs`: `backends: crate::fast::gles::backends()`, one line.

Behaviour without `WGPU_BACKEND` is unchanged. #120 proposed a different default (GL only when Vulkan finds no
adapter); this change leaves the default as it is and only honours the variable. No public API.

## How to Test

- `cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::backends`: unset keeps `VULKAN | GL`,
  `gl` selects only GL, `vulkan` only Vulkan (the tests set and restore the variable under a lock).
- On Linux with a Vulkan driver: `WGPU_BACKEND=gl RUST_LOG=info cargo run -p gpui --example hello_world` logs a GL
  adapter (`Selected GPU adapter: ... (Gl)`); without the variable it logs the Vulkan one.
- `script/check-upstream`, `cargo fmt -p gpui_wgpu -- --check`,
  `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`.

## Checklist

- [ ] `script/check-upstream` passes: one hook in `wgpu_context.rs`, the logic in `fast/gles.rs`.
- [ ] `cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::backends` passes.
- [ ] `cargo test -p gpui --features test-support` passes.
- [ ] `cargo fmt -p gpui_wgpu -- --check` and `cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings`
  pass.
- [ ] Checked on Linux with a Vulkan driver that `WGPU_BACKEND=gl` selects a GL adapter.
- [x] No new public API; unchanged behaviour without `WGPU_BACKEND`.
