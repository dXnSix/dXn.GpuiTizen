# Upstream issues and pull requests

What this repository's work needs from other projects, prepared here and sent by the owner: issues and PRs to
[gpui-fast](https://github.com/longbridge/gpui-fast), [wgpu](https://github.com/gfx-rs/wgpu),
[NSH](https://github.com/longbridge/nsh) and [nsh-compat](https://github.com/longbridge/nsh-compat), from the forks
`dXnSix/*`. One folder per PR branch, in the format of nsh-root's `upstream-pr/`
(`/Users/dinko/work-private/projects/dXn/nsh/nsh-root/upstream-pr/README.md`, read-only for this repository):

```
docs/pr/<project>/[<area>/]<topic>/   ↔  the branch of that PR on the fork
    README.md                          status, base, commits, checks, how to update and open it
    PR.md                              the title (line 1) and the description (from line 3) to paste
docs/pr/<project>/[<area>/]README.md   the order, the stack, what is shared
docs/pr/<project>/[<area>/]issue*.md   the issues that come before the PRs
```

## Projects

| Project | Index | What | Status (2026-11-10, 19:30 CET) |
|---|---|---|---|
| gpui-fast | [gpui-fast/README.md](gpui-fast/README.md) | six renderer changes for GL ES 3.0 TVs (one issue, six PRs) | nothing opened; issues are closed on `longbridge/gpui-fast` |
| wgpu | [wgpu/README.md](wgpu/README.md) | EGL 1.4 on Wayland, GL program binaries (two PRs) | nothing opened |
| NSH | [nsh/README.md](nsh/README.md) | focus, TV keys and category, one wgpu, `Os::Tizen` and the `tizen` / `tizen-web` targets (three issues and a fourth on the desktop defaults, 22 PRs) and one fork-only CI branch | nothing opened or pushed; every branch's sprint checked, reviewed (s14–s21 all passed) and checkpointed |
| nsh-compat | [nsh/compat/README.md](nsh/compat/README.md) | three companions of the NSH PRs | nothing opened; no `dXnSix/nsh-compat` fork yet |

### NSH

| Area | Branches | Issue | Needs first |
|---|---|---|---|
| [focus](nsh/focus/README.md) | `pr/focus/01-core` … `07-web-tv` | [issue.md](nsh/focus/issue.md), [issue-desktop-tab.md](nsh/focus/issue-desktop-tab.md) | nsh-compat `focus-props` (for 04) |
| [tv](nsh/tv/README.md) | `pr/tv/01-keys` … `04-remote-keys` | [issue.md](nsh/tv/issue.md) | nsh-compat `tv-idiom` (for 02) |
| [engine](nsh/engine/README.md) | `pr/engine/01-wgpu-30` | – | – |
| [tizen](nsh/tizen/README.md) | `pr/tizen/01-os` … `07-cli` | [issue.md](nsh/tizen/issue.md) | nsh-compat `tizen-platform` (for 01); gpui-fast released, dXn.GpuiTizen on crates.io (for 04 on) |
| [tizen-web](nsh/tizen-web/README.md) | `pr/tizen-web/01-package` … `03-module` | the `tizen` issue | the TV loader probe (`DEV-6`) |
| [compat](nsh/compat/README.md) | nsh-compat `tv-idiom`, `focus-props`, `tizen-platform` | – | – |
| [fork](nsh/fork/README.md) | `fork/tizen-ci` (fork PR to `dXnSix/nsh` `main`) | – | the `tizen` and `tizen-web` branches and `pr/tv/03-dev-idiom` in the fork's `main` |

## The order of opening

Each step on the owner's go at that moment (the epic's `UP-4`, design `d33-upstream`). A step starts when the one
before it is answered or merged as noted.

1. **NSH focus.** Post [nsh/focus/issue.md](nsh/focus/issue.md) and
   [nsh/focus/issue-desktop-tab.md](nsh/focus/issue-desktop-tab.md). Open `pr/focus/01-core`; once merged, `02-gpui`,
   `03-desktop-stops` (after the desktop issue is answered) and `05-web`; nsh-compat `focus-props`, then `04-tv`; then
   `06-gpui-tab-stops` and `07-web-tv`.
2. **TV keys and category.** Post [nsh/tv/issue.md](nsh/tv/issue.md). Open nsh-compat `tv-idiom`; `pr/tv/01-keys` and
   `pr/tv/02-category`; then `03-dev-idiom` (the TV idiom for `nsh run`, `nsh dev` and `nsh test`) and
   `04-remote-keys`. (Independent of step 1: they may overlap.)
3. **One wgpu.** `pr/engine/01-wgpu-30` (needs nothing; may go earlier).
4. **gpui-fast.** The issue ([gpui-fast/issue.md](gpui-fast/issue.md), posted where a maintainer says, since issues
   are closed there), then its PRs in [gpui-fast/README.md](gpui-fast/README.md)'s order.
5. **wgpu.** Its two PRs ([wgpu/README.md](wgpu/README.md)), either first.
6. **crates.io.** Once a gpui-fast release has the changes: dXn.GpuiTizen moves to it and publishes `dxn_gpui_tizen_sys`
   and `dxn_gpui_tizen` (the epic's `PUB-2`).
7. **NSH `Os::Tizen` and the Tizen targets.** Post [nsh/tizen/issue.md](nsh/tizen/issue.md) (it asks whether GPUI may be
   the native backend of packaged apps). Open nsh-compat `tizen-platform`; `pr/tizen/01-os`, `02-gpui-platform-optional`,
   `03-gpui-tizen-cfg` (these three need nothing of steps 4–6 and may go earlier, the owner's call); then
   `04-platform`, `05-video` and `06-module`.
8. **`tizen-web`, after the TV loader probe** (`DEV-6`: `nsh package --target tizen-web --probe` on a TV). Open
   `pr/tizen-web/01-package`; then `pr/tizen/07-cli` (it needs 01-package's `nsh.json` section); then
   `pr/tizen-web/02-scrollend` (needs nothing; may go earlier) and `03-module`.
9. **The fork.** `fork/tizen-ci` as a fork PR once the `tizen` and `tizen-web` branches and `pr/tv/03-dev-idiom`
   (`nsh test --idiom`, `tests/apps/tv-idiom`, which its `tv-specs` job runs) are in the fork's `main`.

## How the folders are kept

- **One folder per branch, named as the branch** (`pr/focus/01-core` ↔ `nsh/focus/01-core/`; gpui-fast and wgpu
  branches have no area). A stack has its order and state in its area `README.md`.
- **The branch `README.md`** holds the status line (not opened / opened #n / merged #n with date and commit), the
  base and how far `upstream` is past it, what it depends on, its size and commits, the checks with the commands and
  what was recorded where, and the commands to update and open it, marked "owner runs this". Update it whenever the
  branch moves.
- **`PR.md`** is the text pasted into the PR: line 1 `# <title>`, line 2 empty, the body from line 3 (`sed '1,2d'
  PR.md`). NSH texts follow NSH's pull request template (Description, Public API, Breaking Changes, How to Test,
  Checklist); wgpu's follow wgpu's; gpui-fast and nsh-compat have none, so theirs follow NSH's shape. A text that
  needs an nsh-compat PR names it at the top. When a PR changes under review, change `PR.md` and the PR together.
- **Placeholders**: `#(…)` is a PR or issue number to fill in once it exists; `[OWNER: …]` is something only the owner
  can supply (device numbers, probe results, published versions, a link). Replace them before opening.
- **Before opening**: fetch, rebase onto the latest upstream branch (keeping the old head under an `epic/…` tag for
  the epic's records, [nsh/README.md](nsh/README.md#updating-a-branch-and-opening-it)), run the branch's checks, update
  the numbers and the checklist in `PR.md`.
- **After opening**: put the number in the branch `README.md`, the area `README.md`, this index, and the `PR.md` of
  the PRs that depend on it. After merging: the date and the merge commit; the folder stays as the record.
- Nothing is pushed, posted or opened without the owner's go; no AI attribution in commits, PR texts or comments.
  Logs and raw output go to `artifacts-temp/`, never here.
- nsh-root's `upstream-pr/` keeps the fork's other upstream work; this repository never writes there. When an NSH
  branch here merges into the fork's `main`, the fork-only table in nsh-root's `upstream-pr/README.md` is the owner's
  to update.
