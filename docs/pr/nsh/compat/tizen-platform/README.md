# nsh-compat: tizen-platform

| | |
|---|---|
| PR | not opened (to `longbridge/nsh-compat`) |
| Branch | `tizen-platform`, head `c268493` (merges nsh-compat's `origin/main` `714e6fd`, 2026-11-11), in `/Users/dinko/work-private/projects/dXn/nsh-compat`, not pushed (no fork yet) |
| Base | `tv-idiom` (`a5188f7`), on `578f574`; `origin/main` is `ab2670a` (#45, one commit on), onto which it merges without conflict. Not rebased |
| Depends on | `tv-idiom` merged; then rebased so it holds only its two commits |
| Size | own: 2 files, +20 −4 (`build.rs` +8: the `nsh_has_tizen` gate; `src/apps.rs`); with `tv-idiom`: 3 files, +31 −10 |
| Its commits | `9c3cead` compat: Run a script as a native Tizen app with `platform tizen`; `57754c4` compat: Take `platform tizen` only from checkouts that have Os::Tizen |
| Text | [PR.md](PR.md) |
| Needed by | NSH `pr/tizen/01-os` (domain `tizen`: `platform tizen`) |
| Gate | `build.rs` reads the checkout's `crates/nsh-core/src/platform.rs` for the `Tizen` variant (`Os` has no name to look it up by) and sets `nsh_has_tizen` |

Without the gate, `platform <os>` with no second word keeps today's rule (a desktop OS is desktop); with it, the idiom
is the OS's own (`Idiom::default_on`, which `pr/tizen/01-os` adds).

The gate reads a source file by a string match (`"    Tizen,"`), fragile if `platform.rs` is reformatted. The other
gates look the API up by name in `nsh-core`'s prop specs; `Os` has no such list to search, and naming `Os::Tizen` would
not compile against checkouts without it. If the maintainers want another way, an `Os::ALL` list in NSH would do.

## Updating and opening

After `tv-idiom` has merged:

```bash
cd /Users/dinko/work-private/projects/dXn/nsh-compat
git fetch origin
git rebase --onto origin/main tv-idiom tizen-platform
# the checks in ../README.md, then (owner runs this)
git push -u fork tizen-platform
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/compat/tizen-platform
gh pr create --repo longbridge/nsh-compat --base main --head dXnSix:tizen-platform \
  --title 'compat: Run a script as a native Tizen app with `platform tizen`' --body "$(sed '1,2d' PR.md)"
```

Then put its number at the top of NSH's `pr/tizen/01-os` `PR.md`.
