# nsh-compat: tv-idiom

| | |
|---|---|
| PR | not opened (to `longbridge/nsh-compat`) |
| Branch | `tv-idiom`, head `84f680f` (merges nsh-compat's `origin/main` `714e6fd`, 2026-11-11), in `/Users/dinko/work-private/projects/dXn/nsh-compat`, not pushed (no fork yet) |
| Base | `578f574` (#40); `origin/main` is `ab2670a` (#45, one commit on, fetched 2026-11-10), onto which it merges without conflict. Not rebased |
| Size | 2 files, +15 −10: `src/apps.rs` (the `platform` step, its docs and error), `src/values.rs` (`tv.` among the platform prefixes) |
| Its commits | `a5188f7` compat: Run a script on a TV with `platform <os> tv` |
| Text | [PR.md](PR.md) |
| Needed by | NSH `pr/tv/02-category` (the `tv` domain: `platform android tv`), later `pr/tizen/06-module` (`tv/mod-tizen`); the base of `tizen-platform` |
| Gate | none needed: `Idiom::Tv` is in every NSH checkout |

## Updating and opening

After the fork exists ([../README.md](../README.md#where-things-are)):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh-compat
git fetch origin
git rebase origin/main tv-idiom
# the checks in ../README.md, then (owner runs this)
git push -u fork tv-idiom
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/compat/tv-idiom
gh pr create --repo longbridge/nsh-compat --base main --head dXnSix:tv-idiom \
  --title 'compat: Run a script on a TV with `platform <os> tv`' --body "$(sed '1,2d' PR.md)"
```

Then put its number at the top of NSH's `pr/tv/02-category` `PR.md`, and rebase `tizen-platform` onto `origin/main`
once it has merged.
