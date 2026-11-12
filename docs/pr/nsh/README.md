# NSH: Samsung TVs (Tizen) and directional focus

What the epic `tizen-gpui-platform` changes in [NSH](https://github.com/longbridge/nsh), prepared as upstream pull
requests to `longbridge/nsh` from the fork `dXnSix/nsh`, with their nsh-compat companions and one fork-only branch.
One folder per branch, named as the branch:

```
docs/pr/nsh/<area>/<topic>/      ↔  branch pr/<area>/<topic> (fork/<topic> for the fork-only one)
    README.md                     status, base, commits, checks, how to update and open it
    PR.md                         the title (line 1) and the description (from line 3) to paste
docs/pr/nsh/<area>/README.md      the area's order and its stack
docs/pr/nsh/<area>/issue*.md      the issues that come before the area's PRs
```

## Areas

| Area | Index | Branches | Issue first |
|---|---|---|---|
| `focus` | [focus/README.md](focus/README.md) | `pr/focus/01-core` … `07-web-tv` (seven) | [focus/issue.md](focus/issue.md), [focus/issue-desktop-tab.md](focus/issue-desktop-tab.md) |
| `tv` | [tv/README.md](tv/README.md) | `pr/tv/01-keys` … `04-remote-keys` (four) | [tv/issue.md](tv/issue.md) |
| `compat` | [compat/README.md](compat/README.md) | nsh-compat `tv-idiom`, `focus-props`, `tizen-platform` (to `longbridge/nsh-compat`) | – |
| `engine` | [engine/README.md](engine/README.md) | `pr/engine/01-wgpu-30` | – |
| `tizen` | [tizen/README.md](tizen/README.md) | `pr/tizen/01-os` … `07-cli` (seven) | [tizen/issue.md](tizen/issue.md) |
| `tizen-web` | [tizen-web/README.md](tizen-web/README.md) | `pr/tizen-web/01-package` … `03-module` (three) | the `tizen` issue |
| `fork` | [fork/README.md](fork/README.md) | `fork/tizen-ci` (a fork PR to `dXnSix/nsh` `main`, never upstream) | – |

Every branch is **pushed to `dXnSix/nsh`** (2026-11-10; again 2026-11-11 with the fork's `main` merged in), and none
is opened upstream. The branch READMEs list the heads on `origin` as of 2026-11-11. Every stack top is merged into
the fork's `main` through its fork PR, dXnSix/nsh#194 to #203 (2026-11-11 and 2026-11-12,
[below](#merged-into-the-forks-main-2026-11-12)), so the fork's `main` holds the whole epic's NSH work. `feat/tizen`
ran the fork's Tizen CI in dXnSix/nsh#191; all five of its Tizen jobs passed at `5e51c6411`, the emulator boot-only
([fork/tizen-ci/README.md](fork/tizen-ci/README.md#runs-on-the-fork-ci-3-nsh-part)), and again at `2d011802b`, whose
tree is the fork's `main` after #196; #191 merged last (`8b680f8a5`, 2026-11-12).

## The fork's `main` merged in (2026-11-11)

`DEV-1` asks for the branches to be merged into the fork's `main` through fork PRs. The fork's `main` (`7f5084268`,
after the upstream sync #192) was 881 commits past `8c733731f`, and every branch but three conflicted with it. On the
owner's go, the stacks took it by merges, never a rebase:

1. The roots merged `origin/main`: `pr/tv/01-keys`, `pr/tv/02-category`, `pr/engine/01-wgpu-30`, `pr/focus/01-core`,
   `pr/tizen-web/02-scrollend`, `pr/tizen/02-gpui-platform-optional`, `fork/tizen-ci`.
2. Every other branch merged its parents' new heads, in stack order (each README's "Fork's `main`" row lists its
   merges), with `rerere` carrying a resolution to the branches above.
3. `feat/tizen` took a fresh merge of the ten stack tops (`b0ecdd09f`, its tree that merge's), then the fixes below.

Besides keeping both sides, the merges decided:

| Where | What main brought | What the merge does |
|---|---|---|
| `pr/tv/02-category` | `shaders` in the web environment; new prelude names | the branch's `environment_data` sends `shaders`; main's prelude list with `is_tv` |
| `pr/tv/03-dev-idiom` | `nsh test --device` (an iOS simulator) | `starter` takes `device` and `idiom`; main's two new tests pass `idiom: None`; `cc0685a75` allows the starter its eight options |
| `pr/tv/04-remote-keys` | `keyboard.press` chooses application menu items by their shortcut text | `72b2135f0`: the shared `press_stroke` takes the key's text (a remote key's is its keystroke's) |
| `pr/tizen-web/01-package` | per-platform icons (`nsh.json` `icon` object); `compile_package`; one browser per run (`RUNNING`) | a `tizen` icon key (Samsung TVs, `--target tizen-web` and later `tizen`), in `website/docs/packaging.md` (English, zh-CN); the split compile for `tizen-web` too |
| `pr/focus/02-gpui` | #1545: `key_focus.rs`, `focus()` on GPUI stops | folded into the branch's `focus.rs`: `Command::Focus` on a stop (`focus_command`: the window takes the keyboard, the stop is revealed), a pointer going down outside the focused view takes the focus (`pointer_down`); `key_focus.rs` removed; #1545's test reads `crate::focus` |
| `pr/focus/03-desktop-stops` | #1475 RichEdit text inputs; Tab in `input::tab_order` (an overlay's content after its trigger, `tab_after`) | Windows' Tab through the branch's stops in `tab_order` (`21b962ece` keeps a text's links in place); `docs/dev/elements.md` keeps #1519's `FOCUS_VISIBLE` and says macOS and Linux go by tree order |
| `pr/focus/04-tv` | as 02 | the branch's deferred `focus::asked` stays `Command::Focus`'s, now requesting the window's keyboard; `focus_command` dropped |
| `pr/focus/05-web`, `07-web-tv` | `appearance` in the view props; `MOUSE_MODIFIER_KEYS`; `focus-visible:` in the docs | both props written; the docs say `focus-visible:` |
| `pr/focus/06-gpui-tab-stops` | GPUI's Tab in `tab_order`, with a test over fields only | GPUI's stops ordered by `tab_order`; the popover test passes over the trigger, a stop of the branch's Tab |
| `pr/tizen/01-os` | `rustfmt`'s merged imports | `6bb7ee28d` |
| `pr/tizen/04-platform` | a GPUI dev session opened hidden (`held`) | `dev::start(held, transport)` |
| `pr/tizen/07-cli` | `nsh test --device` (an iOS simulator), `nsh package --arch` (macOS) | one `--device` (the simulator with `--target ios`, the TV's `sdb` serial with `tizen`) and one `--arch` (one macOS processor set, or the TPKs' architectures), parsed per target (`package_archs`); `55bd2548b` keeps the device test's name for `chk-nsh-cli-tizen-device` |
| `fork/tizen-ci` | `paths.py` wants every workflow's `pull_request` types | `6e719b0ca` drops `labeled` (a label takes effect with the next push) |
| `feat/tizen` | | `nsh test --target web --idiom` with the TV page (`starter(browser, tv, idiom)`); the Tizen call sites pass no idiom |

nsh-compat's three companions merged its `origin/main` `714e6fd` ([compat/README.md](compat/README.md#the-prs-in-order)).

The epic's 78 NSH checks (`implementation-plan.json`, every `chk-nsh-*` and `fin-nsh-*`) ran on `feat/tizen` with the
committed git patch (an empty `artifacts-temp/gpui-fast-patch.toml` for their `--config`), the failures repaired on
their branches and run again: all pass but `fin-nsh-fmt` and `fin-nsh-gpui-fmt`, which flag
`crates/nsh-platform-macos/src/native.rs`, `crates/nsh-platform-macos/tests/native/main.rs` and
`crates/nsh-platform-windows/src/native/web.rs`, files no branch changes and the fork's `main` has unformatted itself.
Logs: `artifacts-temp/merge-main/checks/`.

The fork PRs, one per stack top, each bringing the branches below it (merge commits, on the owner's go):

| PR | Branch | Brings | Merged |
|---|---|---|---|
| dXnSix/nsh#194 | `pr/engine/01-wgpu-30` | – | merged, `57c000007` |
| dXnSix/nsh#195 | `pr/tizen-web/02-scrollend` | – | merged, `004b31696` |
| dXnSix/nsh#196 | `pr/tv/03-dev-idiom` | `pr/tv/02-category` | merged, `cfd36de9f` (2026-11-12) |
| dXnSix/nsh#197 | `pr/tv/04-remote-keys` | `pr/tv/01-keys` | merged, `be5576011` |
| dXnSix/nsh#198 | `pr/focus/03-desktop-stops` | `pr/focus/01-core` | merged, `8b5396171` |
| dXnSix/nsh#199 | `pr/focus/06-gpui-tab-stops` | `pr/focus/01`, `02`, `04` | merged, `c9e2c3242` (2026-11-12) |
| dXnSix/nsh#200 | `pr/focus/07-web-tv` | `pr/focus/01`, `02`, `04`, `05` | merged, `d7e13e2bd` (2026-11-12) |
| dXnSix/nsh#201 | `pr/tizen/07-cli` | `pr/tv/01`, `02`, `pr/tizen/01` to `06`, `pr/tizen-web/01-package` | merged, `fe3d9d8cd` (2026-11-12) |
| dXnSix/nsh#202 | `pr/tizen-web/03-module` | `pr/tv/01`, `02`, `pr/tizen/01` to `04`, `06`, `pr/tizen-web/01-package` | merged, `7ef5eb28b` (2026-11-12) |
| dXnSix/nsh#203 | `fork/tizen-ci` | – | merged, `9ced98dd0` |
| dXnSix/nsh#205 | `fork/macos-accessibility-frames` | – | merged, `c5add107a` |

Merged on the owner's go (2026-11-11), each once green, #205 first so the others' macOS runs have its wait for
the accessibility frames. The last five first failed `compat` only: the job builds this checkout's `tests/compat/` cases
with `longbridge/nsh-compat`'s harness, which knows neither the `tv` idiom (`platform android tv`), `platform tizen`,
nor the `focusGroup` and `autoFocus` props. Those are the three nsh-compat companions (`tv-idiom`, `focus-props`,
`tizen-platform`), not yet anywhere the job checks out.

With the seven in `main`, `pr/focus/06-gpui-tab-stops` (`456c69373`), `pr/focus/07-web-tv` (`9d5ad246c`) and
`feat/tizen` (`c5412f405`) merged it again: only `docs/dev/elements.md` and `website/{,zh-CN/}docs/web.md` conflicted,
rerere's resolutions kept both sides, and on `pr/focus/07-web-tv` GPUI's Tab stays between fields and native
components (its tree-order Tab is `pr/focus/06`'s).

The owner chose a fork of nsh-compat (2026-11-11): [dXnSix/nsh-compat](https://github.com/dXnSix/nsh-compat), cloned
at `/Users/dinko/work-private/projects/dXn/nsh/nsh-compat` (`origin` the fork, `upstream` longbridge's). Its `main`
is longbridge's `main` (`e56ebce`, #49) with `nsh-tizen` fast-forwarded on, which merges the three companions, each
first merging that `main` (`99ef2fc`); the companions keep their own branches for the upstream PRs. The NSH fork's
`compat.yml` checks out `${{ github.repository_owner }}/nsh-compat` (`fork/compat-fork`, dXnSix/nsh#208), so the
fork runs its own harness and longbridge/nsh its own. The old clone at `dXn/nsh-compat` is no longer used.

## Merged into the fork's `main` (2026-11-12)

On the owner's go the last five merged once `compat` and their own jobs passed, failures that were only runner outages
(the Ubuntu mirror's DNS) or known flakes (iOS gallery timeouts, `compat`'s 2-second `hitSlop` budget, GTK's context
menu) not blocking them. On the way the fork's `main` took these fork-only PRs:

| PR | Branch | What | Merged |
|---|---|---|---|
| dXnSix/nsh#208 | `fork/compat-fork` | `compat.yml` checks out the repository owner's nsh-compat | `f1358ccce` |
| dXnSix/nsh#209 | `sync/upstream-7cd080165` | upstream `main` at `7cd080165` | `6146cd503` |
| dXnSix/nsh#212 | `fork/ci-webkit-for-rust-apps` | WebKitGTK in the fork's web and Android jobs (another session's) | `ca8012e02` |
| dXnSix/nsh#214 | `fork/revert-1687` | reverts upstream #1687 (`nsh-core` and `nsh-base` built once per Rust app), whose `describe` linked a JS engine | `64bdc6b04` |
| dXnSix/nsh#215 | `fork/gpui-test-callers` | GPUI's `tests::windows` waits for the command to bind its keys (`keymap::bound`) | `c314e05d8` |
| dXnSix/nsh#191 | `feat/tizen` | the integration branch, its tree `main`'s: `tv-specs`, `emulator` and the three cross builds pass on it (the rest skip, no paths changed) | `8b680f8a5`, last |

Before each merge the open branches merged the new `main` (never a rebase), `rerere` carrying the earlier resolutions,
each checked against `git merge-tree`'s raw conflict. Besides keeping both sides they decided:

| Where | What main brought | What the merge does |
|---|---|---|
| `pr/tv/03-dev-idiom` | `nsh run`/`nsh dev` `--target tizen-web` and `tizen`; `with_page_host`'s third argument; `DevSession::start`'s fifth | the CLI docs' target lists are main's plus `[--idiom tv\|desktop]`; the new call sites pass `None`, and the Tizen test's two `starter` calls an eighth `None` |
| `pr/tizen/07-cli`, `feat/tizen` | the `tizen` row of `docs/dev/project-structure.md` | the branch's wording (`--target tizen` and `tizen-web`) |
| `pr/focus/07-web-tv` | `docs/dev/elements.md`'s GPUI line | main's: GPUI's Tab stops at every stop in tree order too |

Two failures were real, both outside gpui-fast:

- `tv-specs`' pressable-focus Tab test failed on `main` too: the GPUI test agent's `keystroke` sends a key through
  `input::key`, which skips GPUI's `FocusNext`, so a Tab past the last stop went nowhere. `pr/tv/03-dev-idiom`
  `48389b3f9` wraps an untaken Tab (`input::tab_past_ends`), as `FocusNext` does in a window with no other tab stop,
  in `docs/dev/gpui.md` too.
- GPUI's `tests::windows` failed on busy runners: `a_command_without_a_search_field_takes_the_keyboard_on_focus`
  pressed keys once the platform focus moved, before the command's `onFocus` had bound them in the engine. Reproduced
  under load (`yes > /dev/null`, twice per CPU); #215 waits for the bindings too (`keymap::bound`, test builds only)
  and gives the wait helpers `#[track_caller]`.

## The upstream branches (2026-11-12)

For `UP-4` (prepared, not opened: [../README.md](../README.md#prepared-not-opened-2026-11-12)), every branch that needs no
TV has an `up/` copy on `dXnSix/nsh`: its own commits cherry-picked onto `upstream/main` `ca7e70e51` (#1789), or onto
its parent's `up/` branch, the `pr/*` branch untouched. `up/engine/01-wgpu-30`, `up/tv/01-keys` … `04-remote-keys`,
`up/focus/01-core` … `07-web-tv` (`07` on `04` with `05`'s commit picked in). Each branch README has the base, what the
newer `upstream/main` asked of it and the checks run on its tip, all passing. What the base asked beyond conflicts:

- GPUI's keyboard focus on views lives in upstream's `key_focus.rs` (#1545, #1678, #1795): `up/focus/02-gpui`'s
  `1829d9316` folds it into the branch's `focus.rs`, as the fork's merge did for #1545.
- macOS's Tab (#1832, hidden views) and Windows' Tab order (#1616) in `up/focus/03-desktop-stops`; the popover test
  and `tab_order` in `up/focus/06-gpui-tab-stops`; `composedPath` (#1784) in the web focus tests of 05 and 07.
- The test agent's Tab wrap moved from `tv/03` to `up/focus/06-gpui-tab-stops`, the branch that has `tab_past_ends`.

The nsh-compat companions have `up/tv-idiom` and `up/focus-props` on `dXnSix/nsh-compat` ([compat/README.md](compat/README.md#the-upstream-branches-2026-11-12)).
`upstream/main` moved on the same day to `092438e52` (#1836): `up/tv/03-dev-idiom` and `up/focus/03-desktop-stops`
conflict with it (their READMEs say where); the rest merge onto it cleanly. Rebuild a branch on the newest
`upstream/main` before it is opened.

The upstream PRs still go from the latest `upstream/main` ([below](#updating-a-branch-and-opening-it)): a branch's
merge of the fork's `main` brings fork-only commits, so it is rebased (its own commits) before it is opened.

## Where things are

- Repository: the fork's checkout, shared by worktrees, `/Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen`
  (`upstream` = longbridge/nsh, `origin` = dXnSix/nsh). The branches are local there.
- `nsh-wt/tizen` itself is on **`feat/tizen`**, the epic's integration branch: every `pr/*` branch and
  `fork/tizen-ci` merged at its head (head `7c5cd1c13`, 2026-11-10 19:45 CET), cut from the fork's `origin/main` at
  planning (design `d28-nsh-branches`). The epic's checks ran there, never on a `pr/*` branch alone.
- The branches only ever moved forward: every repair of s15–s21 is a commit appended to its branch, and a branch that
  builds on another took the other's new head by a merge (the reviews checked "no rebase"). At 18:45 CET each stack
  took its parents' latest heads, and at 19:45 CET `pr/tizen/04-platform` took the latest `pr/tizen/01-os`
  (`a892c2d4c`), carried by merges into 05, 06, 07 and `pr/tizen-web/03-module`: no branch holds an older parent.
- Worktrees that have a branch checked out (switch the others in one of them, or add one):
  `nsh-wt/tizen-desktop` (`pr/focus/03-desktop-stops`), `nsh-wt/tizen-gpui02` (`pr/focus/06-gpui-tab-stops`),
  `nsh-wt/tizen-web-tv` (`pr/focus/07-web-tv`), `nsh-wt/tizen-engine` (`pr/tv/03-dev-idiom`),
  `nsh-wt/tizen-remote-keys` (`pr/tv/04-remote-keys`), `nsh-wt/tizen-os` (`pr/tizen/07-cli`), `nsh-wt/tizen-web`
  (`pr/tizen-web/03-module`), `nsh-wt/tizen-ci` (`fork/tizen-ci`).
- nsh-compat: the fork's clone `/Users/dinko/work-private/projects/dXn/nsh/nsh-compat` (`origin` = dXnSix/nsh-compat,
  `upstream` = longbridge/nsh-compat; [above](#the-forks-main-merged-in-2026-11-11)). Its branch `nsh-tizen` merges
  the three companions for local runs.

## The bases (2026-11-10)

- **`41100f2bf`** "cli: Let nsh.json say whether a web app's pages are browser history entries (#1158)",
  2026-11-09: the base of the `focus` stack.
- **`8c733731f`** "macos: Keep text visible inside transformed ancestors (#1135)", 2026-11-09, the next upstream
  commit: the base of everything else (`tv`, `engine`, `tizen`, `tizen-web`, `fork/tizen-ci`) and of `feat/tizen`.
- **`upstream/main` is `c16a70096`** "engine: Open shaders on the next backend when one refuses its device (#1601)",
  2026-11-10 23:27 +0800, fetched, and the same as the live `main` (`git ls-remote`): 162 commits after `41100f2bf`,
  161 after `8c733731f`.
- `git merge-tree` against `c16a70096`: `pr/focus/01-core`, `pr/engine/01-wgpu-30` and `fork/tizen-ci` merge
  cleanly; every other branch has conflicts, listed in its README. Upstream also merged work that overlaps ours
  (#1545 GPUI focus on focusable views, #1519 `focus-visible:` by input modality, #1439 web views reporting they are
  unavailable, #1475 Windows' RichEdit text inputs); the branch READMEs say where.
- **Every branch is rebased onto the latest `upstream/main` before it is opened**, and its README's checks are then run
  on the rebased branch alone, for the first time: every result recorded so far is from `feat/tizen`
  ([below](#updating-a-branch-and-opening-it)).

## Rules

They are nsh-root's (`/Users/dinko/work-private/projects/dXn/nsh/nsh-root/AGENTS.md`, "Upstream pull requests", and
`upstream-pr/README.md`), read-only for this epic:

- One thing per PR. A stack is opened one PR at a time: the next is opened once the one it builds on has merged,
  after moving it onto `upstream/main` so that it holds only its own commits (the shaders series does the same). Two
  open PRs never share commits: a squash of the top one would take the whole stack.
- A PR is the core behaviour, each platform's part and its compat case. New API passes the v0.2.0 record on its own
  (`cargo xtask compat`); when nsh-compat must learn to print something, its PR is opened first and named at the top
  of the NSH PR ([compat/README.md](compat/README.md)).
- Anything that changes a recorded meaning is raised in an issue before the PR. The issue drafts here are those
  issues; they are posted, and answered, before the PRs that depend on them are opened.
- Commits and titles are `<area>: <What the change does>`; public API is listed under `## Public API` in `PR.md`;
  documented behaviour changes its `docs/dev` page and the `website/` pages (English and `zh-CN`) in the same commit.
- Nothing is pushed, no issue or PR is opened, without the owner's go at that moment. The owner runs every
  `git push`, `gh issue create` and `gh pr create` here ("owner runs this").
- No AI attribution in commits, PR texts or comments. The NSH checklist's "Reviewed the changes ... AI generated
  code" box is the owner's to tick.

## Updating a branch and opening it

Every check result a branch README records comes from the integration branch `feat/tizen` (with the other branches
merged in), none from the branch alone; each README's checks section says which of its checks cannot run on the branch
alone, and gives the command that does where there is one. So before a branch is opened (owner's step):

1. **Rebase** it onto the latest `upstream/main` (commands below), resolving the conflicts its README lists (its
   "Base" row and "Before opening").
2. **Run its README's checks on its tip**, on that branch alone (in the worktree that has it), with the commands for
   the branch alone where its README gives them, and the pre-open checks below.
3. **Update the numbers and the checklist in `PR.md`**.

The epic's AgentX records name these branches' commits, and the fork's rule is not to rebase a committed branch.
Opening upstream needs a rebase all the same; keep the old head reachable first:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen      # or the worktree that has the branch
git fetch upstream origin
git tag epic/<area>/<topic> pr/<area>/<topic>                     # the commits the epic's records name
git rebase --onto upstream/main <its base or merged parent> pr/<area>/<topic>
# a branch with several parents: cherry-pick its own commits instead (its README lists them)
```

Then steps 2 and 3, replace the `#(…)` placeholders with the real issue and PR numbers, and (owner runs this):

```bash
git push -u origin pr/<area>/<topic>
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/<area>/<topic>
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/<area>/<topic> \
  --title "<line 1 of PR.md without '# '>" --body "$(sed '1,2d' PR.md)"
```

Record the PR number in the branch README, the area README and [../README.md](../README.md). The fork's `main` then
merges the same branch through a fork PR (merge commit), as nsh-root's `upstream-pr/README.md` describes; `feat/tizen`
stays the local mirror of that.

## Pre-open checks (every branch)

Besides each branch's own groups: the epic's final checks, not yet recorded for any branch
(`implementation-plan.json` `fin-nsh-*`), and upstream's checklist:

```bash
cargo +nightly fmt --all --check                          # fin-nsh-fmt
(cd crates/gpui && cargo +nightly fmt --all --check)      # fin-nsh-gpui-fmt
cargo clippy -p nsh-core -p nsh-js -p nsh-base -p nsh-engine -p nsh-platform-tui -p nsh-testing -p nsh-dev \
  -p nsh-cli -p xtask --all-targets -- -D warnings        # fin-nsh-clippy
(cd crates/gpui && cargo clippy --all-targets --features nsh-platform-gpui/dev -- -D warnings)   # fin-nsh-clippy-gpui
cargo clippy -p nsh-platform-web --target wasm32-unknown-unknown --features compiler -- -D warnings  # fin-nsh-clippy-web
cargo test -p <each touched crate>                        # upstream's checklist asks for `cargo test --workspace`
cargo xtask compat                                        # against nsh-compat main once its companion merged
```

Platform code also gets upstream's Catalyst and Android clippy runs (`cargo xtask clippy-android`), which the epic did
not run.

## The epic's checks and the environment they need

The checks a README lists are the epic's sprint checks (`.agentx/tasks/tizen-gpui-platform/implementation-plan.json`
`checks`, with their `argv` and owning tasks). Their results are in `.agentx/tasks/tizen-gpui-platform/state.json`
(`checkResults`, with test counts) and summed per sprint in `artifacts-temp/agentx/tizen-gpui-platform/check-s<n>.json`:

| Sprint | Branches | Final checks | Review |
|---|---|---|---|
| s14 | `pr/focus/01-core` | 2 passed (2 run), revision 236, 2026-11-09 13:15 UTC (`s14-check-3.json`) | 2 findings (`f-s14-cell-points-inexact` major, `f-s14-frame-unit-contract` minor), resolved by `ad514bb85` and `ce993fd21`; passed (`review-s14-3.json`) |
| s15 | `pr/focus/02` … `07` | 13 passed (5 run, 8 reused), revision 258, 2026-11-10 10:36–10:41 UTC | 8 findings, all resolved; passed (`review-s15-2.json`) |
| s16 | `pr/tv/*`, `pr/engine/01-wgpu-30` | 17 passed (2 run, 15 reused), revision 280, 2026-11-10 11:26–13:00 UTC | 8 findings, all resolved; passed (`review-s16-2.json`) |
| s17 | `pr/tizen/01` … `03` | 12 passed (1 run, 11 reused), revision 293, 2026-11-10 13:09–14:42 UTC | 4 findings, all resolved; passed (`review-s17-2.json`) |
| s18 | `pr/tizen/04` … `06` | 10 passed (10 run), revision 305, 2026-11-10 15:49–16:09 UTC | 7 findings, all resolved; passed (`review-s18-2.json`) |
| s19 | `pr/tizen/07-cli` | 9 passed (8 run, 1 reused), revision 321, 2026-11-10 16:14–17:14 UTC | 9 findings, all resolved; passed (`review-s19-3.json`) |
| s20 | `pr/tizen-web/*` | 6 passed (5 run, 1 reused), revision 336, 2026-11-10 17:17–17:46 UTC | 5 findings (one withdrawn), all resolved; passed (`review-s20-2.json`) |
| s21 | `fork/tizen-ci`, and `pr/tv/03-dev-idiom`'s repair | 7 passed (3 run, 4 reused), revision 351, 2026-11-10 17:51–18:30 UTC | 13 findings (with dXn.GpuiTizen's CI), all resolved; passed (`review-s21-3.json`) |

Every sprint from s14 to s21 is checkpointed; t22 (these folders) is the next task. The findings and what each
branch changed for them are in the branch READMEs and the area READMEs' review tables.

All of them ran in `nsh-wt/tizen` on `feat/tizen`, with the other branches merged in as of their sprint, so they prove
the branches together, not each one alone: no recorded result is from a branch alone. Running them on a branch: check it
out (its worktree), and for the checks owned by dXn.GpuiTizen (`cargo xtask …` from
`/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen`) point `--manifest-path` at that worktree's `crates/gpui`. A
check whose test, app or file is another branch's cannot run on the branch alone, or runs with fewer tests there; each
branch README says so under "On this branch alone", with the command that runs there where one exists (for the compat
checks: the branch's own nsh-compat companion, or `main`, in place of `nsh-tizen`).

The Tizen backend's checks need dXn.GpuiTizen and the two forks reachable before they are pushed, and the gpui-fast
patch kept outside the checkout ([tizen/04-platform](tizen/04-platform/README.md)). Written below as **`$TIZEN_ENV`**:

```bash
G=/Users/dinko/work-private/projects/dXn/dXn.GpuiTizen
export CARGO_NET_GIT_FETCH_WITH_CLI=true GIT_CONFIG_COUNT=3 \
  GIT_CONFIG_KEY_0=url.$G.insteadOf GIT_CONFIG_VALUE_0=https://github.com/dXnSix/dXn.GpuiTizen \
  GIT_CONFIG_KEY_1=url./Users/dinko/work-private/projects/dXn/gpui-fast.insteadOf GIT_CONFIG_VALUE_1=https://github.com/dXnSix/gpui-fast \
  GIT_CONFIG_KEY_2=url./Users/dinko/work-private/projects/dXn/wgpu.insteadOf GIT_CONFIG_VALUE_2=https://github.com/dXnSix/wgpu
PATCH=$G/artifacts-temp/gpui-fast-patch.toml                    # cargo xtask gpui-fast-release --verify writes it
LOCK=$G/artifacts-temp/locks/nsh-tizen-feat/Cargo.lock
mkdir -p "$(dirname "$LOCK")" && cp crates/gpui/Cargo.lock "$LOCK"
```

The web checks build the browser module with Homebrew's LLVM, written below as **`$WASM_BUILD`**:

```bash
CARGO_TARGET_DIR=target CC_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/clang \
  AR_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/llvm-ar \
  cargo build -p nsh-platform-web --target wasm32-unknown-unknown --no-default-features --features browser-tests \
  --profile web
```

The `tizen-web` checks run the CLI on a web runtime built from the checkout, written below as **`$WEB_RUNTIME`**
(then `NSH_RUNTIME="$PWD/target/nsh-runtime" cargo run -p nsh-cli -- test <app> --target tizen-web`):

```bash
CARGO_TARGET_DIR=target CC_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/clang \
  AR_wasm32_unknown_unknown=/opt/homebrew/opt/llvm/bin/llvm-ar \
  cargo build -p nsh-platform-web --profile web --no-default-features --features dev --target wasm32-unknown-unknown && \
  mkdir -p target/nsh-runtime/web && \
  cp target/wasm32-unknown-unknown/web/nsh_platform_web.wasm target/nsh-runtime/web/nsh-dev.wasm && \
  cp crates/nsh-platform-web/web/nsh.js target/nsh-runtime/web/nsh.js && \
  rm -rf target/nsh-runtime/web/browser && cp -R crates/nsh-platform-web/web/browser target/nsh-runtime/web/browser
```

The compat checks clone the local nsh-compat branch that merges the three companions:

```bash
rm -rf target/nsh-compat/nsh-compat && mkdir -p target/nsh-compat && \
  git clone -q --branch nsh-tizen /Users/dinko/work-private/projects/dXn/nsh/nsh-compat target/nsh-compat/nsh-compat && \
  NSH_BIN=/Users/dinko/work-private/projects/dXn/nsh/target-shared/debug/nsh cargo xtask compat
# expects "compatible with every recorded release"
```
