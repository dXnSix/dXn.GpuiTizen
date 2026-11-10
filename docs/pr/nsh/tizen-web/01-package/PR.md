# cli: Package, run and test apps as Tizen web apps

Part of #(Tizen issue). **Depends on:** #(pr/tv/01-keys) and #(pr/tv/02-category), merged. No nsh-compat change.

## Description

Samsung TVs (2024 on, Tizen 8) run web apps (`.wgt`) in their browser engine. An NSH app's published site runs there
with Tizen's manifest and a TV shim; `platform.os` stays `"web"` and the app is in the TV idiom (`platform.tv`,
`tv={{…}}` props).

**`nsh package --target tizen-web [--sign <profile>] [--run]`** writes the site `--target web` writes
(`web::write_site`, with a `Variant`) with Tizen's `config.xml`, the app's icon at 512×423 and a TV shim the page runs
before `nsh.js`, in `<out>/tizen-web/<name>/`, zipped into `<out>/tizen-web/<name>.wgt`. With a certificate profile
(`--sign`, or `nsh.json` `tizen.signing`) the Tizen SDK's `tz` signs it into `<out>/<name>.wgt` (`tz pack -t wgt -s
<profile>`), and `--run` installs and launches it with `tz` (`tz run -p` with the package id). The page starts the player with `idiom: "tv"` and no
history entries, and reads its files where it is (no `web.base`, no `.br` / `.gz`).

**The TV shim** sends the remote's Back (key code 10009) to the app as `BrowserBack`, then as `Escape`; when nothing
takes it, it asks whether to exit and calls `tizen.application.getCurrentApplication().exit()`. It registers the color,
media and channel keys with `tizen.tvinputdevice.registerKey`, which `nsh.js` reads by Samsung's key codes in the TV
idiom (#(pr/tv/01-keys)), and exposes `globalThis.nshTizen` for the web half of `nsh/tizen`.

**`nsh run --target tizen-web`** (and `nsh dev`) serves the dev server's page as a Tizen web app's (the shim, the TV
idiom, no history entries), so a browser here shows the app as the TV does, with hot updates. It writes a development
package (`NshDevHost.NshDev`) whose page opens the dev server's at this machine's LAN address once it answers. Its
policy is a development package's: `<access origin="*" subdomains="true"/>` (the dev page's `ws://` socket for hot
updates and logs) and `<tizen:allow-navigation>` to the dev server's host; the remote dev page has no `$WEBAPIS`
(only package pages resolve it). With `tizen.signing` it is signed, installed and launched with `tz` (`--device`: the
TV's serial). **`nsh test --target tizen-web`** runs the tests as `--target web`
does, in headless Chromium, on that page, at 1920×1080 by default.

**`nsh package --target tizen-web --probe`** writes, instead of the app, a package (`NshLdProbe.Probe`, signed into
`<out>/NshProbe.wgt`) whose page tries each way the app's page loads what it needs from package files: a module script,
the page's import map (a static import and `import()`), a same-origin iframe realm, a classic bootstrap fetched with
CORS into it, the realm's import map and a blob module, a Wasm `fetch` (`instantiateStreaming` and `arrayBuffer`), the
same fetch in a package page under `connect-src 'self'` (`probe/csp.html` in an iframe: the policy an app's page gets
when its `capabilities` limit the network), and Samsung's `$WEBAPIS/webapis/webapis.js` (`webapis` defined). It shows
each result with the user agent, the origin and whether the engine has `scrollend`, `popover` and `inert`.
On a [OWNER: TV model, firmware]: [OWNER: the probe's results].

**`nsh.json` `tizen`** names the package: `package` (10 letters and digits), `appId`, `version`, `apiVersion`,
`privileges`, `signing`, every key with a default; ids and versions Tizen refuses are errors.

**Smart Hub's preview** is opt-in, `nsh.json` `tizen.smartHubPreview`, as Samsung's guides describe it:

- a URL is a public preview: `<tizen:metadata key="http://samsung.com/tv/metadata/use.preview"
  value="endpoint_URL=<url>"/>`, whose JSON the TV reads from the app's server;
- `"service"` is a personal preview (`bg_service`): Samsung sets it from a background service of the package, so the
  package carries NSH's preview service (`service/preview.js`, as `<tizen:service id="<package>.NshPreview">`) and asks
  for `application.launch`. The shim's `nshTizen.preview` names the service, for `nsh/tizen`'s `setPreviewData`
  (#(pr/tizen-web/03-module)) to start it with the data, which the service hands to
  `webapis.preview.setPreviewData` before it exits.

Tiles' `action_data` does not reach the app yet. Both forms, signing, launching and the development package are
unverified on a TV until the device checklist in `docs/dev/web.md` (with the probe) has run: [OWNER: DEV-6].

Docs: `docs/dev/web.md` ("Tizen web apps"), `docs/dev/compatibility.md`; the website's web, CLI, platforms and
project-structure pages (English and zh-CN).

## Public API

- CLI: `--target tizen-web` for `nsh package` (`--sign <profile>`, `--run`, `--probe`), `nsh run`, `nsh dev`,
  `nsh test`; `--device <serial>` for `nsh run --target tizen-web`.
- `nsh.json`: `tizen.package`, `tizen.appId`, `tizen.version`, `tizen.apiVersion`, `tizen.privileges`,
  `tizen.signing`, `tizen.smartHubPreview` (a URL or `"service"`).

## How to Test

- `cargo test -p nsh-cli --test cli tizen_web::` (a fake linkable runtime and a fake `tz`: the package's files and
  `config.xml`, signing, the development package's policy and the dev server's page, install and launch by package
  id, the probe's files, `--probe` refused off `tizen-web`, both Smart Hub previews), `cargo test -p nsh-cli --bin nsh
  tizen_web::`.
- `node --test crates/nsh-cli/src/tizen_web/shim.test.mjs` (the shim with fake `tizen` globals: Back, the exit
  question, the registered keys, the preview service).
- `nsh test examples/counter --target tizen-web`.
- On a TV: the probe, then the gallery as a `.wgt` driven by the remote.

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): CLI and `nsh.json` only.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (on a Samsung TV instead).
