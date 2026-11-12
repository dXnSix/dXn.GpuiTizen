# pr/engine/01-wgpu-30

| | |
|---|---|
| Upstream PR | ready, not opened: nothing goes to `longbridge/nsh` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Fork PR | dXnSix/nsh#194 (merge commit), into the fork's `main`, on the owner's go |
| Upstream branch | `up/engine/01-wgpu-30` on `dXnSix/nsh`, head `b09109824`, pushed 2026-11-12: `upstream/main` `ca7e70e51` (#1789); see below |
| Branch | `pr/engine/01-wgpu-30`, head `176a2bd27`, pushed to `dXnSix/nsh` |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it merges without conflict (`git merge-tree`), lockfiles included. Not rebased |
| Fork's `main` | `7f5084268` merged on 2026-11-11, by merges (no rebase), from `c5a018805`: `176a2bd27` ([the merge](../../README.md#the-forks-main-merged-in-2026-11-11)) |
| Depends on | nothing |
| Size | 6 files, +96 −322: `Cargo.toml` (`wgpu` and `naga` 29 → 30), `Cargo.lock`, `crates/gpui/Cargo.toml` (a comment), `crates/gpui/Cargo.lock` (−290: one wgpu stack instead of two), `crates/nsh-engine/src/shader.rs`, `shader_surfaces.rs` |
| Its commits | `c5a018805` engine: Move the shader renderer to wgpu 30 |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `977124147`) |
| Review findings | none: s16's review (`review-s16-2.json`, passed) judged the wgpu 30 adaptations correct, with one wgpu and one naga in both lockfiles |
| Needs | no issue, no nsh-compat change |

`upstream/main`'s `crates/gpui/Cargo.lock` holds wgpu 29.0.4 (the engine's) and 30.0.1 (gpui-fast 0.1.13's) today.


## The upstream branch (2026-11-12)

`up/engine/01-wgpu-30` holds this branch's own commits on `upstream/main` `ca7e70e51`, cherry-picked (the `pr/*` branch, which carries
merges of the fork's `main`, stays as it is): `0e6541994` engine: Move the shader renderer to wgpu 30; `b09109824` engine: Lock gpu-allocator on the windows crate wgpu-hal 30 takes.

Cherry-picked cleanly (the lockfiles through `rerere`; `cargo metadata --locked` passes for the root workspace, and `crates/gpui/Cargo.lock` lacks only `url`, as `upstream/main`'s own does).

Checks on its tip `b09109824` (logs under `artifacts-temp/agentx/tizen-gpui-platform/t25-nsh/`): `nsh-engine --test engine shader::` 5 passed; `tizen-deps --single wgpu` (one wgpu, 30.0.1, in this branch's `crates/gpui/Cargo.lock`); `cargo test -p nsh-engine --features shader` 393 passed (a first run under load failed `components::the_topmost_page_or_dialog_decides_the_status_bar` once, which passed five times alone and in the rerun); fmt; the `nsh-engine` and GPUI clippy.

## Before opening

- Upstream changed the shader engine since the base: #1601 (`c16a70096`), #1476, #1442 (`7626758b4`, macOS shader
  views without the display link). The merge is clean; rebuild and rerun the shader tests on macOS (Metal:
  `texture_from_raw`'s new drop callback), and if possible on Windows (DX12) and Linux (Vulkan / GL).
- After the rebase, check both lockfiles still change only in the wgpu stack (`cargo tree -i wgpu` in both
  workspaces shows 30 only).

## Checks

| Check | Command | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-shader-wgpu30` | `cargo test -p nsh-engine --features shader --test engine shader::` (in the branch's checkout) | `t16-wgpu30` | passed, 5 tests, 2026-11-10 11:26 UTC (s16) |
| `chk-nsh-one-wgpu` | `cargo xtask tizen-deps --manifest-path ../nsh/nsh-wt/tizen/crates/gpui/Cargo.toml --single wgpu` (from `/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen`; expects "one wgpu") | `t16-wgpu30` | passed, 11:26 UTC (s16) |

**On this branch alone** (no result above is from it): `chk-nsh-shader-wgpu30` runs as written in a worktree that has
this branch; `chk-nsh-one-wgpu`'s `--manifest-path` names `nsh-wt/tizen`, which is `feat/tizen`: point it at that
worktree's `crates/gpui/Cargo.toml`. Not run: the pre-open checks, the shader tests on Windows and Linux, the gallery's
shader demos.

## Updating and opening

Nothing is opened before the owner's agreement with NSH's owners ([../../../README.md](../../../README.md#prepared-not-opened-2026-11-12)).
The branch that opens is `up/engine/01-wgpu-30`, never `pr/engine/01-wgpu-30`, which stays as the epic's records name it.
Before opening, rebuild `up/engine/01-wgpu-30` on the newest `upstream/main` (or its parent's `up/` branch, once that has
merged upstream) as [../../README.md](../../README.md#updating-a-branch-and-opening-it) says, and run the checks
above on its tip. Then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git push --force-with-lease origin up/engine/01-wgpu-30
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/engine/01-wgpu-30
gh pr create --repo longbridge/nsh --base main --head dXnSix:up/engine/01-wgpu-30 \
  --title "engine: Move the shader renderer to wgpu 30" --body "$(sed '1,2d' PR.md)"
```
