#!/usr/bin/env bash
# Clippy for a target this machine cannot build C for (Windows MSVC from macOS, a GTK Linux target
# without its development packages): C compilers, archivers and pkg-config are replaced by
# stand-ins that compile nothing (scripts/check-only/), and bindgen gets -ffreestanding with stub
# libc headers. The Rust code is type-checked and linted; nothing links or runs, so this proves
# only that the code compiles and passes clippy. Run from the workspace to check, for example
#   ../../dXn.GpuiTizen/scripts/check-only-clippy.sh x86_64-pc-windows-msvc -p nsh-platform-windows -- -D warnings
# The objects are empty, so it uses its own target directory (CHECK_ONLY_TARGET_DIR, default under
# $TMPDIR).
set -euo pipefail
if [ "$#" -lt 1 ]; then
  echo "usage: scripts/check-only-clippy.sh <target triple> [cargo clippy arguments...]" >&2
  exit 2
fi
target=$1
shift
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/check-only"
triple=${target//-/_}
export CARGO_TARGET_DIR="${CHECK_ONLY_TARGET_DIR:-${TMPDIR:-/tmp}/check-only-clippy}"
export "CC_${triple}=$here/bin/fakecc" "CXX_${triple}=$here/bin/fakecc" "AR_${triple}=$here/bin/fakear"
export "BINDGEN_EXTRA_CLANG_ARGS_${triple}=-ffreestanding -I$here/inc"
export PKG_CONFIG="$here/bin/fakepkgconfig" PKG_CONFIG_ALLOW_CROSS=1
rustup target list --installed | grep -qx "$target" || rustup target add "$target"
exec cargo clippy --target "$target" "$@"
