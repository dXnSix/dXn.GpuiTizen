#!/usr/bin/env bash
# Runs a command in the `gpuitizen` Lima VM (design `d19-lima-desktop-path`), in the caller's
# directory (this repository, or a sibling such as ../wgpu), under Xvfb, on the GL ES 3.0 path: WGPU_BACKEND=gl, MESA_GLES_VERSION_OVERRIDE=3.0 and no
# storage buffers (MESA_EXTENSION_OVERRIDE), a VM-local
# CARGO_TARGET_DIR, and, for Cargo commands, the local gpui-fast patch
# (artifacts-temp/gpui-fast-patch.toml, written by `cargo xtask gpui-fast-release --verify`) with
# its own lockfile, so the repository's Cargo.lock never records the path patch.
#
# Usage: scripts/lima-run.sh cargo test -p example_counter --test gles_smoke
#        LIMA_RUN_TIMEOUT=600 scripts/lima-run.sh ...   (the VM-side time limit, default 5400 s)
#        scripts/lima-run.sh --weston cargo test -p dxn_gpui_tizen --test egl_wayland
#          (under `weston --backend=headless` instead of Xvfb, WAYLAND_DISPLAY set)
#        (cd ../wgpu && ../dXn.GpuiTizen/scripts/lima-run.sh cargo test -p wgpu-hal --features gles --lib)
#        scripts/lima-run.sh dbus-run-session -- cargo test -p dxn_gpui_tizen --test a11y_atspi
set -euo pipefail

VM=gpuitizen
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

weston=0
if [ "${1:-}" = "--weston" ]; then
  weston=1
  shift
fi
if [ "$#" -eq 0 ]; then
  echo "usage: scripts/lima-run.sh [--weston] <command...>" >&2
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
WORKDIR="$(pwd -P)"
REPO_REAL="$(cd "$REPO" && pwd -P)"
# This repository's workspace: its root or any directory inside it.
in_repo=0
case "$WORKDIR/" in "$REPO_REAL"/*) in_repo=1 ;; esac
# A target directory per workspace inside the VM (this repository keeps the plain one), and the
# GL ES 3.0 situation (below) for this repository only: wgpu's own GPU tests run on plain Mesa.
target_dir='$HOME/target-gpuitizen'
gles_path=1
if [ "$in_repo" = 0 ]; then
  target_dir="\$HOME/target-gpuitizen-$(basename "$WORKDIR")"
  gles_path=0
fi
# The command's time limit in seconds (read here, on the host).
limit="${LIMA_RUN_TIMEOUT:-5400}"
case "$limit" in ''|*[!0-9]*) echo "LIMA_RUN_TIMEOUT must be a number of seconds" >&2; exit 2 ;; esac
# The gpui-fast patch belongs to this repository's workspace only.
if [ "$in_repo" = 1 ] && [ "$cargo_at" -ge 0 ] \
  && [ -f "$REPO/artifacts-temp/gpui-fast-patch.toml" ]; then
  lock="$REPO/artifacts-temp/locks/lima/Cargo.lock"
  mkdir -p "$(dirname "$lock")"
  cp "$REPO/Cargo.lock" "$lock"
  # After the subcommand: an external one (`clippy`) runs `cargo check` again and would not see
  # options given before it.
  sub=$((cargo_at + 1))
  args=("${args[@]:0:$sub}" "${args[$sub]}" --config "$REPO/artifacts-temp/gpui-fast-patch.toml" \
    --config "resolver.lockfile-path='$lock'" "${args[@]:$((sub + 1))}")
fi

quoted="$(printf '%q ' "${args[@]}")"
exec limactl shell --workdir "$WORKDIR" "$VM" -- bash -lc "
  set -euo pipefail
  export PATH=\"\$HOME/.cargo/bin:\$PATH\"
  export CARGO_TARGET_DIR=\"$target_dir\"
  # wgpu 30 opens a desktop GL context through EGL here, which MESA_GLES_VERSION_OVERRIDE does
  # not reach; hiding GL_ARB_shader_storage_buffer_object makes Mesa report GL below 4.3 without
  # storage buffers, the GL ES 3.0 TVs' and the emulator's situation (gpui-fast change 1's path).
  export WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1
  if [ $gles_path = 1 ]; then
    export MESA_GLES_VERSION_OVERRIDE=3.0 MESA_EXTENSION_OVERRIDE=-GL_ARB_shader_storage_buffer_object
  fi
  export RUST_LOG=\"\${RUST_LOG:-info}\"
  # 8 GiB is not enough for GNU ld on GPUI's debug binaries: link with lld, with line tables only.
  export RUSTFLAGS=\"\${RUSTFLAGS:-} -C link-arg=-fuse-ld=lld\" CARGO_PROFILE_DEV_DEBUG=line-tables-only
  # A hard limit, so a stalled app or test never keeps spinning in the VM after the host gave up.
  if [ $weston = 1 ]; then
    # Weston's headless backend on a private runtime directory, for Wayland EGL clients.
    export XDG_RUNTIME_DIR=\"\$(mktemp -d)\"
    chmod 700 \"\$XDG_RUNTIME_DIR\"
    weston --backend=headless --socket=wayland-dxn --idle-time=0 >\"\$XDG_RUNTIME_DIR/weston.log\" 2>&1 &
    weston_pid=\$!
    trap 'kill \$weston_pid 2>/dev/null; wait \$weston_pid 2>/dev/null; rm -rf \"\$XDG_RUNTIME_DIR\"' EXIT
    for _ in \$(seq 50); do [ -S \"\$XDG_RUNTIME_DIR/wayland-dxn\" ] && break; sleep 0.1; done
    [ -S \"\$XDG_RUNTIME_DIR/wayland-dxn\" ] || { cat \"\$XDG_RUNTIME_DIR/weston.log\" >&2; exit 1; }
    export WAYLAND_DISPLAY=wayland-dxn
    unset DISPLAY
    timeout --kill-after=30 $limit $quoted
  else
    timeout --kill-after=30 $limit xvfb-run -a -s '-screen 0 1920x1080x24' $quoted
  fi
"
