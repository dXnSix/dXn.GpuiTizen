---
agentx:
  kind: rules
  scope: knowledge/repository
  status: current
  created: 2026-11-08T15:00:00+01:00
  updated: 2026-11-08T15:00:00+01:00
  topics:
    - taxonomy
    - validation
  supersedes: []
  superseded_by: []
---

# Repository Rules

- Add a knowledge domain (and its decision) only in the change that introduces the source it describes; derive its name from the package, crate, or module name the source uses ([taxonomy decision](../../decisions/repository/2026-11-08-repository-agentx-memory-taxonomy.md)).
- When the first manifests and test layout land, document in `AGENTS.md` where a new feature's test group goes and how one group is selected, so the AgentX scoped-test rule is actionable.
