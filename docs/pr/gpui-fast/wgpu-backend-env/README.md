# wgpu-backend-env

| | |
|---|---|
| Change | 2, GL on request |
| Upstream PR | ready, not opened: nothing is opened at `longbridge/gpui-fast` before the owner's agreement with its owners ([../../README.md](../../README.md#prepared-not-opened-2026-11-12)) |
| Branch | `wgpu-backend-env` on `dXnSix/gpui-fast`, head `dd0d50884`, pushed 2026-11-12 (rebased from `54a64d3`) |
| Base | `upstream/main` `d1d6286` (#139, fetched 2026-11-12); rebased onto it without conflicts in its own files |
| Size | 3 files, +66 −1: `crates/gpui_wgpu/src/fast/gles.rs` (new, with its tests), `fast/mod.rs` (+1), `wgpu_context.rs` (+1 −1, one hook) |
| Its commits | `dd0d508` fast: let WGPU_BACKEND choose the native backends |
| Text | [PR.md](PR.md) |
| On `tizen` | `fee7e2c` |
| Needs | the issue ([../issue.md](../issue.md)); no new API. If `gles-instance-path` merges first, a rebase: both create `fast/gles.rs` and add `pub(crate) mod gles;` |

`WgpuContext::instance()` asks for `wgpu::Backends::from_env()` when `WGPU_BACKEND` is set, and for `VULKAN | GL` as
before when it is not. A desktop or CI machine with Vulkan (lavapipe included) can then run the GL ES path the TVs
take; that is how this repository's CI tests change 1 under Xvfb.

Upstream PR #120 (trevortomlin, closed 2026-11-08 without comment) changed the default instead: a Vulkan-only instance
first, GL only when it finds no adapter, `WGPU_BACKEND` overriding. This branch keeps the default and only honours the
variable.

### Rebasing after `gles-instance-path`

`fast/gles.rs` then exists on `main` with its `tests::device` group. Keep that file and add this branch's part to it:
the module comment's paragraph, `DEFAULT_BACKENDS`, `backends()`, and the `tests::backends` group next to
`tests::device` (as on `tizen`); drop the second `pub(crate) mod gles;` line. The PR's size then drops to 2 files.

## Checks

To run on the branch before opening:

```bash
cargo test -p gpui_wgpu --features test-support --lib fast::gles::tests::backends   # 3 tests
script/check-upstream
cargo fmt -p gpui_wgpu -- --check
cargo clippy -p gpui_wgpu --lib --features test-support -- -D warnings
cargo test -p gpui --features test-support
```

The tests set and restore `WGPU_BACKEND` under a lock; run them as a group, as above.

What has run (dXn.GpuiTizen's epic checks, in `../gpui-fast` on `tizen`, macOS; nothing recorded on this branch
itself):

- 2026-11-08 23:07 UTC: `fast::gles::tests::backends` passed (3 tests); `script/check-upstream` and the `gpui_wgpu`
  clippy passed.
- 2026-11-09 00:38 and 02:08 UTC: the examples' `gles_smoke` tests in the Linux VM ran with `WGPU_BACKEND=gl` and
  drew through GL, against the release workspace generated from `tizen`.
- Not run: the upstream CI steps on Linux and Windows.


### Run on the rebased tip (2026-11-12)

On `dd0d50884`, macOS (`../gpui-fast-wt/pr`): `fast::gles::tests::backends` 3 passed, `cargo test -p gpui --features test-support` 842, `script/check-upstream`, `cargo fmt`, the `gpui_wgpu` clippy. Logs: `artifacts-temp/agentx/tizen-gpui-platform/t25-gf/wgpu-backend-env.log`.

## Updating and opening

Rebase onto the fresh `upstream/main` (see above if `gles-instance-path` merged first), run the checks, tick the
checklist in `PR.md`, replace `#(issue)`. The owner runs this:

```bash
cd /Users/dinko/work-private/projects/dXn/gpui-fast
git fetch upstream origin
git worktree add ../gpui-fast-wt/pr wgpu-backend-env 2>/dev/null || git -C ../gpui-fast-wt/pr switch wgpu-backend-env
cd ../gpui-fast-wt/pr
git rebase upstream/main
# the checks above, then
git push --force-with-lease -u origin wgpu-backend-env
gh pr create --repo longbridge/gpui-fast --base main --head dXnSix:wgpu-backend-env \
  --title "$(head -1 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/wgpu-backend-env/PR.md | sed 's/^# //')" \
  --body-file <(tail -n +3 /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/gpui-fast/wgpu-backend-env/PR.md)
```

Then record the PR number here and in [../README.md](../README.md).
