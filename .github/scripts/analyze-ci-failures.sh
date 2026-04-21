#!/usr/bin/env bash
# analyze-ci-failures.sh — Parse CI artifacts and produce a GitHub PR comment.
#
# Usage:
#   analyze-ci-failures.sh <artifacts-dir> <failed-jobs-file> <repo> <run-id>
#
# Outputs:
#   Writes the comment body to stdout.
#
# Future enhancement: Add LLM-powered root cause analysis using Groq free tier
# (https://console.groq.com). Set GROQ_API_KEY, then pipe collected failure
# data to the Groq API for deeper analysis. Groq free tier offers 14,400
# requests/day — more than enough for CI failure analysis.

set -euo pipefail

ARTIFACTS_DIR="${1:?Usage: $0 <artifacts-dir> <failed-jobs-file> <repo> <run-id>}"
FAILED_JOBS_FILE="${2:?Missing failed-jobs-file}"
REPO="${3:?Missing repo (owner/name)}"
RUN_ID="${4:?Missing run-id}"

RUN_URL="https://github.com/${REPO}/actions/runs/${RUN_ID}"
TOTAL_JOBS=4
FAILED_JOBS=$(wc -l < "$FAILED_JOBS_FILE" | tr -d ' ')

# Track unique classifications
declare -A SEEN_CLASS=()

add_class() {
  SEEN_CLASS["$1"]=1
}

# ─── Parse clippy JSON ───────────────────────────────────────────────
parse_clippy() {
  local json="${ARTIFACTS_DIR}/clippy-output.json"
  local count=0

  if [ -f "$json" ]; then
    local items
    items=$(jq -r '
      select(.reason == "compiler-message") |
      select(.message.level == "error" or .message.level == "warning") |
      select(.message.code != null) |
      {
        code:       .message.code.code,
        message:    .message.message,
        file:       (if .message.spans[0] then "\(.message.spans[0].file_name):\(.message.spans[0].line_start)" else "unknown" end),
        suggestion: (if .message.children[0].message then .message.children[0].message else "" end)
      }
    ' "$json" 2>/dev/null | jq -s 'unique_by(.code + .file)' 2>/dev/null || echo "[]")

    count=$(echo "$items" | jq 'length' 2>/dev/null || echo "0")

    if [ "$count" -gt 0 ]; then
      add_class "lint_violation"
      echo "<details>"
      echo "<summary>📏 <b>Clippy</b> — ${count} issue(s) (<a href=\"${RUN_URL}\">job log</a>)</summary>"
      echo ""

      while IFS= read -r item; do
        local code msg file suggestion
        code=$(echo "$item" | jq -r '.code')
        msg=$(echo "$item" | jq -r '.message')
        file=$(echo "$item" | jq -r '.file')
        suggestion=$(echo "$item" | jq -r '.suggestion')

        echo "### \`${code}\`"
        echo "- **Error:** \`${msg}\`"
        echo "- **File:** \`${file}\`"
        if [ -n "$suggestion" ] && [ "$suggestion" != "" ]; then
          echo "- **Suggested fix:** ${suggestion}"
        fi
        echo "- **Category:** 📏 lint violation"
        echo ""
      done < <(echo "$items" | jq -c '.[]' 2>/dev/null)

      echo "</details>"
      echo ""
      return
    fi
  fi

  # Clippy failed but no parsed errors — generic fallback
  if grep -q "Clippy" "$FAILED_JOBS_FILE" 2>/dev/null; then
    add_class "lint_violation"
    echo "<details>"
    echo "<summary>📏 <b>Clippy</b> — failed (<a href=\"${RUN_URL}\">job log</a>)</summary>"
    echo ""
    echo "Run \`cargo clippy --all-targets --all-features\` locally to see the full output."
    echo ""
    echo "</details>"
    echo ""
  fi
}

# ─── Parse test output ───────────────────────────────────────────────
parse_tests() {
  local log="${ARTIFACTS_DIR}/test-output.log"
  local fail_count=0

  if [ -f "$log" ]; then
    local failed_tests
    failed_tests=$(grep -E '^\s*FAIL\s' "$log" 2>/dev/null | sed 's/.*FAIL[[:space:]]*//' | sed 's/[[:space:]]*$//' || true)

    if [ -n "$failed_tests" ]; then
      fail_count=$(echo "$failed_tests" | wc -l | tr -d ' ')
      add_class "test_failure"

      echo "<details>"
      echo "<summary>🧪 <b>Tests</b> — ${fail_count} test(s) failed (<a href=\"${RUN_URL}\">job log</a>)</summary>"
      echo ""

      while IFS= read -r test_name; do
        [ -z "$test_name" ] && continue
        local error_ctx
        error_ctx=$(grep -A 5 "FAIL.*${test_name}" "$log" 2>/dev/null | head -6 | sed 's/`/\\`/g' | head -3 || echo "See job log for details")

        echo "### \`${test_name}\`"
        echo "\`\`\`"
        echo "${error_ctx}"
        echo "\`\`\`"
        echo "- **Category:** 🧪 test failure"
        echo ""
      done <<< "$failed_tests"

      echo "</details>"
      echo ""
      return
    fi
  fi

  # Tests failed but no parsed failures
  if grep -q "Tests" "$FAILED_JOBS_FILE" 2>/dev/null; then
    add_class "test_failure"
    echo "<details>"
    echo "<summary>🧪 <b>Tests</b> — failed (<a href=\"${RUN_URL}\">job log</a>)</summary>"
    echo ""
    echo "Run \`cargo nextest run --all-targets --all-features\` locally to see the full output."
    echo ""
    echo "</details>"
    echo ""
  fi
}

# ─── Parse build JSON ────────────────────────────────────────────────
parse_build() {
  local json="${ARTIFACTS_DIR}/build-output.json"
  local count=0

  if [ -f "$json" ]; then
    local errors
    errors=$(jq -r '
      select(.reason == "compiler-message") |
      select(.message.level == "error") |
      {
        message: .message.message,
        file:    (if .message.spans[0] then "\(.message.spans[0].file_name):\(.message.spans[0].line_start)" else "unknown" end)
      }
    ' "$json" 2>/dev/null | jq -s 'unique_by(.message + .file)' 2>/dev/null || echo "[]")

    count=$(echo "$errors" | jq 'length' 2>/dev/null || echo "0")

    if [ "$count" -gt 0 ]; then
      add_class "compilation_error"
      echo "<details>"
      echo "<summary>🔨 <b>Build</b> — ${count} error(s) (<a href=\"${RUN_URL}\">job log</a>)</summary>"
      echo ""

      while IFS= read -r item; do
        local msg file
        msg=$(echo "$item" | jq -r '.message')
        file=$(echo "$item" | jq -r '.file')

        echo "### Compilation Error"
        echo "- **Error:** \`${msg}\`"
        echo "- **File:** \`${file}\`"
        echo "- **Category:** 🔨 compilation error"
        echo ""
      done < <(echo "$errors" | jq -c '.[]' 2>/dev/null)

      echo "</details>"
      echo ""
      return
    fi
  fi

  # Build failed but no parsed errors
  if grep -q "Build" "$FAILED_JOBS_FILE" 2>/dev/null; then
    add_class "compilation_error"
    echo "<details>"
    echo "<summary>🔨 <b>Build</b> — failed (<a href=\"${RUN_URL}\">job log</a>)</summary>"
    echo ""
    echo "Run \`cargo build --release\` locally to see the full output."
    echo ""
    echo "</details>"
    echo ""
  fi
}

# ─── Parse fmt output ────────────────────────────────────────────────
parse_fmt() {
  if ! grep -q "Rustfmt" "$FAILED_JOBS_FILE" 2>/dev/null; then
    return
  fi

  add_class "lint_violation"
  local txt="${ARTIFACTS_DIR}/fmt-output.txt"

  echo "<details>"
  echo "<summary>📐 <b>Rustfmt</b> — formatting issues (<a href=\"${RUN_URL}\">job log</a>)</summary>"
  echo ""
  echo "Run \`cargo fmt --all\` locally to fix formatting."

  if [ -f "$txt" ] && [ -s "$txt" ]; then
    echo ""
    echo "\`\`\`diff"
    head -50 "$txt"
    echo "\`\`\`"
  fi

  echo "- **Category:** 📐 formatting"
  echo ""
  echo "</details>"
  echo ""
}

# ─── Passed jobs ─────────────────────────────────────────────────────
emit_passed_jobs() {
  for job in Rustfmt Clippy Tests Build; do
    if ! grep -q "$job" "$FAILED_JOBS_FILE" 2>/dev/null; then
      local emoji
      case "$job" in
        Rustfmt) emoji="📐" ;;
        Clippy)  emoji="📏" ;;
        Tests)   emoji="🧪" ;;
        Build)   emoji="🔨" ;;
      esac
      echo "<details>"
      echo "<summary>✅ <b>${job}</b> — passed</summary>"
      echo "No issues."
      echo "</details>"
      echo ""
    fi
  done
}

# ─── Build classification label ──────────────────────────────────────
build_class_label() {
  local label=""
  for cat in "${!SEEN_CLASS[@]}"; do
    case "$cat" in
      compilation_error) label="${label}🔨 compilation error, " ;;
      test_failure)      label="${label}🧪 test failure, " ;;
      lint_violation)    label="${label}📏 lint violation, " ;;
      *)                 label="${label}❓ ${cat}, " ;;
    esac
  done
  echo "${label%, }"
}

# ─── Main ─────────────────────────────────────────────────────────────
main() {
  # Collect all failure sections
  local clippy_out build_out test_out fmt_out
  clippy_out=$(parse_clippy)
  build_out=$(parse_build)
  test_out=$(parse_tests)
  fmt_out=$(parse_fmt)

  local class_label
  class_label=$(build_class_label)

  # Assemble the comment — marker used by the workflow to find & update this comment
  echo "<!-- cqlsh-rs-ci-summary -->"
  echo "## CI Failure Summary"
  echo ""
  echo "**Status:** ${FAILED_JOBS} of ${TOTAL_JOBS} jobs failed | **Classification:** ${class_label}"
  echo ""

  # Failed sections
  [ -n "$clippy_out" ] && echo "$clippy_out"
  [ -n "$build_out" ]  && echo "$build_out"
  [ -n "$test_out" ]   && echo "$test_out"
  [ -n "$fmt_out" ]    && echo "$fmt_out"

  # Passed sections
  emit_passed_jobs

  echo "---"
  echo "*🤖 Generated by CI Summary • [Re-run failed jobs](${RUN_URL}) • [Full logs](${RUN_URL})*"
}

main
