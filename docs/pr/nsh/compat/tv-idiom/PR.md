# compat: Run a script on a TV with `platform <os> tv`

NSH is about to make TVs a category of their own (longbridge/nsh#(pr/tv/02-category)): `platform.tv`, `tv={{…}}`
props on every element and a `tv` key in `platform.select`, on any OS. Its compat cases run in a new `tv` domain whose
script starts with `platform android tv`, which the checker refuses today.

The `platform` step now takes `tv` as its second word, beside `desktop` and `terminal`: the device is a TV
(`Idiom::Tv`) on the OS the first word names. `prop` steps take a `tv.` prefix as they take `ios.` or `desktop.`.

It uses only what NSH's `main` already has (`Idiom::Tv`), so it builds against NSH checkouts with or without the
category and can merge on its own.

## How to test

- `cargo test` against NSH's `main`: the existing scripts are unchanged.
- `cargo xtask compat` in an NSH checkout of longbridge/nsh#(pr/tv/02-category): the `tv` domain (`platform android
  tv`) passes: `mod-device-tv` reports idiom `tv`, `platform.tv` true and desktop false; `props-platform-tv` applies the
  `tv={{…}}` props and not the `desktop={{…}}` ones.
