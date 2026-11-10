#!/usr/bin/env bash
# Tests plan.sh for manual, pull-request and schedule events; prints `plan tests: <n> passed`.
# Needs bash, git and jq. Run from anywhere: bash .github/actions/plan/plan_test.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
plan="$here/plan.sh"
passed=0
failed=0

# A scratch repository with a base commit, for the pull-request cases that diff BASE..HEAD.
repo="$(mktemp -d)"
trap 'rm -rf "$repo"' EXIT
git -C "$repo" init -q
git -C "$repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m base
base="$(git -C "$repo" rev-parse HEAD)"

# Commits the given files on top of the base and prints the head.
change() {
  git -C "$repo" checkout -q --detach "$base"
  for file in "$@"; do
    mkdir -p "$repo/$(dirname "$file")"
    echo x >"$repo/$file"
    git -C "$repo" add "$file"
  done
  git -C "$repo" -c user.name=t -c user.email=t@t commit -q -m change
  git -C "$repo" rev-parse HEAD
}

# Runs plan.sh with the given environment (NAME=value ...) and prints its outputs.
run() {
  (cd "$repo" && env -i PATH="$PATH" HOME="$HOME" "$@" bash "$plan" 2>/dev/null)
}

expect() { # description, expected output line, then the environment
  local name=$1 want=$2
  shift 2
  local out
  if out="$(run "$@")" && grep -qxF "$want" <<<"$out"; then
    passed=$((passed + 1))
  else
    failed=$((failed + 1))
    echo "FAIL: $name: wanted '$want', got:" >&2
    echo "$out" | sed 's/^/  /' >&2
  fi
}

expect_failure() { # description, then the environment
  local name=$1
  shift
  if run "$@" >/dev/null; then
    failed=$((failed + 1))
    echo "FAIL: $name: plan.sh succeeded" >&2
  else
    passed=$((passed + 1))
  fi
}

all='jobs=["check","test","host","cross","gles","egl"]'
both='os=["ubuntu-24.04","macos-15"]'

# Manual runs: the inputs choose.
expect "manual default jobs" "$all" EVENT_NAME=workflow_dispatch DISPATCH_JOBS=all DISPATCH_PLATFORMS=all
expect "manual default platforms" "$both" EVENT_NAME=workflow_dispatch DISPATCH_JOBS=all DISPATCH_PLATFORMS=all
expect "manual jobs" 'jobs=["cross","gles"]' EVENT_NAME=workflow_dispatch DISPATCH_JOBS="cross, gles" DISPATCH_PLATFORMS=all
expect "manual platform" 'os=["macos-15"]' EVENT_NAME=workflow_dispatch DISPATCH_JOBS=check DISPATCH_PLATFORMS=macos
expect "manual grep" 'grep=release::tests' EVENT_NAME=workflow_dispatch DISPATCH_JOBS=test DISPATCH_GREP=release::tests
expect "manual one platform" 'os=["ubuntu-24.04"]' EVENT_NAME=workflow_dispatch DISPATCH_PLATFORMS=ubuntu
expect_failure "manual unknown job" EVENT_NAME=workflow_dispatch DISPATCH_JOBS=lane-a
expect_failure "manual unknown platform" EVENT_NAME=workflow_dispatch DISPATCH_PLATFORMS=windows

# Pull requests: labels choose; without a `ci:` label, documentation-only changes run `check`.
code="$(change crates/dxn_gpui_tizen/src/lib.rs)"
expect "pr code change runs everything" "$all" EVENT_NAME=pull_request PR_LABELS='[]' BASE_SHA="$base" HEAD_SHA="$code"
docs="$(change docs/setup.md README.md crates/dxn_gpui_tizen/README.md .agentx/x.json)"
expect "pr docs-only runs check" 'jobs=["check"]' EVENT_NAME=pull_request PR_LABELS='[]' BASE_SHA="$base" HEAD_SHA="$docs"
store="$(change store/feature-description.video.md)"
expect "pr store templates are not docs" "$all" EVENT_NAME=pull_request PR_LABELS='[]' BASE_SHA="$base" HEAD_SHA="$store"
expect "pr job labels add up" 'jobs=["cross","egl"]' EVENT_NAME=pull_request PR_LABELS='["ci:cross","bug","ci:egl"]' BASE_SHA="$base" HEAD_SHA="$docs"
expect "pr ci:all" "$all" EVENT_NAME=pull_request PR_LABELS='["ci:all"]' BASE_SHA="$base" HEAD_SHA="$docs"
expect "pr os label" 'os=["ubuntu-24.04"]' EVENT_NAME=pull_request PR_LABELS='["ci-os:ubuntu"]' BASE_SHA="$base" HEAD_SHA="$code"
expect_failure "pr unknown job label" EVENT_NAME=pull_request PR_LABELS='["ci:lane-b"]' BASE_SHA="$base" HEAD_SHA="$code"

# Schedule: everything, on every platform.
expect "schedule jobs" "$all" EVENT_NAME=schedule
expect "schedule platforms" "$both" EVENT_NAME=schedule
expect "schedule has no grep" 'grep=' EVENT_NAME=schedule

if [ "$failed" -eq 0 ]; then
  echo "plan tests: $passed passed"
else
  echo "plan tests: $passed passed, $failed failed"
  exit 1
fi
