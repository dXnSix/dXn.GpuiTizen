# compat: the nsh-compat companions

Three small PRs to [longbridge/nsh-compat](https://github.com/longbridge/nsh-compat), so that the compat scripts of
the NSH PRs can say what they check. Each is opened (and merged) before the NSH PR that needs it, and named at the top
of that PR's text, as nsh-root's rules ask. Each builds against NSH checkouts with or without the NSH change (the
`build.rs` gates below), so it can merge on its own; nsh-compat's CI builds it against NSH's `main`.

## The PRs, in order

All in the local clone `/Users/dinko/work-private/projects/dXn/nsh-compat`, not pushed; their heads have not moved
since s16. Each is cut from `578f574` (#40, 2026-11-10). `origin/main` is `ab2670a` (#45, "compat: Name the `touch`
step's pointer a finger", 2026-11-10 22:58 +0800; fetched 2026-11-10 19:30 CET), one commit on; all three branches
merge onto it without conflict (`git merge-tree`). Rebase before opening. Sizes against `578f574`.

| # | Folder | Branch | Title | Size | NSH PR that needs it | Status |
|---|---|---|---|---|---|---|
| 01 | [tv-idiom/](tv-idiom/) | `tv-idiom` | compat: Run a script on a TV with `platform <os> tv` | 2 files, +15 −10 | [`pr/tv/02-category`](../tv/02-category/) (domain `tv`, `platform android tv`); later `pr/tizen/06-module` (`tv/mod-tizen`) | not opened |
| 02 | [focus-props/](focus-props/) | `focus-props` | compat: Report views' focus props and focus groups | 2 files, +145 −3 | [`pr/focus/04-tv`](../focus/04-tv/) (`props-view-focus`, `base-focus-group`) | not opened |
| 03 | [tizen-platform/](tizen-platform/) | `tizen-platform` (on `tv-idiom`) | compat: Run a script as a native Tizen app with `platform tizen` | own: 2 files, +20 −4 (with 01: 3 files, +31 −10) | [`pr/tizen/01-os`](../tizen/01-os/) (domain `tizen`, `platform tizen`) | not opened |

`nsh-tizen` (local only, never a PR) merges all three for the epic's compat runs (`chk-nsh-compat`,
`chk-nsh-compat-tizen`), which clone it into NSH's `target/nsh-compat/`.

The split itself is a review repair: s16's `f-s16-nsh-compat-one-thing-per-pr` (minor) found one nsh-compat branch
serving three NSH PRs; it became these three topic branches, with `nsh-tizen` (`bc033c9`) merging them, whose tree
equals the old single branch's (resolved in `review-s16-2.json`). The same review's `f-s16-tv5-os-tizen-unmet` moved
`Os::Tizen`'s compat case to NTZ-2 by amendment: `tizen-platform` serves `pr/tizen/01-os`, checked by s17's
`chk-nsh-compat-tizen`. s16's re-review accepted the `tizen-platform` gate that matches `platform.rs`'s text.

## Where things are

- There is **no fork** `dXnSix/nsh-compat` yet (`gh repo view` finds none, 2026-11-10), and the clone's only remote
  is `origin` = longbridge/nsh-compat. The owner makes the fork once:

  ```bash
  gh repo fork longbridge/nsh-compat --clone=false          # owner runs this
  git -C /Users/dinko/work-private/projects/dXn/nsh-compat remote add fork https://github.com/dXnSix/nsh-compat.git
  ```
- The crate depends on NSH at `../nsh` (`/Users/dinko/work-private/projects/dXn/nsh`, the fork's `main` checkout).
  Its tests therefore run against whatever that checkout holds; to test against an NSH branch, use NSH's
  `cargo xtask compat` (which clones nsh-compat into `target/nsh-compat/`, next to the checkout under test).
- nsh-compat has no PR template and no issues (`hasIssuesEnabled: false`); its PRs are a description and
  `## How to test` (as #41).

## Checks (each branch)

```bash
cd /Users/dinko/work-private/projects/dXn/nsh-compat
git switch <branch>
(cd ../nsh && cargo build -p nsh-cli)
cp ../nsh/Cargo.lock . && cargo test --target-dir ../nsh/target    # against ../nsh (the fork's main)
cargo fmt --check
```

and, from an NSH checkout of the PR that needs it, `cargo xtask compat` with `target/nsh-compat/nsh-compat` cloned from
the branch. The epic ran the last one only, with `nsh-tizen` (all three): `chk-nsh-compat` passed on `feat/tizen`
2026-11-10 13:00 UTC (s16), `chk-nsh-compat-tizen` 14:42 UTC (s17), both "compatible with every recorded release".
Not run: `cargo test` in nsh-compat on any branch.
