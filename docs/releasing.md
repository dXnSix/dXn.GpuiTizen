# Releasing `dxn_gpui_tizen_sys` and `dxn_gpui_tizen`

A release publishes the two library crates to crates.io, tags `<crate>-vX.Y.Z` on `main` and creates a GitHub
Release of the same name. Nothing else in the workspace is published: the diagnostic app, the examples and `xtask`
inherit `publish = false` and release-plz releases none of them (`release = false`).

Every member takes its version from `[workspace.package]`, so a release bump moves the whole workspace: both crates
always carry the same version, and so do the apps' TPKs (`cargo xtask package` reads that version). A store
submission's version therefore follows the crates'; bump it by hand only between releases.

**release-plz** does the work (`release-plz.toml`, `.github/workflows/release-plz.yml`), as in dXn.Reactive, but
with crates.io publication on. On every push to `main` it

- keeps a release pull request with the next versions and their `CHANGELOG.md` sections
  (`crates/<crate>/CHANGELOG.md`), generated from Conventional Commits, and
- when a crate's version is not on crates.io yet, publishes it, tags it and creates the GitHub Release with the
  changelog section as notes.

| Commit | Release |
| --- | --- |
| `fix:`, `perf:` | patch |
| `feat:` | minor (`features_always_increment_minor`; a breaking change, which `semver_check` reports, is minor too on 0.x) |
| `docs:`, `test:`, `ci:`, `build:`, `refactor:`, `chore:` | none on their own; listed in the next release |

`chore(agentx)`, `chore(release)` and `chore(deps)` commits are left out of the changelogs.

## Before the first release

The workflow's jobs stay off until the repository variable `PUBLISH_CRATES` is `true`. The first publish waits for
gpui-fast's release of the renderer changes this platform needs (GL ES 3.0 devices, partial redraws, the external
target, holes): a published crate resolves `gpui-fast` from crates.io and cannot carry the `[patch]` lines local and
CI builds use. When that release is out:

1. Move the workspace's `gpui` and `gpui_wgpu` pins to it (and NSH's with them), and drop the git
   `[patch.crates-io]` lines from `Cargo.toml` (the wgpu ones too once wgpu releases the EGL 1.4 and program-binary
   fixes; until then they stay, and the crates cannot be published).
2. Check that `cargo package -p dxn_gpui_tizen_sys -p dxn_gpui_tizen --no-verify --allow-dirty` still succeeds
   (it runs against crates.io dependencies only), and that `cargo publish --dry-run` builds.
3. Add the secrets `RELEASE_PLZ_TOKEN` (a fine-grained token with contents and pull-request write access) and
   `CARGO_REGISTRY_TOKEN` (crates.io), then set `PUBLISH_CRATES` to `true`. The next push to `main` publishes 0.1.0.

These steps, the tokens and the crates.io owners are the owner's.
