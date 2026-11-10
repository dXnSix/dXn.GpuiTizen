# pr/tizen-web/02-scrollend

| | |
|---|---|
| Upstream PR | not opened |
| Branch | `pr/tizen-web/02-scrollend`, head `53670ef6a`, local in `nsh-wt/tizen`, not pushed |
| Base | `upstream/main` `8c733731f`. Against `c16a70096` it conflicts in `website/docs/web.md` and `website/zh-CN/docs/web.md`. Not rebased |
| Depends on | nothing |
| Size | 5 files, +158 −4: `crates/nsh-platform-web/web/nsh.js` (+22 −1), the new `web/browser/scrollend.test.mjs` (+130), `docs/dev/web.md`, the website's web page (English and zh-CN) |
| Its commits | `53670ef6a` web: End scrolls without scrollend |
| Text | [PR.md](PR.md) |
| In `feat/tizen` | yes (merge `304fc3b24`) |
| Review findings | none: s20's review (`review-s20-2.json`, passed) found the `scrollend` fallback correct |
| Needs | no issue (a fix), no nsh-compat change |

It is independent of everything else here and could be opened at any time; [../../../README.md](../../../README.md)
keeps it with the `tizen-web` area, after the loader probe, which also reports whether the TV's engine has
`scrollend`.

## Checks

| Check | Command (in the branch's checkout) | Task | Recorded on `feat/tizen` |
|---|---|---|---|
| `chk-nsh-scrollend` | `node --expose-gc --test crates/nsh-platform-web/web/browser/scrollend.test.mjs` | `t20-scrollend` | passed, 4 tests, 2026-11-10 17:17 UTC (s20; reused by the final check, revision 336) |

**On this branch alone** (no result above is from it): the command runs as written (4 tests there); the recorded run
loaded `feat/tizen`'s `nsh.js`, with the focus and TV branches' changes. Not run: the pre-open checks.

## Updating and opening

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-wt/tizen
git fetch upstream origin
git tag epic/tizen-web/02-scrollend pr/tizen-web/02-scrollend
git rebase --onto upstream/main 8c733731f pr/tizen-web/02-scrollend
# the check above and the pre-open checks, then (owner runs this)
git push -u origin pr/tizen-web/02-scrollend
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/tizen-web/02-scrollend
gh pr create --repo longbridge/nsh --base main --head dXnSix:pr/tizen-web/02-scrollend \
  --title "web: End scrolls without scrollend" --body "$(sed '1,2d' PR.md)"
```
