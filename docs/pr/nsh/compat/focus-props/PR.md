# compat: Report views' focus props and focus groups

NSH is about to let views name where the arrow keys take the focus (longbridge/nsh#(pr/focus/04-tv)): `autoFocus` on
every element, `preferredFocus`, the `nextFocusUp` / `Down` / `Left` / `Right` routes, and `nsh/base`'s `FocusGroup`
(`destinations`, `trap`). Its compat cases check them with `prop` / `noprop` steps, which cannot print them yet.

`prop` and `noprop` steps can now check a view's `autoFocus` (an input keeps its own), `preferredFocus`, the four
`nextFocus*` routes and its focus group, written `focusGroup trap=<bool> destinations=<views>`. A route or a
destination is written as the selector that finds the view (`id:<testID>`, or `ref`):

```
prop id:resume nextFocusDown id:play
prop id:shelf focusGroup trap=false destinations=id:second
```

`build.rs` turns this on only when the NSH checkout declares `preferredFocus`, so the checker still builds against
NSH without these props and can merge on its own.

## How to test

- `cargo test focus_props_name_the_views_they_route_to` (and the rest of `cargo test`), against NSH's `main`: the
  gate is off and nothing changes.
- `cargo xtask compat` in an NSH checkout of longbridge/nsh#(pr/focus/04-tv): `props-view-focus` (the props, a route
  to a view rendered after the one naming it) and `base-focus-group` (trap and destinations following the app's
  state) pass.
