# engine: one wgpu in GPUI builds

One PR: `nsh-engine`'s shader renderer moves from wgpu 29 to wgpu 30, the version gpui-fast 0.1.13 draws with, so the
GPUI workspace's lockfile holds one wgpu and one naga, and shader views share GPUI's GPU stack. On a TV that means one
GL ES device and one copy of wgpu in a 32-bit library with a memory budget; on the desktops a smaller GPUI build.

| # | Folder | Branch | Built on | Title | Size | Needs | Status |
|---|---|---|---|---|---|---|---|
| 01 | [01-wgpu-30/](01-wgpu-30/) | `pr/engine/01-wgpu-30` | `upstream/main` `8c733731f` | engine: Move the shader renderer to wgpu 30 | 6 files, +96 −322 | – | ready, not opened: `up/engine/01-wgpu-30` |

It needs no issue (no API, no behaviour change) and no other PR. In the order of [../../README.md](../../README.md) it
comes after the focus and TV PRs and before gpui-fast's, because the TV build needs it and nothing else waits on it;
the owner may open it earlier.

`git merge-tree` against `upstream/main` `c16a70096`: clean. Upstream's #1601 (`c16a70096`, "Open shaders on the next
backend when one refuses its device") and #1476 (`shaderSupport`) touched the shader engine since; rebuild and rerun
the shader tests after the rebase.
