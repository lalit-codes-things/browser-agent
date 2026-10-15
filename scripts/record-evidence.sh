#!/usr/bin/env bash
#
# Evidence recorder (Section 20.4, [BA-TRACE-002], Section 21).
#
# Runs the checks that can run on this machine and writes a record containing the
# commit, the working-tree state, the platform, the exact commands, and the real
# outcome of each one. It exists so a claim in the requirements registry can point
# at a produced artifact instead of at a promise.
#
# It never records a proof it did not run. Proofs that need the native CEF host,
# a second platform or the model runtime are listed as NOT RUN with the reason, so
# a reader can see the gap rather than infer success from silence.
#
# Usage: scripts/record-evidence.sh [output-directory]

set -euo pipefail

cd "$(dirname "$0")/.."

out_dir="${1:-docs/evidence}"
mkdir -p "$out_dir"

commit=$(git rev-parse HEAD 2>/dev/null || echo "no-git")
short_commit=${commit:0:12}
platform=$(uname -sm | tr ' ' '-')

if [ "$(uname -s)" = "Darwin" ]; then
  os_version="macOS $(sw_vers -productVersion 2>/dev/null || echo unknown)"
else
  os_version="$(uname -sr)"
fi

if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
  dirty="true"
else
  dirty="false"
fi
work_tree_digest=$(git diff HEAD 2>/dev/null | shasum -a 256 | awk '{print $1}')
untracked=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l | tr -d ' ')

# A commit SHA alone does not identify the code when the tree carries untracked
# or modified sources, so the record also carries a digest over the content of
# every tracked and untracked-not-ignored file that was actually tested.
content_manifest=$(git ls-files -o -c --exclude-standard 2>/dev/null | LC_ALL=C sort | while IFS= read -r file; do
  if [ -f "$file" ]; then
    printf '%s  %s\n' "$(shasum -a 256 "$file" | awk '{print $1}')" "$file"
  fi
done)
content_digest=$(printf '%s\n' "$content_manifest" | shasum -a 256 | awk '{print $1}')
content_files=$(printf '%s\n' "$content_manifest" | grep -c . || true)

timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
stamp=$(date -u +"%Y%m%dT%H%M%SZ")
run_dir="$out_dir/${stamp}-${short_commit}-${platform}"
logs_dir="$run_dir/logs"
mkdir -p "$logs_dir"
md="$run_dir/record.md"
json="$run_dir/record.json"

results_dir=$(mktemp -d)
trap 'rm -rf "$results_dir"' EXIT

# Each entry is "id|command". The command runs unchanged; its exit status is the
# status recorded, and the output is kept as the proof artifact for that check.
checks=(
  "rust-format|cargo fmt --all -- --check"
  "rust-clippy|cargo clippy --workspace --all-targets -- -D warnings"
  "rust-tests|cargo test --workspace"
  "core-binary|cargo build -p ba-runtime --bin ba-core"
  "ui-format|npm run format:check"
  "ui-lint|npm run lint"
  "ui-types|npm run typecheck"
  "ui-tests|npm test"
  "ui-build|npm run build"
  "registry|npm run registry:check"
  "supply-chain|./scripts/supply-chain.sh"
)

rows=()
failed=0
for entry in "${checks[@]}"; do
  id=${entry%%|*}
  command=${entry#*|}
  log="$logs_dir/$id.log"
  set +e
  bash -c "$command" >"$log" 2>&1
  status=$?
  set -e
  if [ $status -eq 0 ]; then
    outcome="passed"
  else
    outcome="failed"
    failed=$((failed + 1))
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$id" "$command" "$outcome" "$status" "logs/$id.log" >>"$results_dir/summary.tsv"
  rows+=("| \`$id\` | \`$command\` | $outcome | $status | \`logs/$id.log\` |")
  echo "evidence: $id -> $outcome (exit $status)"
done

{
  echo "# Execution evidence"
  echo
  echo "- Recorded: $timestamp"
  echo "- Commit: \`$commit\`"
  echo "- Working tree: $([ "$dirty" = "true" ] && echo "modified (digest of tracked changes \`$work_tree_digest\`, $untracked untracked file(s))" || echo "clean")"
  echo "- Tested content digest: \`$content_digest\` over $content_files tracked and untracked source files"
  echo "- Platform: $platform"
  echo "- OS: $os_version"
  echo "- rustc: $(rustc --version 2>/dev/null || echo unavailable)"
  echo "- cargo: $(cargo --version 2>/dev/null || echo unavailable)"
  echo "- node: $(node --version 2>/dev/null || echo unavailable)"
  echo "- npm: $(npm --version 2>/dev/null || echo unavailable)"
  echo
  echo "## Checks that ran"
  echo
  echo "| Check | Command | Outcome | Exit | Output |"
  echo "| --- | --- | --- | --- | --- |"
  printf '%s\n' "${rows[@]}"
  echo
  echo "A \`passed\` row means the command exited 0 on this platform and commit. The"
  echo "complete output of every check is kept in \`logs/\` beside this file, so a"
  echo "reader can inspect what actually ran rather than trusting the summary."
  echo
  echo "## Proofs not run"
  echo
  echo "| Proof | Required by | Why it is absent |"
  echo "| --- | --- | --- |"
  echo "| CEF native view composition, focus and input routing | [BA-BROWSER-001], [BA-CEF-001], Phase 0/1 | The native CEF host does not exist in this checkout yet (ADR-010). No page is rendered, and the interface reports that absence instead of simulating one. |"
  echo "| Window resize, multiple monitors, high-DPI, full-screen behavior | [BA-BROWSER-001] | Requires the native host and a display; cannot be exercised by the automated suites here. |"
  echo "| Windows x64 results | Section 20.3, Section 21 | This record was produced on $platform only. Windows evidence must be recorded separately on that platform. |"
  echo "| Packaged macOS/Windows installers, signing and notarization | Phase 8, Section 20.3 | No packaging pipeline exists yet. |"
  echo "| Local model install and inference | Section 16, Phase 5 | No model artifact is registered in this build and no runner is integrated, so readiness stays \`NOT_INSTALLED\`; no progress is simulated. |"
  echo "| CDP-disabled production inspection | Section 11.5 | There is no production browser binary to inspect yet. |"
  echo
  echo "A check that passed here is not evidence for any row above: absent evidence"
  echo "stays absent until it is produced, outside this document."
} >"$md"

{
  echo "{"
  echo "  \"recorded_at\": \"$timestamp\","
  echo "  \"commit\": \"$commit\","
  echo "  \"work_tree_dirty\": $dirty,"
  echo "  \"work_tree_digest\": \"$work_tree_digest\","
  echo "  \"content_digest\": \"$content_digest\","
  echo "  \"content_files\": $content_files,"
  echo "  \"untracked_files\": $untracked,"
  echo "  \"platform\": \"$platform\","
  echo "  \"os_version\": \"$os_version\","
  echo "  \"toolchain\": {"
  echo "    \"rustc\": \"$(rustc --version 2>/dev/null | tr -d '"')\","
  echo "    \"cargo\": \"$(cargo --version 2>/dev/null | tr -d '"')\","
  echo "    \"node\": \"$(node --version 2>/dev/null)\","
  echo "    \"npm\": \"$(npm --version 2>/dev/null)\""
  echo "  },"
  echo "  \"checks\": ["
  first=1
  while IFS=$'\t' read -r id command outcome status log; do
    if [ $first -eq 0 ]; then echo ","; fi
    first=0
    escaped=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1]))" "$command")
    printf '    {"id": "%s", "command": %s, "outcome": "%s", "exit_code": %s, "log": "%s"}' "$id" "$escaped" "$outcome" "$status" "$log"
  done <"$results_dir/summary.tsv"
  echo
  echo "  ],"
  echo "  \"not_run\": ["
  echo "    \"CEF native view composition, focus and input routing ([BA-BROWSER-001], [BA-CEF-001])\","
  echo "    \"window resize, multiple monitors, high-DPI, full-screen ([BA-BROWSER-001])\","
  echo "    \"Windows x64 results (Section 20.3, Section 21)\","
  echo "    \"packaged installers, signing and notarization (Phase 8)\","
  echo "    \"local model install and inference (Section 16, Phase 5)\","
  echo "    \"CDP-disabled production inspection (Section 11.5)\""
  echo "  ]"
  echo "}"
} >"$json"

echo "evidence: wrote $md"
echo "evidence: wrote $json"

if [ $failed -gt 0 ]; then
  echo "evidence: $failed check(s) failed; the record states which" >&2
  exit 1
fi
