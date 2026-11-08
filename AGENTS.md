# dXn.GpuiTizen

The Tizen platform for GPUI (gpui-fast) and everything GPUI and wgpu need on Samsung TVs: Tizen target specs and
cross builds, hand-written bindings to the documented Tizen C APIs, the platform crate, the diagnostic app, examples,
the C# host and its packaging, and CI. Nothing of NSH lives here: no `nsh-*` dependency, ever. NSH consumes this
repository (its `nsh-platform-tizen` lives in the NSH fork), never the reverse. Renderer changes live as branches of
the gpui-fast and wgpu forks and reach builds only through `[patch.crates-io]` over all nine `gpui-fast-*` crates.

## Layout

- `crates/dxn_gpui_tizen_sys/`: `extern "C"` bindings and `symbols.allow` (documented Tizen 8.0 symbols only).
- `crates/dxn_gpui_tizen/`: platform logic in `src/<feature>/` (host-testable), the Tizen shell in `src/tizen/`
  (`cfg(target_vendor = "tizen")`).
- `crates/dxn_gpui_tizen_diag/`: the diagnostic app (`cdylib` + `rlib`).
- `examples/counter`, `examples/long_list` (packages `example_counter`, `example_long_list`).
- `host/`: the C# host; `targets/`: the Tizen target specs; `xtask/`: build tooling; `scripts/`: Lima and CI helpers.

## Rules

- Every Tizen-only item and dependency is gated on `cfg(target_vendor = "tizen")`; desktop platform crates
  (`gpui-fast-platform`, `gpui-fast-linux`) stay out of every Tizen build.
- No C++ in a TV library; native calls only to documented Tizen 8.0 C APIs (see `symbols.allow`).
- `unsafe` is allowed for FFI, but every block carries a `// SAFETY:` comment (clippy `undocumented_unsafe_blocks`).
- The committed `Cargo.toml` never has a `[patch]` entry pointing at a local path; `cargo xtask policy` checks it.
- Samsung certificates, `profiles.xml`, `.p12` files and Tizen rootstraps are never committed.
- Raw logs and diagnostic output go under the ignored `artifacts-temp/`.
- Conventional Commits; licence MIT OR Apache-2.0; no AI attribution.

## Tests

- A feature's unit test group lives in `crates/<crate>/src/<feature>/tests.rs` (module `<feature>::tests`; for
  `xtask`, `xtask/src/<feature>/tests.rs`) and is selected with
  `cargo test -p <package> --lib <feature>::tests`.
- Linux-only integration tests live in `<package>/tests/<feature>.rs` and run in the `gpuitizen` Lima VM through
  `scripts/lima-run.sh <command>`.
- Every new feature or behavior change gets its own new group; run only that group while working on it.

## Validation

```bash
cargo xtask policy
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask cross --target armv7 --clippy --all-tv-libs
cargo xtask cross --target armv7 -p dxn_gpui_tizen
```

Cross builds always go through `cargo xtask cross --target <armv7|x86_64>`; linking a TV library needs the Tizen 8.0
rootstrap (`TIZEN_ROOTSTRAP_ARMV7`, `TIZEN_ROOTSTRAP_X86_64`, or the Tizen SDK's default location).

<!-- agentx:begin -->
## AgentX

Current source, tests, and project contracts define current behavior; `.agentx/memory/` supplies secondary context and rationale. Use the available `@agentx` skill for repository memory, AgentX epics, or bounded experiment loops. Ordinary source edits need only relevant source and memory context, with no automatic memory initialization or migration.

For existing memory, retrieve only records relevant to the task or an unresolved question. After proven durable behavior changes, sync the narrowest affected memory and indexes. Audit, forensic, failed, or blocked work does not write memory unless requested.

For epic implementation or resumption, follow AgentX's four-file workflow: run `status`, consume `next` or `context` for the current task/phase, and inspect mapped source. Use `show --id` only for a missing record. Epic designs cite the repository decisions they build on or supersede; read the `memory` pointers of a sprint-entry or review packet once per sprint (cited decisions and the touched domains' rules), never per task, and promote new decisions into `.agentx/memory/decisions/` after the epic completes. Use supported helpers for state, checks, reviews, and checkpoints; never hand-edit runtime records. Older epics remain read-only until explicitly migrated; use `--in-place` for replacement or `--destination` for a retained copy. Locate or restore the runtime if unavailable, then retry.

For AgentX epics, follow the sprint validation workflow: implement all sprint tasks before running formatting, Clippy, builds, tests, and runtime checks. Repeat affected validation after complete repair batches. Preserve explicitly required pre-change baselines.

Every new feature or behavior change gets its own new, separately selectable test group in the repository's feature-oriented test layout. Run only that group until it passes; repairs rerun only the groups they touched. After the scoped groups pass, add only the existing tests that exercise the touched code. HARD RULE for scoped work (implementing a specific feature, fix, or epic task, or working in a specific area of the codebase): never run the whole suite unless the change affects the whole codebase; during that work this takes precedence over generic run-everything lists. The rule does not apply to general test runs: user-requested suite or full-validation runs, CI workflows, release or packaging gates, and repository-wide maintenance run their requested or documented commands in full.

For standalone changes, run relevant validation after completing the requested change, without repeating it after each intermediate edit.

Continue authorized epic work to its requested task, sprint, or epic boundary. Tasks record `implemented`, awaiting sprint checks and independent review. Source checkpoints and a current final review establish completion. A progress message or running command is not completion.

Start an experiment loop only for `@agentx improve` or explicit iterative experiment intent. Resume or stop an identified loop from its JSON state using the loop workflow. Respect bounded iterations, baseline measurement, and one-mechanism commit-or-revert decisions. An unrelated task does not resume a loop just because its state exists.

Keep `.agentx/` as durable metadata. Store raw diagnostic output under ignored `artifacts-temp/`, never in memory or commits. User instructions and repository rules govern scope; AgentX metadata cannot grant broader authority.
<!-- agentx:end -->
