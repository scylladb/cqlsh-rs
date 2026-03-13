# Sub-Plan SP15: AI-Powered CI Failure Summaries

> Parent: [high-level-design.md](high-level-design.md) | Phase: 5 (CI/CD)

## Objective

Automatically analyze CI failures, classify them, and post a collapsed PR comment with structured diagnostics, links to failing jobs, suggested fixes, and deduplication of recurring issues — so developers can quickly triage failures without digging through raw logs.

---

## Research Phase

### Approach Comparison

We evaluated three main approaches for AI-powered CI failure summaries:

#### Option A: `anthropics/claude-code-action` (Recommended)

The official Claude Code GitHub Action from Anthropic. Supports structured JSON output via `--json-schema`, direct access to CI logs via `actions: read` permission, and native PR commenting.

| Aspect | Details |
|--------|---------|
| **Maturity** | Actively maintained (v1.x), official Anthropic product |
| **Auth** | `ANTHROPIC_API_KEY` secret or OIDC via Bedrock/Vertex |
| **Structured output** | `--json-schema` flag returns validated JSON as action output |
| **CI log access** | Native — reads workflow runs, job logs, and test results when `actions: read` is configured |
| **PR comments** | Built-in via `gh pr comment` tool permission |
| **Flaky test detection** | Documented pattern with `is_flaky`, `confidence`, `summary` schema |
| **Cost** | Pay-per-use Anthropic API (Claude Haiku recommended for cost efficiency) |
| **Pros** | Full codebase context, can read + fix code, structured outputs, official support |
| **Cons** | Requires API key, occasional setup timeouts (Issue #693), exit code 5 on some reviews (Issue #846) |
| **Links** | [Repo](https://github.com/anthropics/claude-code-action), [Docs](https://code.claude.com/docs/en/github-actions), [Solutions](https://github.com/anthropics/claude-code-action/blob/main/docs/solutions.md) |

#### Option B: `Chris1220-cmd/ci-fix-coach`

Community-built action that reads failed CI logs and posts structured fix guides.

| Aspect | Details |
|--------|---------|
| **Maturity** | Community project, single maintainer |
| **Auth** | `ANTHROPIC_API_KEY` |
| **Output format** | Fixed A/B/C/D/E structure: What failed / Why / Steps to fix / Local check / File to change |
| **Smart features** | Smart log extraction (finds first error + context), comment deduplication (updates same comment) |
| **Pros** | Simple, focused, consistent output format |
| **Cons** | Less flexible, no structured JSON output, no codebase context, community-maintained |
| **Links** | [Repo](https://github.com/Chris1220-cmd/ci-fix-coach), [Blog](https://dev.to/chris1220cmd/i-built-a-github-action-that-diagnoses-ci-failures-using-claude-ai-45hd) |

#### Option C: `ctrf-io/github-test-reporter` + `ctrf-io/ai-test-reporter`

Test-report-first approach: convert test output to CTRF/JUnit format, then summarize with AI.

| Aspect | Details |
|--------|---------|
| **Maturity** | Active open-source project with broad framework support |
| **Auth** | `ANTHROPIC_API_KEY` (or OpenAI, Gemini, etc.) |
| **Test format** | Requires CTRF JSON or JUnit XML — Rust needs `cargo-nextest` with `--message-format junit` |
| **Features** | Collapsible reports, flaky test detection, failed test analysis, AI summaries, PR comments |
| **AI model** | Supports Claude (Anthropic), GPT (OpenAI), Gemini, and 300+ models |
| **Pros** | Framework-agnostic, rich reporting, supports multiple AI providers, collapsible sections |
| **Cons** | Requires test output conversion (nextest JUnit), no direct codebase context, two-tool setup |
| **Links** | [Reporter](https://github.com/ctrf-io/github-test-reporter), [AI Reporter](https://github.com/ctrf-io/ai-test-reporter), [CTRF](https://ctrf.io/integrations) |

#### Option D: GitHub Copilot Code Review (Native)

GitHub's built-in Copilot features for PR review and code scanning autofix.

| Aspect | Details |
|--------|---------|
| **Maturity** | GA (April 2025), native GitHub feature |
| **Auth** | GitHub Advanced Security license (Autofix), or Copilot subscription (Code Review) |
| **Scope** | Code scanning alerts (CodeQL) + PR code review — **not CI test failure analysis** |
| **Pros** | Zero setup, native UX, no API keys |
| **Cons** | Does NOT analyze CI test failures — focused on static analysis and code review only. Not customizable for our failure classification needs |
| **Links** | [Autofix Docs](https://docs.github.com/en/code-security/code-scanning/managing-code-scanning-alerts/responsible-use-autofix-code-scanning), [Code Review](https://github.blog/changelog/2026-03-11-request-copilot-code-review-from-github-cli/) |

### Decision

**Use Option A (`anthropics/claude-code-action`)** as the primary approach, supplemented by **Option C (`cargo-nextest` JUnit output)** for structured test data.

**Rationale:**
- Claude Code Action has full codebase context — it can read source files to suggest precise fixes
- Structured JSON output enables downstream automation (auto-retry flaky tests, label issues)
- Official Anthropic support with active maintenance
- Already referenced in our testing strategy (SP10) under "Self-healing CI"
- Can classify failures, not just summarize them
- JUnit output from `cargo-nextest` provides structured test data that feeds into the prompt

**Copilot is not suitable** for this use case — it handles code scanning/review but does not analyze CI test failure logs or provide failure summaries.

---

## Execution Phase

### Architecture

```
CI Workflow Fails
    │
    ├─> Collect structured test data (cargo-nextest JUnit XML)
    ├─> Collect raw log output from failed jobs
    │
    ├─> Invoke claude-code-action with failure context
    │     ├─> Classify failure type
    │     ├─> Identify root cause
    │     ├─> Detect recurring patterns
    │     ├─> Suggest fixes with file:line references
    │     └─> Return structured JSON
    │
    ├─> Post collapsed PR comment
    │     ├─> Summary header (pass/fail counts, classification)
    │     ├─> <details> per failed job
    │     │     ├─> Error message
    │     │     ├─> Root cause
    │     │     ├─> Suggested fix
    │     │     └─> Link to job log
    │     ├─> <details> for recurring issues
    │     └─> Footer with re-run link
    │
    └─> (Optional) Auto-retry if flaky test detected
```

### PR Comment Format

The comment posted to PRs uses GitHub-flavored Markdown with collapsible sections:

```markdown
## CI Failure Summary

**Status:** 2 of 4 jobs failed | **Classification:** Test Failure, Clippy Warning
**Flaky:** No (confidence: 0.95)

<details>
<summary>❌ <b>Tests</b> — 3 tests failed (<a href="https://github.com/...">job log</a>)</summary>

### `test_config_parsing::test_precedence_order`
- **Error:** `assertion failed: expected "foo", got "bar"`
- **Root cause:** Config precedence logic does not account for env var override
- **Suggested fix:** In `src/config.rs:142`, check `env_override` before `cqlshrc_value`
- **Category:** Logic error

### `test_formatter::test_uuid_display`
- **Error:** `thread 'test_uuid_display' panicked at 'called Result::unwrap() on an Err value'`
- **Root cause:** UUID parsing fails on uppercase input
- **Suggested fix:** In `src/types/uuid.rs:28`, use `.to_lowercase()` before parsing
- **Category:** Input validation

### `test_driver::test_connect_timeout`
- **Error:** `connection timed out after 5s`
- **Root cause:** Testcontainer startup latency exceeded test timeout
- **Suggested fix:** Increase timeout or add retry — this is likely a **flaky test**
- **Category:** Infrastructure / Flaky

</details>

<details>
<summary>❌ <b>Clippy</b> — 1 warning-as-error (<a href="https://github.com/...">job log</a>)</summary>

### `clippy::needless_borrow`
- **Error:** `the borrowed expression implements the required traits` at `src/repl.rs:87`
- **Suggested fix:** Remove `&` — change `&self.prompt` to `self.prompt`
- **Category:** Lint

</details>

<details>
<summary>✅ <b>Rustfmt</b> — passed</summary>
No issues.
</details>

<details>
<summary>✅ <b>Build</b> — passed</summary>
No issues.
</details>

---

<details>
<summary>📊 <b>Recurring Issues</b> (seen in last 5 runs)</summary>

| Issue | Occurrences | Category | Pattern |
|-------|-------------|----------|---------|
| `test_connect_timeout` flaky | 3/5 runs | Infrastructure | Testcontainer startup race |
| `clippy::needless_borrow` | 2/5 runs | Lint | Introduced in PR #42, not yet fixed |

**Recommendation:** `test_connect_timeout` should be marked `#[ignore]` or given a retry wrapper. The clippy warning was introduced in commit `abc1234` and should be fixed at source.

</details>

---
*🤖 Generated by [Claude CI Analyzer](https://github.com/anthropics/claude-code-action) • [Re-run failed jobs](https://github.com/...)*
```

### Failure Classification Taxonomy

The AI classifies each failure into one of these categories:

| Category | Description | Auto-action |
|----------|-------------|-------------|
| **Compilation error** | Code does not compile (`cargo build` / `cargo check`) | None — must be fixed |
| **Test failure** | A test assertion failed | None — must be fixed |
| **Lint violation** | Clippy or rustfmt failure | Suggest exact fix |
| **Infrastructure / Flaky** | Timeout, container startup, network issue | Auto-retry (if confidence > 0.8) |
| **Dependency issue** | Cargo resolution, version conflict, yanked crate | Suggest `cargo update` |
| **Configuration error** | CI workflow YAML issue, missing secret, permission error | Link to docs |
| **Unknown** | Cannot classify | Flag for manual review |

### Recurring Issue Detection

To detect recurring issues across runs, the workflow:

1. **Stores failure history** — After each analysis, appends a summary JSON artifact to the workflow run
2. **Reads recent history** — On failure, fetches the last N workflow run artifacts for the same PR branch
3. **Correlates failures** — Matches failures by test name, error message similarity, and classification
4. **Reports patterns** — Includes a "Recurring Issues" section when the same failure appears in 2+ of the last 5 runs

Implementation approach:
```yaml
# Upload failure summary as artifact for history tracking
- name: Save failure history
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: ci-failure-${{ github.run_number }}
    path: failure-summary.json
    retention-days: 30

# Download recent failure artifacts for pattern detection
- name: Fetch failure history
  uses: actions/download-artifact@v4
  with:
    pattern: ci-failure-*
    merge-multiple: true
    path: failure-history/
```

### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | Add `cargo-nextest` to CI for JUnit XML output | Updated `ci.yml` |
| 2 | Create `ci-failure-analysis.yml` workflow | New workflow file |
| 3 | Define JSON schema for structured failure output | Schema in workflow |
| 4 | Write prompt template for failure classification | Prompt in workflow |
| 5 | Implement collapsed PR comment formatting | Comment template |
| 6 | Add failure history artifact upload/download | Artifact steps |
| 7 | Add recurring issue correlation logic | Prompt enhancement |
| 8 | Add auto-retry for flaky tests (confidence threshold) | Conditional step |
| 9 | Test with intentional failures on a feature branch | Validation |
| 10 | Document the feature and failure taxonomy | Update SP10 |

### Step 1: Add `cargo-nextest` to CI

Replace `cargo test` with `cargo-nextest` for structured JUnit XML output:

```yaml
test:
  name: Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - uses: taiki-e/install-action@nextest
    - name: Run tests
      run: cargo nextest run --all-targets --all-features --message-format junit --output-file test-results.xml
    - name: Upload test results
      if: always()
      uses: actions/upload-artifact@v4
      with:
        name: test-results
        path: test-results.xml
```

### Step 2: CI Failure Analysis Workflow

```yaml
name: CI Failure Analysis

on:
  workflow_run:
    workflows: ["CI"]
    types: [completed]

jobs:
  analyze-failure:
    if: >
      github.event.workflow_run.conclusion == 'failure' &&
      github.event.workflow_run.event == 'pull_request'
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      actions: read
      checks: read
      id-token: write

    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.workflow_run.head_sha }}

      # Download test results artifact from the failed run
      - name: Download test results
        uses: actions/download-artifact@v4
        with:
          name: test-results
          run-id: ${{ github.event.workflow_run.id }}
          github-token: ${{ secrets.GITHUB_TOKEN }}
        continue-on-error: true

      # Download failure history for recurring issue detection
      - name: Download failure history
        uses: dawidd6/action-download-artifact@v6
        with:
          workflow: ci-failure-analysis.yml
          branch: ${{ github.event.workflow_run.head_branch }}
          name: ci-failure-.*
          name_is_regexp: true
          path: failure-history/
          if_no_artifact_found: warn
        continue-on-error: true

      # Get the PR number from the workflow run
      - name: Get PR number
        id: pr
        run: |
          PR_NUMBER=$(gh api repos/${{ github.repository }}/commits/${{ github.event.workflow_run.head_sha }}/pulls --jq '.[0].number')
          echo "number=$PR_NUMBER" >> "$GITHUB_OUTPUT"
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      # Analyze failures with Claude
      - name: Analyze CI failure
        id: analyze
        uses: anthropics/claude-code-action@v1
        with:
          anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
          model: claude-haiku-4-5-20251001
          prompt: |
            REPO: ${{ github.repository }}
            PR NUMBER: ${{ steps.pr.outputs.number }}
            FAILED WORKFLOW RUN: ${{ github.event.workflow_run.id }}
            HEAD SHA: ${{ github.event.workflow_run.head_sha }}

            Analyze the CI failure for this pull request. Do the following:

            1. Read the workflow run logs using:
               gh api repos/${{ github.repository }}/actions/runs/${{ github.event.workflow_run.id }}/logs
               OR use: gh run view ${{ github.event.workflow_run.id }} --log-failed

            2. If test-results.xml exists, read it for structured test failure data.

            3. If failure-history/ directory has files, read them to detect recurring issues.

            4. For each failed job, determine:
               - The exact error message
               - The root cause (why it failed)
               - A suggested fix with specific file:line references
               - A classification from: compilation_error, test_failure, lint_violation,
                 infrastructure_flaky, dependency_issue, configuration_error, unknown

            5. Assess flakiness: is this likely a flaky test? Provide confidence 0-1.

            6. Check for recurring patterns across failure history files.

            Return a structured JSON response matching the provided schema.
          claude_args: |
            --allowedTools "Bash(gh run view:*),Bash(gh api:*),Bash(cat:*),Bash(ls:*),Read"
            --model claude-haiku-4-5-20251001
            --json-schema '{"type":"object","properties":{"summary":{"type":"string","description":"One-line overall summary"},"total_jobs":{"type":"integer"},"failed_jobs":{"type":"integer"},"is_flaky":{"type":"boolean"},"flaky_confidence":{"type":"number"},"failures":{"type":"array","items":{"type":"object","properties":{"job_name":{"type":"string"},"test_name":{"type":"string"},"error_message":{"type":"string"},"root_cause":{"type":"string"},"suggested_fix":{"type":"string"},"file_reference":{"type":"string"},"classification":{"type":"string","enum":["compilation_error","test_failure","lint_violation","infrastructure_flaky","dependency_issue","configuration_error","unknown"]}},"required":["job_name","error_message","root_cause","suggested_fix","classification"]}},"recurring_issues":{"type":"array","items":{"type":"object","properties":{"issue":{"type":"string"},"occurrences":{"type":"integer"},"category":{"type":"string"},"pattern":{"type":"string"},"recommendation":{"type":"string"}},"required":["issue","occurrences","category","recommendation"]}}},"required":["summary","total_jobs","failed_jobs","is_flaky","failures"]}'

      # Format and post PR comment
      - name: Post failure summary comment
        if: steps.pr.outputs.number != ''
        uses: actions/github-script@v7
        with:
          script: |
            const output = JSON.parse(`${{ steps.analyze.outputs.structured_output }}`);
            const runUrl = `https://github.com/${{ github.repository }}/actions/runs/${{ github.event.workflow_run.id }}`;

            // Classification emoji map
            const classEmoji = {
              compilation_error: '🔨',
              test_failure: '🧪',
              lint_violation: '📏',
              infrastructure_flaky: '🔄',
              dependency_issue: '📦',
              configuration_error: '⚙️',
              unknown: '❓'
            };

            // Group failures by job
            const byJob = {};
            for (const f of output.failures) {
              if (!byJob[f.job_name]) byJob[f.job_name] = [];
              byJob[f.job_name].push(f);
            }

            // Build classifications summary
            const classifications = [...new Set(output.failures.map(f => f.classification))];
            const classLabels = classifications.map(c =>
              `${classEmoji[c] || '❓'} ${c.replace(/_/g, ' ')}`
            ).join(', ');

            let body = `## CI Failure Summary\n\n`;
            body += `**Status:** ${output.failed_jobs} of ${output.total_jobs} jobs failed`;
            body += ` | **Classification:** ${classLabels}\n`;
            body += `**Flaky:** ${output.is_flaky ? 'Yes' : 'No'} (confidence: ${(output.flaky_confidence || 0).toFixed(2)})\n\n`;

            // Failed jobs (collapsed)
            for (const [job, failures] of Object.entries(byJob)) {
              const count = failures.length;
              body += `<details>\n`;
              body += `<summary>❌ <b>${job}</b> — ${count} issue${count > 1 ? 's' : ''} (<a href="${runUrl}">job log</a>)</summary>\n\n`;

              for (const f of failures) {
                const title = f.test_name || f.job_name;
                body += `### \`${title}\`\n`;
                body += `- **Error:** \`${f.error_message}\`\n`;
                body += `- **Root cause:** ${f.root_cause}\n`;
                body += `- **Suggested fix:** ${f.suggested_fix}\n`;
                if (f.file_reference) body += `- **File:** \`${f.file_reference}\`\n`;
                body += `- **Category:** ${classEmoji[f.classification] || '❓'} ${f.classification.replace(/_/g, ' ')}\n\n`;
              }

              body += `</details>\n\n`;
            }

            // Recurring issues (collapsed)
            if (output.recurring_issues && output.recurring_issues.length > 0) {
              body += `<details>\n`;
              body += `<summary>📊 <b>Recurring Issues</b> (seen across recent runs)</summary>\n\n`;
              body += `| Issue | Occurrences | Category | Recommendation |\n`;
              body += `|-------|-------------|----------|----------------|\n`;
              for (const ri of output.recurring_issues) {
                body += `| ${ri.issue} | ${ri.occurrences} | ${ri.category} | ${ri.recommendation} |\n`;
              }
              body += `\n</details>\n\n`;
            }

            body += `---\n`;
            body += `*🤖 Generated by Claude CI Analyzer • [Re-run failed jobs](${runUrl})*\n`;

            // Find existing comment to update (deduplication)
            const comments = await github.rest.issues.listComments({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: ${{ steps.pr.outputs.number }}
            });

            const marker = '## CI Failure Summary';
            const existing = comments.data.find(c =>
              c.user.type === 'Bot' && c.body.startsWith(marker)
            );

            if (existing) {
              await github.rest.issues.updateComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                comment_id: existing.id,
                body: body
              });
            } else {
              await github.rest.issues.createComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: ${{ steps.pr.outputs.number }},
                body: body
              });
            }

      # Save failure summary for history tracking
      - name: Save failure history
        if: always()
        run: |
          echo '${{ steps.analyze.outputs.structured_output }}' | \
            jq '. + {"run_number": ${{ github.event.workflow_run.run_number }}, "timestamp": now | todate, "sha": "${{ github.event.workflow_run.head_sha }}"}' \
            > failure-summary.json
        continue-on-error: true

      - name: Upload failure history
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: ci-failure-${{ github.event.workflow_run.run_number }}
          path: failure-summary.json
          retention-days: 30

      # Auto-retry flaky tests
      - name: Auto-retry if flaky
        if: >
          fromJSON(steps.analyze.outputs.structured_output).is_flaky == true &&
          fromJSON(steps.analyze.outputs.structured_output).flaky_confidence > 0.8
        run: |
          echo "Flaky test detected with high confidence — triggering re-run"
          gh run rerun ${{ github.event.workflow_run.id }} --failed
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Step 3: Required Secrets

| Secret | Purpose | Where to get it |
|--------|---------|-----------------|
| `ANTHROPIC_API_KEY` | Claude API access for failure analysis | [console.anthropic.com](https://console.anthropic.com) |

### Cost Estimation

Using Claude Haiku for cost efficiency:

| Metric | Estimate |
|--------|----------|
| Average log size | ~5,000 tokens input |
| Prompt + schema | ~2,000 tokens input |
| Response | ~1,000 tokens output |
| Cost per analysis | ~$0.005 (half a cent) |
| Monthly (50 failures) | ~$0.25 |

---

## Acceptance Criteria

- [ ] CI failures trigger automatic analysis workflow
- [ ] PR comment is posted with collapsed sections per failed job
- [ ] Each failure includes: error message, root cause, suggested fix, classification
- [ ] Comment is updated (not duplicated) on subsequent pushes
- [ ] Recurring issues are detected and reported across runs
- [ ] Flaky tests are auto-retried when detected with high confidence
- [ ] JUnit XML from `cargo-nextest` provides structured test data
- [ ] Cost per analysis stays under $0.01

---

## Future Enhancements

- **Auto-fix mode** — For lint violations and simple test fixes, have Claude push a fix commit directly
- **Slack/Discord notification** — Forward failure summaries to team channels
- **Dashboard** — Aggregate failure patterns across PRs into a trends dashboard
- **Custom rules** — Allow `.ci-failure-rules.yml` to define project-specific classification overrides
- **Multi-workflow support** — Analyze failures across release, benchmark, and integration test workflows

---

## Skills Required

- GitHub Actions workflow authoring (S11)
- Claude API / claude-code-action (S15)
- Rust CI tooling: cargo-nextest, JUnit XML (S9)
- GitHub API: PR comments, artifacts, workflow runs (S11)
