# web: Serve nsh/tizen in Tizen web apps from the TV's web APIs

Closes #(Tizen issue) for the web target. **Depends on:** #(pr/tizen/06-module) and #(pr/tizen-web/01-package),
merged. No nsh-compat change.

## Description

`nsh/tizen` (#(pr/tizen/06-module)) is available natively on a TV; in a Tizen web app it rejected every call. In a
Tizen web app (`--target tizen-web`) the browser runtime now puts its own `nsh/tizen` in place of the runtime's
(`web/browser/tizen.js`, `createTizen`). Where the page's TV shim says it runs on a TV (`nshTizen.tv`):

- `isAvailable()` is true;
- `productInfo()` reads `webapis.productinfo` and `tizen.systeminfo`;
- `registerKey` / `unregisterKey` map `nsh/keymap`'s names to Tizen's for `tizen.tvinputdevice`, checked as natively
  (a `TypeError` for no key, a `RangeError` for one the TV keeps);
- `exitApp()` leaves at once through `tizen.application`;
- `setPreviewData(data)` sets Smart Hub's personal preview: Samsung takes it only from a background service of the
  package, so the shim launches the package's preview service (#(pr/tizen-web/01-package)'s `nsh.json`
  `tizen.smartHubPreview: "service"`, `nshTizen.preview.service`) with `tizen.application.launchAppControl` and the
  data as JSON under `preview`, and the service hands it to `webapis.preview.setPreviewData`. Without a personal
  preview, or with a public one (a URL, which the TV reads itself), the call rejects and says why.

The TV's errors reject with `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM`; promises and errors are the app realm's. Any other
page keeps the runtime's module, unavailable as on every platform but a TV.

The shim's `globalThis.nshTizen` gains `productInfo`, `registerKey`, `unregisterKey` and `setPreviewData`; packages
load the TV's `$WEBAPIS/webapis/webapis.js` before it (`web::Variant::device_scripts`) and ask for the
`http://developer.samsung.com/privilege/productinfo` privilege.

Docs: `docs/dev/{js-modules,web}.md` (what Smart Hub needs, unverified on a TV); the website's web and `nsh/tizen`
pages (English and zh-CN).

## Public API

None new: `nsh/tizen`'s functions (#(pr/tizen/06-module)) now work in Tizen web apps, `setPreviewData` among them
(with `tizen.smartHubPreview: "service"`). Packages for `--target tizen-web` ask for the `productinfo` privilege.

## How to Test

- `node --test crates/nsh-platform-web/web/browser/tizen.test.mjs` (the module with fake `tizen` and `webapis`
  globals, and without them).
- `node --test crates/nsh-cli/src/tizen_web/shim.test.mjs` (the shim's TV half, `setPreviewData` launching the preview
  service or rejecting without one), `cargo test -p nsh-cli --test cli tizen_web::` (the privilege and `webapis.js` in
  the package).
- On a TV: [OWNER: `productInfo()`, a registered color key and Smart Hub's preview in the gallery's `.wgt`].

## Checklist

- [ ] I have read the [CONTRIBUTING](../CONTRIBUTING.md) document and followed the guidelines.
- [ ] Reviewed the changes in this PR and confirmed AI generated code (if any) is accurate.
- [ ] Passed `cargo test --workspace`, `cargo +nightly fmt --all` and clippy (including the Catalyst and Android runs for platform code).
- [ ] Changes to what apps use keep what released versions shipped and come with a case in `tests/compat/` (`cargo xtask compat`): #(pr/tizen/06-module)'s `tv/mod-tizen`.
- [ ] Ran the related `examples/gallery` demos on iOS and Android (on a Samsung TV instead).
