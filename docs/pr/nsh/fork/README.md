# fork: the fork's own Tizen CI

One branch that never goes upstream: `fork/tizen-ci` adds `.github/workflows/fork-tizen.yml`, the fork's checks of the
Tizen targets, beside the fork's `fork-ci.yml` and `fork-release.yml`. It is a fork PR to `dXnSix/nsh` `main`, merged
with a merge commit as every fork PR is, and stays on the fork's list of fork-only differences (nsh-root's
`upstream-pr/README.md`, "The fork's `main`"; add a row there when it merges, which this epic does not write).

| Folder | Branch | Built on | Title | Size | Needs | Status |
|---|---|---|---|---|---|---|
| [tizen-ci/](tizen-ci/) | `fork/tizen-ci` | `upstream/main` `8c733731f` (which the fork's `main` holds) | ci: Check the Tizen targets in a workflow of their own | 1 file, +241 | the `pr/tizen/*`, `pr/tizen-web/*` and `pr/tv/03-dev-idiom` branches in the fork's `main`; dXn.GpuiTizen, `dXnSix/gpui-fast` `tizen-release` and `dXnSix/wgpu` `v30-tizen` pushed | not opened |

Upstream gets no Tizen job from this; whether it wants one is question 6 of [../tizen/issue.md](../tizen/issue.md).
