---
agentx:
  kind: decision
  scope: decisions/repository
  status: current
  created: 2026-11-08T15:00:00+01:00
  updated: 2026-11-08T15:00:00+01:00
  topics:
    - agentx-memory
    - taxonomy
  supersedes: []
  superseded_by: []
---

## Decision: Start dXn.GpuiTizen's AgentX memory with only a `repository` governance domain

### Context: AgentX memory was initialized on 2026-11-08, when the repository had an unborn `main` branch with no commits, manifests, source, docs, tests, or CI. The AgentX taxonomy rules require domains derived from repository evidence and forbid placeholder domains.

### Alternatives considered: Guess architecture domains from the repository name (for example `gpui`, `tizen`, `platform`, `rendering`), copy a sibling dXn repository's taxonomy, or defer initialization until source exists.

### Reasoning: The only evidence-backed boundary is repository governance, which every AgentX tree needs to hold its taxonomy decision. A `repository` domain matches the sibling dXn repositories (dXn.Reactive, dXn.Dtcg), so cross-repository navigation stays familiar. Guessed domains would be empty placeholders and would likely be renamed once real crate or module names exist. Initializing now still installs the AgentX block in `AGENTS.md` and the routing indexes for future work.

### Trade-offs accepted: Excluded categories: any architecture, platform, rendering, UI-component, build, or release domain, because no source, manifest, or CI exists yet. The first change that introduces source must add matching domains and a decision that builds on this one. The project's intent is recorded only as a hypothesis in `knowledge/repository/hypotheses.md`.

### Supersedes: None.
