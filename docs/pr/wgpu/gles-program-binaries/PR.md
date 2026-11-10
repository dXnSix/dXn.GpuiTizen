# [gles] Implement PipelineCache with program binaries

## Connections

None upstream. Our use: GPUI ([gpui-fast](https://github.com/longbridge/gpui-fast)) on Samsung Tizen TVs, which
draw through GL ES 3.0. GPUI passes a `wgpu::PipelineCache` when the adapter has `Features::PIPELINE_CACHE`; on GL
that feature does not exist yet, so every start compiles all of its shaders on the main thread.

## Description

The GLES backend's `PipelineCache` is an empty struct today. `create_pipeline_cache` ignores the data,
`pipeline_cache_get_data` returns `None`, and GL never reports `Features::PIPELINE_CACHE`. GL ES 3.0 and GL 4.1
(and `GL_ARB_get_program_binary`) can save a linked program with `glGetProgramBinary` and restore it with
`glProgramBinary`. This PR uses that to back the cache.

- **Feature.** The adapter reports `Features::PIPELINE_CACHE` when the context has program binaries and
  `GL_NUM_PROGRAM_BINARY_FORMATS > 0`. Native only: WebGL has no program binaries.
- **Keys.** A program's key is a stable 128-bit hash (FNV-1a, not `std`'s seeded hasher) of its GLSL sources, in
  stage order, with lengths, and of the adapter's `GL_RENDERER` and `GL_VERSION`. The generated GLSL reflects the
  naga options, the binding layout and multiview, so a change to any of them changes the key. The uniform-block
  and sampler bindings that GL sets after linking are set on a loaded program as well.
- **Loading.** `create_program` looks the key up in the cache passed to the pipeline. If a binary is there, it calls
  `glProgramBinary` and checks the link status. If the driver rejects the binary (a driver update, for example),
  the entry is removed and the program is compiled as before. The fresh binary then takes its place.
- **Storing.** Every program a pipeline uses goes into the cache passed to it, if the cache does not have it yet.
  This includes a program an earlier pipeline linked, which the existing program cache shares.
  `pipeline_cache_get_data` encodes the entries.
- **Data** (`wgpu-hal/src/gles/pipeline_cache.rs`): a magic, a format version and the adapter key, then entries of
  key, binary format, length, binary and a checksum. If the magic, version or adapter key is different, the cache
  starts empty. An entry with a bad checksum is dropped. A truncated entry ends the list. Stale or corrupt data
  costs a compile and never causes an error.
- **wgpu-core.** `adapter_key` gets a GL case. GL reports no device id, so the key is a hash of the renderer name
  and the driver info (`GL_VERSION`) with the vendor. A driver update therefore invalidates the cache in wgpu-core
  before the backend checks it.
- **wgpu.** `util::pipeline_cache_key` returns `wgpu_pipeline_cache_gl_<renderer>` for GL, with the renderer's
  non-alphanumeric characters replaced and the name cut to 64 characters.

`create_shader` becomes `shader_source`, which returns the GLSL instead of compiling it, so the program can be
keyed before anything is compiled. Shaders are compiled only when no binary was loaded.

No new public API.

## Testing

- `gles::pipeline_cache::tests` (`cargo test -p wgpu-hal --features gles --lib`): keys depend on the sources, their
  order and the adapter; data round-trips; data with another magic, version or adapter, a bad checksum or a
  truncated entry falls back. 6 passed on Linux.
- `pipeline_cache::tests::gl_adapters_are_keyed_by_renderer_and_version` in wgpu-core
  (`cargo test -p wgpu-core --lib pipeline_cache`): 13 passed.
- A new GPU test, `pipeline_cache_gl_program_binaries` (`tests/tests/wgpu-gpu/pipeline_cache.rs`), proves the
  loading by what the pipeline computes, not by counters:
  - A cache holds, under a pipeline's program key, the binary of a different shader. The pipeline created with it
    computes that other shader's values, so the program was loaded and not compiled.
  - A binary the driver rejects (garbage behind a valid entry) leads to a compile, and the fresh binary replaces it
    in the cache.
  - Data with a bad entry checksum is dropped, and the program is compiled.
  - A program that an earlier pipeline linked is stored in a cache passed to a later pipeline.
- `cargo test -p wgpu-test --test wgpu-gpu -- pipeline_cache --test-threads=1` after `generate_gpuconfig_report`,
  on Mesa llvmpipe (GL 4.5, Mesa 25.2.8, aarch64 Linux): `pipeline_cache` and `pipeline_cache_gl_program_binaries`
  passed.
- `cargo clippy -p wgpu-hal -p wgpu-core -p wgpu -p wgpu-test --features wgpu-hal/gles --all-targets -- -D warnings`
  passed on both commits. `cargo fmt --check` passed.
- Not run on a GL ES driver from this branch. _(Owner: add the TV run here if there is one.)_

## Squash or Rebase?

Squash. The first commit adds test-only counters on `wgpu_hal::gles::Device`, and the second replaces them with the
behavioral test above.

## LLM Use?

_(The owner writes this section.)_

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
- [ ] The description was automatically written by an Agent.
      -->
