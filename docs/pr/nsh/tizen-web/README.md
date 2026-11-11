# tizen-web: Samsung TVs through the TV's browser engine

The second Samsung TV target: the app as a Tizen web app (`.wgt`), NSH's web runtime in the TV's browser engine with a
TV shim (Back, the exit question, the remote's keys, the TV idiom), `scrollend` for engines without it, and
`nsh/tizen` from the TV's web APIs. It shares the `tizen` issue ([../tizen/issue.md](../tizen/issue.md)).

## The PRs, in order

All pushed to `dXnSix/nsh` (2026-11-10), worked on in `nsh-wt/tizen`; built on `upstream/main` `8c733731f`.

| # | Folder | Branch | Built on | Title | Own size | Needs | Status |
|---|---|---|---|---|---|---|---|
| 01 | [01-package/](01-package/) | `pr/tizen-web/01-package` | `pr/tv/02-category`, merges `pr/tv/01-keys` (both at their heads) | cli: Package, run and test apps as Tizen web apps | 35 files, +2863 −66 | `pr/tv/01-keys`, `pr/tv/02-category` merged; the loader probe on a TV (below) | not opened |
| 02 | [02-scrollend/](02-scrollend/) | `pr/tizen-web/02-scrollend` | `upstream/main` `8c733731f` | web: End scrolls without scrollend | 5 files, +158 −4 | – | not opened |
| 03 | [03-module/](03-module/) | `pr/tizen-web/03-module` | `pr/tizen/06-module` + 01 (both at their heads) | web: Serve nsh/tizen in Tizen web apps from the TV's web APIs | 14 files, +427 −26 | `pr/tizen/06-module` and 01 merged | not opened |

```
pr/tv/02 + pr/tv/01 ── 01-package ──┬── (pr/tizen/07-cli merges it)
pr/tizen/06-module ─────────────────┴── 03-module
02-scrollend  independent
```

- 01 comes before `pr/tizen/07-cli`, which reads its `nsh.json` `tizen` section ([../tizen/README.md](../tizen/README.md)).
- 02 needs nothing and fixes any engine without `scrollend` (Chromium before 114 among them): it can be
  opened at any time, on the owner's call; [../../README.md](../../README.md) keeps it with this area.
- 03 waits for `pr/tizen/06-module` (`nsh/tizen` itself) and 01.

## The loader probe (DEV-6)

Whether a TV's engine loads what the web runtime needs from package files (module scripts, the page's import map,
an iframe realm with a CORS-fetched bootstrap, a blob module, Wasm through `fetch`, a package page under
`connect-src 'self'`, Samsung's `$WEBAPIS/webapis/webapis.js`) is not known from the desk. 01 has a probe for it:
`nsh package --target tizen-web --probe` writes and signs `NshProbe.wgt`, which shows each result on screen with the
user agent, the page's origin and whether the engine has `scrollend`, `popover` and `inert`. The same `DEV-6` run
checks what else is unverified until a TV: signing and `tz run -p`, the development package's policy, and Smart Hub's
preview (public and personal; a personal preview Smart Hub starts with no data only exits). The
epic's `DEV-6` (owner-run) installs it on a TV first and records the results; then the gallery as a `.wgt`, navigable
by remote, with start-up and frame times. If the probe fails, the ways out (a hosted web app, or a classic-script build)
become tasks before 01 is opened. Results: [OWNER: DEV-6].

## State (2026-11-10, 19:30 CET)

- **Every branch holds its parents at their heads.** 01 holds `pr/tv/01-keys` and `pr/tv/02-category` at their heads;
  03 merged the latest 01 (`a59cb3ada`) and the latest `pr/tizen/06-module` (`661d69f2d`), so its tree now has the
  crates.io `version` on the dXn.GpuiTizen entries, the C# project fix, the loopback-only listener and the localized
  exit question. 03's own commits are `3be0df576` and `54f54bc36`. All three are in `feat/tizen` (head `7c5cd1c13`).
- 01 gained four commits since the draft (`6bfecf051`, `6f0005235`, `1fd94197c`, `0294d0204`) and 03 one
  (`54f54bc36`): the repairs of the s20 review.
- s20's checks ran (`check-s20.json`, revision 336: 6 passed, 5 run and 1 reused) and its review passed
  (`review-s20-2.json`, snapshot `2b098ae8`); the tasks are `done` and the sprint is checkpointed.
- Against `upstream/main` `c16a70096`: 01 conflicts in `nsh-cli`'s `package.rs` and `test_web.rs`,
  `crates/nsh-core/src/prelude.rs`, `crates/nsh-platform-web/src/browser/engine.rs`, `web/nsh.test.mjs` and the
  website's CLI and project-structure pages (English and zh-CN); 02 in the website's web page (English and zh-CN); 03
  in both its parents' files (`.github/workflows/gpui.yml`, `nsh-platform-gpui`'s `app.rs` and `dev.rs`,
  `website/zh-CN/docs/gpui.md` besides 01's).

## The s20 review

| Finding | Branch | What changed |
|---|---|---|
| `f-s20-preview-metadata` (major) | 01, 03 | no package declared Smart Hub's preview: `nsh.json` `tizen.smartHubPreview` opts in, a URL as a public preview (`use.preview` `endpoint_URL=<url>`), `"service"` as a personal one (`bg_service`, NSH's preview service in the package, `application.launch`) (`0294d0204`); `nsh/tizen`'s `setPreviewData` in a Tizen web app starts that service with the data (`54f54bc36`) |
| `f-s20-preview-endpoint-value` (major) | 01 | withdrawn: Samsung's guide shows `endpoint_URL=<url>`, which the package writes; no source change |
| `f-s20-dev-package-policy` (minor) | 01 | the development package's `config.xml` opens `<access origin="*" subdomains="true"/>` (the dev page's `ws://` socket) and navigation to the dev server's host (`6f0005235`) |
| `f-s20-tz-run-id` (minor) | 01 | `tz run -p` takes the package id (`6bfecf051`) |
| `f-s20-probe-csp` (minor) | 01 | the probe adds a CSP `'self'` row (`probe/csp.html`) and a `webapis` row (`1fd94197c`) |

02 had no finding.

## Rules for this area

nsh-root's ([../README.md](../README.md#rules)).
