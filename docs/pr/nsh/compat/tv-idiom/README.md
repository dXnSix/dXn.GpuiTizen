# nsh-compat: tv-idiom

| | |
|---|---|
| PR | ready, not opened (to `longbridge/nsh-compat`, after the owner's agreement with its owners) |
| Upstream branch | `up/tv-idiom` on `dXnSix/nsh-compat`, head `cd0d519`, pushed 2026-11-12: this branch's own commit on longbridge's `main` `e56ebce` (#49) |
| Branch | `tv-idiom` on [dXnSix/nsh-compat](https://github.com/dXnSix/nsh-compat), head `4ab402d` (merges longbridge's `main` `e56ebce`, #49, 2026-11-11), in `/Users/dinko/work-private/projects/dXn/nsh/nsh-compat` |
| Base | `578f574` (#40); `origin/main` is `ab2670a` (#45, one commit on, fetched 2026-11-10), onto which it merges without conflict. Not rebased |
| Size | 2 files, +15 −10: `src/apps.rs` (the `platform` step, its docs and error), `src/values.rs` (`tv.` among the platform prefixes) |
| Its commits | `a5188f7` compat: Run a script on a TV with `platform <os> tv` |
| Text | [PR.md](PR.md) |
| Needed by | NSH `pr/tv/02-category` (the `tv` domain: `platform android tv`), later `pr/tizen/06-module` (`tv/mod-tizen`); the base of `tizen-platform` |
| Gate | none needed: `Idiom::Tv` is in every NSH checkout |

## Updating and opening

Nothing is opened before the owner's agreement with nsh-compat's owners. The branch that opens is `up/tv-idiom` on
`dXnSix/nsh-compat`: before opening, rebuild it with its own commit on the newest longbridge `main` (`upstream`), run
the checks in [../README.md](../README.md#checks-each-branch), then (owner runs this):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-compat
git push --force-with-lease origin up/tv-idiom
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/compat/tv-idiom
gh pr create --repo longbridge/nsh-compat --base main --head dXnSix:up/tv-idiom \
  --title 'compat: Run a script on a TV with `platform <os> tv`' --body "$(sed '1,2d' PR.md)"
```

Then put its number at the top of NSH's `up/tv/02-category` `PR.md`, and rebuild `tizen-platform` on longbridge's `main`
once it has merged.
