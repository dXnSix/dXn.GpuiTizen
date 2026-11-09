#!/usr/bin/env bash
# Runs a command in the `gpuitizen` Lima VM (design `d19-lima-desktop-path`), in this repository's
# path, under Xvfb, on the GL ES 3.0 path: WGPU_BACKEND=gl, MESA_GLES_VERSION_OVERRIDE=3.0 and no
# storage buffers (MESA_EXTENSION_OVERRIDE), a VM-local
# CARGO_TARGET_DIR, and, for Cargo commands, the local gpui-fast patch
# (artifacts-temp/gpui-fast-patch.toml, written by `cargo xtask gpui-fast-release --verify`) with
# its own lockfile, so the repository's Cargo.lock never records the path patch.
#
# Usage: scripts/lima-run.sh cargo test -p example_counter --test gles_smoke
#        scripts/lima-run.sh dbus-run-session -- cargo test -p dxn_gpui_tizen --test a11y_atspi
set -euo pipefail

VM=gpuitizen
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$#" -eq 0 ]; then
  echo "usage: scripts/lima-run.sh <command...>" >&2
  exit 2
fi

status="$(limactl list --format '{{.Status}}' "$VM" 2>/dev/null || true)"
if [ -z "$status" ]; then
  echo "the $VM VM does not exist; create it with: limactl create --name $VM scripts/lima/gpuitizen.yaml" >&2
  exit 1
fi
if [ "$status" != "Running" ]; then
  limactl start "$VM" >&2
fi

args=("$@")
# The first `cargo` argument, also behind a wrapper such as `dbus-run-session --`.
cargo_at=-1
for i in "${!args[@]}"; do
  if [ "${args[$i]}" = "cargo" ]; then cargo_at=$i; break; fi
done
if [ "$cargo_at" -ge 0 ] && [ -f "$REPO/artifacts-temp/gpui-fast-patch.toml" ]; then
  lock="$REPO/artifacts-temp/locks/lima/Cargo.lock"
  mkdir -p "$(dirname "$lock")"
  cp "$REPO/Cargo.lock" "$lock"
  args=("${args[@]:0:$cargo_at}" cargo --config "$REPO/artifacts-temp/gpui-fast-patch.toml" \
    --config "resolver.lockfile-path='$lock'" "${args[@]:$((cargo_at + 1))}")
fi

quoted="$(printf '%q ' "${args[@]}")"
exec limactl shell --workdir "$REPO" "$VM" -- bash -lc "
  set -euo pipefail
  export PATH=\"\$HOME/.cargo/bin:\$PATH\"
  export CARGO_TARGET_DIR=\"\$HOME/target-gpuitizen\"
  # wgpu 30 opens a desktop GL context through EGL here, which MESA_GLES_VERSION_OVERRIDE does
  # not reach; hiding GL_ARB_shader_storage_buffer_object makes Mesa report GL below 4.3 without
  # storage buffers, the GL ES 3.0 TVs' and the emulator's situation (gpui-fast change 1's path).
  export WGPU_BACKEND=gl MESA_GLES_VERSION_OVERRIDE=3.0 LIBGL_ALWAYS_SOFTWARE=1
  export MESA_EXTENSION_OVERRIDE=-GL_ARB_shader_storage_buffer_object
  export RUST_LOG=\"\${RUST_LOG:-info}\"
  # 8 GiB is not enough for GNU ld on GPUI's debug binaries: link with lld, with line tables only.
  export RUSTFLAGS=\"\${RUSTFLAGS:-} -C link-arg=-fuse-ld=lld\" CARGO_PROFILE_DEV_DEBUG=line-tables-only
  xvfb-run -a -s '-screen 0 1920x1080x24' $quoted
"
