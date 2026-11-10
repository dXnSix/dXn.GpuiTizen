# compat: Run a script as a native Tizen app with `platform tizen`

NSH is about to report Samsung's TV OS as an OS of its own (longbridge/nsh#(pr/tizen/01-os)): `Os::Tizen`,
`platform.os` `"tizen"`, a TV by default and never a desktop. Its compat cases run in a new `tizen` domain whose
script starts with `platform tizen`.

- `platform tizen` runs a script on `Os::Tizen`, in the TV idiom unless the script says otherwise: with no second
  word the idiom is the OS's own (`Idiom::default_on`: a desktop OS's desktop, Tizen's TV, else a phone).
- `build.rs` reads the checkout's `platform.rs` for the `Tizen` variant and sets `nsh_has_tizen`, so the checker still
  builds against NSH without `Os::Tizen` (and keeps today's rule for a missing second word there).

Builds on #(tv-idiom) (`platform <os> tv`), merged.

## How to test

- `cargo test` against NSH's `main`: the gate is off and nothing changes.
- `cargo xtask compat` in an NSH checkout of longbridge/nsh#(pr/tizen/01-os): the `tizen` domain's `mod-device-tizen`
  reports OS `tizen`, idiom `tv`, `platform.tv` true and desktop false, and `platform.select` takes `tizen` before `tv`.
