# compat: the nsh-compat companions

Three small PRs to [longbridge/nsh-compat](https://github.com/longbridge/nsh-compat), so that the compat scripts of
the NSH PRs can say what they check. Each is opened (and merged) before the NSH PR that needs it, and named at the top
of that PR's text, as nsh-root's rules ask. Each builds against NSH checkouts with or without the NSH change (the
`build.rs` gates below), so it can merge on its own; nsh-compat's CI builds it against NSH's `main`.

## The PRs, in order

Pushed to the fork [dXnSix/nsh-compat](https://github.com/dXnSix/nsh-compat) (2026-11-11), cloned at
`/Users/dinko/work-private/projects/dXn/nsh/nsh-compat` (`origin` the fork, `upstream` longbridge's): there each
merged longbridge's `main` `e56ebce` (#49), and the fork's `main` and `nsh-tizen` are `99ef2fc`, merging the three.
The NSH fork's `compat` job runs that `main` ([../README.md](../README.md#the-forks-main-merged-in-2026-11-11)).
Earlier, in the old local clone, on 2026-11-11 each merged nsh-compat's `origin/main` `714e6fd` (#48), which has #45's `TouchPoint::pointer_type` that NSH's `main` now needs
(`build.rs`'s `rustc-check-cfg` lists kept both sides), and `nsh-tizen` merged the three again (`d576912`); before that
their heads had not moved since s16. The text below is as of 2026-11-10. Each is cut from `578f574` (#40, 2026-11-10). `origin/main` is `ab2670a` (#45, "compat: Name the `touch`
step's pointer a finger", 2026-11-10 22:58 +0800; fetched 2026-11-10 19:30 CET), one commit on; all three branches
merge onto it without conflict (`git merge-tree`). Rebase before opening. Sizes against `578f574`.

| # | Folder | Branch | Title | Size | NSH PR that needs it | Status |
|---|---|---|---|---|---|---|
| 01 | [tv-idiom/](tv-idiom/) | `tv-idiom` | compat: Run a script on a TV with `platform <os> tv` | 2 files, +15 −10 | [`pr/tv/02-category`](../tv/02-category/) (domain `tv`, `platform android tv`); later `pr/tizen/06-module` (`tv/mod-tizen`) | ready, not opened: `up/tv-idiom` |
| 02 | [focus-props/](focus-props/) | `focus-props` | compat: Report views' focus props and focus groups | 2 files, +145 −3 | [`pr/focus/04-tv`](../focus/04-tv/) (`props-view-focus`, `base-focus-group`) | ready, not opened: `up/focus-props` |
| 03 | [tizen-platform/](tizen-platform/) | `tizen-platform` (on `tv-idiom`) | compat: Run a script as a native Tizen app with `platform tizen` | own: 2 files, +20 −4 (with 01: 3 files, +31 −10) | [`pr/tizen/01-os`](../tizen/01-os/) (domain `tizen`, `platform tizen`) | not opened |

`nsh-tizen` (on the fork, never a PR) merges all three for the epic's compat runs (`chk-nsh-compat`,
`chk-nsh-compat-tizen`), which clone it into NSH's `target/nsh-compat/`.

The split itself is a review repair: s16's `f-s16-nsh-compat-one-thing-per-pr` (minor) found one nsh-compat branch
serving three NSH PRs; it became these three topic branches, with `nsh-tizen` (`bc033c9`) merging them, whose tree
equals the old single branch's (resolved in `review-s16-2.json`). The same review's `f-s16-tv5-os-tizen-unmet` moved
`Os::Tizen`'s compat case to NTZ-2 by amendment: `tizen-platform` serves `pr/tizen/01-os`, checked by s17's
`chk-nsh-compat-tizen`. s16's re-review accepted the `tizen-platform` gate that matches `platform.rs`'s text.

### The upstream branches (2026-11-12)

`up/tv-idiom` (`cd0d519`) and `up/focus-props` (`856ad9d`) on `dXnSix/nsh-compat` hold each companion's own commit on
longbridge's `main` `e56ebce` (#49); `focus-props` keeps both sides of `build.rs`'s `rustc-check-cfg` list (#49's
`nsh_has_accessibility_description`). `tizen-platform` waits for `UP-5`; on that `main` it conflicts the same way in
`build.rs`, and its `nsh_has_tizen` lines need `cargo fmt`.

Checks (logs under `artifacts-temp/agentx/tizen-gpui-platform/t25-compat/`): `cargo fmt --check` on both; `cargo test`
against the fork's `main` (`../nsh` at `4d5dd4f0f`) with the three companions together: 39 passed. Each alone fails
only `checkout_apps_run` on the cases the others serve (`focusGroup`; `platform tizen`, `platform android tv`), as the
fork's `main` has all three. From NSH, `cargo xtask compat` passed on `up/tv/02-category` with `up/tv-idiom` and on
`up/focus/04-tv` with `up/focus-props` (compatible with every recorded release).

## Where things are

- The fork `dXnSix/nsh-compat` exists (the owner's choice, 2026-11-11). Its clone is
  `/Users/dinko/work-private/projects/dXn/nsh/nsh-compat`, with `origin` the fork, `upstream` longbridge/nsh-compat
  and `local` the old clone `/Users/dinko/work-private/projects/dXn/nsh-compat`, no longer used.
- The crate depends on NSH at `../nsh` (`/Users/dinko/work-private/projects/dXn/nsh/nsh`, the fork's `main` checkout).
  Its tests therefore run against whatever that checkout holds; to test against an NSH branch, use NSH's
  `cargo xtask compat` (which clones nsh-compat into `target/nsh-compat/`, next to the checkout under test).
- nsh-compat has no PR template and no issues (`hasIssuesEnabled: false`); its PRs are a description and
  `## How to test` (as #41).

## Checks (each branch)

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-compat
git switch <branch>
(cd ../nsh && cargo build -p nsh-cli)
cp ../nsh/Cargo.lock . && cargo test --target-dir ../nsh/target    # against ../nsh (the fork's main)
cargo fmt --check
```

and, from an NSH checkout of the PR that needs it, `cargo xtask compat` with `target/nsh-compat/nsh-compat` cloned from
the branch. The epic ran the last one only, with `nsh-tizen` (all three): `chk-nsh-compat` passed on `feat/tizen`
2026-11-10 13:00 UTC (s16), `chk-nsh-compat-tizen` 14:42 UTC (s17), both "compatible with every recorded release".
Not run: `cargo test` in nsh-compat on any branch.
