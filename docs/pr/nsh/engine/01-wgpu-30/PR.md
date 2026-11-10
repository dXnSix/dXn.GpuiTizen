# engine: Move the shader renderer to wgpu 30

No issue (no API or behaviour change). No nsh-compat change.

## Description

`nsh-engine`'s shader renderer (`shader` feature) is on wgpu 29, while gpui-fast 0.1.13, which the GPUI backend draws
with, is on wgpu 30. So the GPUI workspace builds two wgpu stacks (`crates/gpui/Cargo.lock` holds wgpu 29.0.4 and
30.0.1, with two naga and two sets of hal backends), and shader views on GPUI run on a GPU stack of their own.

wgpu and naga move to 30 in the main workspace, so the GPUI workspace's lockfile holds one wgpu and one naga, and
shader views share GPUI's stack. This matters most where the build is small and the GPU is shared: our case is a
Samsung TV (32-bit ARM, GL ES), where a second wgpu is a second GL device in the same process and several MB of code.

The API changes the renderer meets in wgpu 30:

- `BufferSlice::get_mapped_range` returns a `Result`: a range that cannot be read leaves the frame shown.
- `hal::metal::Device::texture_from_raw` takes a drop callback: none, the texture owns the Metal texture.
- `Device::create_texture_from_hal` takes the texture's initial state: uninitialized.

Both lockfiles change only in the wgpu stack (`crates/gpui/Cargo.lock`: −290 lines).

## Public API

None. `nsh-engine` exposes no wgpu types.

## How to Test

- `cargo test -p nsh-engine --features shader --test engine shader::` on macOS (Metal), Windows (DX12) and Linux
  (Vulkan, GL).
- `cargo tree --manifest-path crates/gpui/Cargo.toml -i wgpu`: one version, 30.
- The gallery's shader demos on iOS, Android and the desktops.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): nothing apps use changes.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (the shader demos).
