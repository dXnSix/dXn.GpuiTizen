# [gles] Implement PipelineCache with program binaries

## Connections

No issue. We run GPUI (through [gpui-fast](https://github.com/longbridge/gpui-fast)) on Samsung Tizen TVs, which draw
with GL ES 3.0. GPUI already passes a `wgpu::PipelineCache` when the adapter has `Features::PIPELINE_CACHE`, but GL
never reports it, so every start compiles all shaders again on the main thread, which is slow on TV hardware.

## Description

On GL the pipeline cache is an empty struct today: `create_pipeline_cache` ignores the data,
`pipeline_cache_get_data` returns `None`, and the feature is never reported. GL ES 3.0 and GL 4.1 (or
`GL_ARB_get_program_binary`) can save a linked program with `glGetProgramBinary` and load it back with
`glProgramBinary`. This PR uses that to back the cache.

- **Feature.** The adapter reports `Features::PIPELINE_CACHE` when the context has program binaries and
  `GL_NUM_PROGRAM_BINARY_FORMATS > 0`. Native only; WebGL has no program binaries.
- **Keys.** A program is keyed by a stable 128-bit hash (FNV-1a, not `std`'s seeded hasher) of its GLSL sources, in
  stage order and with their lengths, plus the adapter's `GL_RENDERER` and `GL_VERSION`. The GLSL already reflects the
  naga options, the binding layout and multiview, so changing any of them changes the key.
- **Loading.** `create_program` looks the key up in the cache passed to the pipeline. If there is a binary, it calls
  `glProgramBinary` and checks the link status. If the driver rejects it (after a driver update, for example), the
  entry is dropped and the program is compiled as before, and the new binary replaces it. The uniform block and
  sampler bindings that GL sets after linking are also set on a loaded program.
- **Storing.** Every program a pipeline uses is added to the cache passed to it, if missing, including one that an
  earlier pipeline linked and the existing program cache shares.
- **Data format** (`wgpu-hal/src/gles/pipeline_cache.rs`): a magic, a version and the adapter key, then entries of
  key, binary format, length, binary and checksum. A different magic, version or adapter starts an empty cache, a bad
  checksum drops that entry and a truncated entry ends the list. Bad data costs a compile, never an error.
- **wgpu-core.** `adapter_key` gets a GL case. GL has no device id, so the key hashes the renderer name and driver info
  (`GL_VERSION`) with the vendor, and a driver update invalidates the cache before the backend looks at it.
- **wgpu.** `util::pipeline_cache_key` returns `wgpu_pipeline_cache_gl_<renderer>` for GL, with non-alphanumeric
  characters replaced and the name cut to 64 characters.

To key a program before compiling anything, `create_shader` becomes `shader_source`, which returns the GLSL; shaders
are compiled only when no binary was loaded. No new public API.

## Testing

- `gles::pipeline_cache::tests` (`cargo test -p wgpu-hal --features gles --lib`, 9 passed): keys depend on the
  sources, their order and the adapter; the data round-trips; another magic, version or adapter, a bad checksum or a
  truncated entry falls back.
- `pipeline_cache::tests::gl_adapters_are_keyed_by_renderer_and_version` in wgpu-core
  (`cargo test -p wgpu-core --lib pipeline_cache`, 13 passed).
- A new GPU test, `pipeline_cache_gl_program_binaries`, checks the loading by what the pipeline computes rather than
  by counters:
  - a cache holding a different shader's binary under a pipeline's key makes that pipeline compute the other
    shader's values, so the program was loaded, not compiled;
  - a binary the driver rejects leads to a compile, and the fresh binary replaces it;
  - an entry with a bad checksum is dropped and the program compiled;
  - a program an earlier pipeline linked is stored in a cache passed to a later pipeline.
- `cargo test -p wgpu-test --test wgpu-gpu -- pipeline_cache --test-threads=1` on Mesa llvmpipe (GL 4.5, Mesa 25.2.8,
  aarch64 Linux): `pipeline_cache` and `pipeline_cache_gl_program_binaries` passed.
- `cargo clippy -p wgpu-hal -p wgpu-core -p wgpu -p wgpu-test --features wgpu-hal/gles --all-targets -- -D warnings`
  on both commits, and `cargo fmt --check`.
- [OWNER: say here whether you ran it on a GL ES driver (a TV) from this branch; if not, say so plainly.]

## Squash or Rebase?

Squash. The first commit adds test-only counters on `wgpu_hal::gles::Device`, and the second replaces them with the
behavioral test above.

## LLM Use?

[OWNER: rewrite in your own words and keep only what is true.] I used an AI coding agent (Claude Code) for most of
this PR. It wrote the code and the tests from my direction, first on our v30 fork for the TV build and then ported
to `trunk`, and it drafted this description. I reviewed the diff, including the data format, the fallback paths and
the GPU test, and rewrote the description. I understand every change and will answer review comments myself.

<!--
Thanks for filing! Reviewers are assigned for non-draft PRs in the weekly wgpu maintainers meetings.

After you get a review and have addressed any comments, please explicitly re-request a review from the
person(s) who reviewed your changes. This will make sure it gets re-added to their review queue - you're not bothering us!
-->

## Checklist

<!-- Note that checking all the boxes is not necessary to open a PR. -->

- [ ] I self-reviewed and fully understand this PR.
- [x] `CHANGELOG.md` entries for the user-facing effects of this change are present. <!-- See instructions at the top of `CHANGELOG.md`. -->
- [x] The PR is minimal, and doesn't make sense to land as multiple PRs.
- [ ] Commits are logically scoped and individually reviewable.
- [x] The PR description has enough context to understand the motivation and solution implemented.
- [ ] (If applicable) WebGPU implementations built with `wgpu` may be affected behaviorally.
- [x] (If applicable) Validation and feature gates are in place to confine behavioral changes.
- [x] (If applicable) Tests demonstrate the validation and altered logic works. <!-- See `docs/testing.md` -->

<!-- Agentic agents creating a PR, please add the following to the list and fill it out accurately. Checking this does not automatically invalidate the PR.
- [ ] This PR was automatically created by an Agent.
- [ ] The description was automatically written by an Agent. [OWNER: tick this if you keep the draft mostly as it is]
      -->
