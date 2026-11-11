# pr/engine/01-wgpu-30

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/engine/01-wgpu-30`, head `c5a018805`, pushed to `dXnSix/nsh` |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it merges without conflict (`git merge-tree`), lockfiles included. Not rebased |
| Depends on | nothing |
| Size | 6 files, +96 −322: `Cargo.toml` (`wgpu` and `naga` 29 → 30), `Cargo.lock`, `crates/gpui/Cargo.toml` (a comment), `crates/gpui/Cargo.lock` (−290: one wgpu stack instead of two), `crates/nsh-engine/src/shader.rs`, `shader_surfaces.rs` |
| Its commits | `c5a018805` engine: Move the shader renderer to wgpu 30 |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `977124147`) |
| Review findings | none: s16's review (`review-s16-2.json`, passed) judged the wgpu 30 adaptations correct, with one wgpu and one naga in both lockfiles |
| Needs | no issue, no nsh-compat change |

`upstream/main`'s `crates/gpui/Cargo.lock` holds wgpu 29.0.4 (the engine's) and 30.0.1 (gpui-fast 0.1.13's) today.

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

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/engine/01-wgpu-30 pr/engine/01-wgpu-30
git rebase --onto upstream/main 8c733731f pr/engine/01-wgpu-30
# the checks above and the pre-open checks, then (owner runs this)
git push -u origin pr/engine/01-wgpu-30
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/engine/01-wgpu-30
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/engine/01-wgpu-30 \
  --title "engine: Move the shader renderer to wgpu 30" --body "$(sed '1,2d' PR.md)"
```
