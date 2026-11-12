# nsh-compat: focus-props

| | |
|---|---|
| PR | ready, not opened (to `longbridge/nsh-compat`, after the owner's agreement with its owners) |
| Upstream branch | `up/focus-props` on `dXnSix/nsh-compat`, head `856ad9d`, pushed 2026-11-12: this branch's own commit on longbridge's `main` `e56ebce` (#49) |
| Branch | `focus-props` on [dXnSix/nsh-compat](https://github.com/dXnSix/nsh-compat), head `28b3b49` (merges longbridge's `main` `e56ebce`, #49, 2026-11-11), in `/Users/dinko/work-private/projects/dXn/nsh/nsh-compat` |
| Base | `578f574` (#40); `origin/main` is `ab2670a` (#45, one commit on, fetched 2026-11-10), onto which it merges without conflict. Not rebased |
| Size | 2 files, +145 −3: `build.rs` (+6: the `nsh_has_focus_routes` gate), `src/values.rs` (+142 −3: writing the props, and the test `focus_props_name_the_views_they_route_to`) |
| Its commits | `32a1859` compat: Report views' focus props and focus groups |
| Text | [PR.md](PR.md) |
| Needed by | NSH `pr/focus/04-tv` (`props/props-view-focus`, `base/base-focus-group`) |
| Gate | `build.rs` turns it on (`nsh_has_focus_routes`) only when the NSH checkout declares `preferredFocus` |

`nsh-tizen` merges it with `tizen-platform`, whose `build.rs` change touches the same lines (a conflict resolved there,
`bc033c9`); whichever merges second upstream gets that small conflict.

## Updating and opening

After the fork exists ([../README.md](../README.md#where-things-are)):

```bash
cd /Users/dinko/work-private/projects/dXn/nsh/nsh-compat
git fetch origin
git rebase origin/main focus-props
# the checks in ../README.md, then (owner runs this)
git push -u origin focus-props
cd /Users/dinko/work-private/projects/dXn/dXn.GpuiTizen/docs/pr/nsh/compat/focus-props
gh pr create --repo longbridge/nsh-compat --base main --head dXnSix:focus-props \
  --title "compat: Report views' focus props and focus groups" --body "$(sed '1,2d' PR.md)"
```

Then put its number at the top of NSH's `pr/focus/04-tv` `PR.md`.
