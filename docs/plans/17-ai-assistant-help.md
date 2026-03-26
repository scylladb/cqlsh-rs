# SP17: Embedded AI Assistant (`--ai-help`)

> **Status: NOT STARTED** — Post-v1 value-add feature. All 4 phases pending.

## Overview

Embed a small, quantized LLM (Qwen2.5-Coder-0.5B) directly into cqlsh-rs to provide instant, offline CQL diagnostic suggestions when queries fail. This is a **value-add feature** beyond Python cqlsh compatibility — it must never interfere with core functionality.

**Design principle:** The AI assistant is strictly opt-in, gracefully degradable, and invisible when not requested. A user who never passes `--ai-help` must never download a model, load inference code, or experience any performance impact.

---

## Table of Contents

1. [Goals & Non-Goals](#goals--non-goals)
2. [Feature Flag & Compile-Time Gating](#feature-flag--compile-time-gating)
3. [Model Selection & Quantization](#model-selection--quantization)
4. [Inference Engine Selection](#inference-engine-selection)
5. [Distribution & Caching Strategy](#distribution--caching-strategy)
6. [CLI Integration](#cli-integration)
7. [Prompt Engineering](#prompt-engineering)
8. [Execution UX & Graceful Degradation](#execution-ux--graceful-degradation)
9. [Resource Management](#resource-management)
10. [Module Architecture](#module-architecture)
11. [Testing Strategy](#testing-strategy)
12. [Security Considerations](#security-considerations)
13. [Platform Support Matrix](#platform-support-matrix)
14. [Phased Implementation](#phased-implementation)
15. [Open Questions & Decisions](#open-questions--decisions)
16. [Risks & Mitigations](#risks--mitigations)

---

## Goals & Non-Goals

### Goals

| # | Goal | Measure of Success |
|---|------|-------------------|
| A1 | **Offline CQL diagnostics** | When a CQL query fails and `--ai-help` is active, display a 1-3 sentence suggestion |
| A2 | **Zero impact when disabled** | Binary size, startup time, and memory usage are identical when compiled without the `ai-help` feature flag |
| A3 | **Lazy model acquisition** | Model downloads only on first use; subsequent runs use cached weights |
| A4 | **Sub-10s response time** | Inference completes within 10 seconds on a 4-core laptop CPU |
| A5 | **Silent failure** | Any AI subsystem failure falls back to standard CQL error output with zero user-visible disruption |
| A6 | **Cross-platform** | Works on Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64, Windows x86_64 |

### Non-Goals

- **Chat interface or multi-turn conversation** — this is single-shot diagnostic help
- **Query generation** — the assistant explains errors, it does not write queries for the user
- **GPU acceleration** — CPU-only; GPU support is a future consideration
- **Online/API-based inference** — the model runs entirely locally
- **Model fine-tuning or training** — use the pre-trained model as-is
- **Replacing standard error messages** — AI suggestions supplement, never replace, the native CQL error

---

## Feature Flag & Compile-Time Gating

The entire AI subsystem is behind a Cargo feature flag. This is non-negotiable: users who do not want AI functionality get a binary with zero AI code compiled in.

```toml
[features]
default = []
ai-help = ["dep:llama-cpp-2", "dep:hf-hub", "dep:indicatif", "dep:dirs"]
```

**Conditional compilation pattern:**

- All AI modules are gated with `#[cfg(feature = "ai-help")]`
- The `--ai-help` CLI flag is only registered when the feature is enabled
- When compiled without `ai-help`, the `--ai-help` flag is absent from `--help` output entirely
- CI builds two variants: standard (default) and `ai-help`-enabled

**Binary size impact:**

| Build | Expected Size |
|-------|---------------|
| Standard (no AI) | ~5-10 MB |
| With `ai-help` feature | ~15-25 MB (inference engine statically linked) |

The model weights (350-500 MB) are **never** embedded in the binary.

---

## Model Selection & Quantization

### Model: Qwen2.5-Coder-0.5B-Instruct

**Rationale:**
- 0.5B parameters — small enough for instant CPU inference
- Instruction-tuned — follows chat template prompts reliably
- Code-specialized — trained on code and technical content, suitable for CQL diagnostics
- Permissive license (Apache 2.0) — no distribution restrictions

### Quantization Format: GGUF

**Selected quantization: Q4_K_M**

| Format | Size | Quality | Speed | Decision |
|--------|------|---------|-------|----------|
| fp16 (raw) | ~1 GB | Best | Slow | **Rejected** — excessive RAM, no benefit for diagnostic suggestions |
| Q8_0 | ~500 MB | Near-lossless | Fast | Fallback option if Q4_K_M quality proves insufficient |
| Q4_K_M | ~350 MB | Good for short outputs | Fastest | **Selected** — best trade-off for terse diagnostic suggestions |
| Q4_0 | ~300 MB | Degraded | Fastest | **Rejected** — quality degradation noticeable in short outputs |

**Key decision:** Q4_K_M is the default. If user testing reveals quality issues with CQL-specific suggestions, upgrade to Q8_0. The download/caching system handles either transparently.

### Model Source

Download from Hugging Face Hub. The specific model ID and revision hash are pinned in code (not configurable by users) to prevent supply-chain risks:

```
Model ID: Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF
File:     qwen2.5-coder-0.5b-instruct-q4_k_m.gguf
```

**Integrity verification:** After download, verify the file against a SHA-256 hash embedded in the binary. If verification fails, delete the file and re-download. If re-download also fails verification, disable AI help and warn the user.

---

## Inference Engine Selection

### Decision: `llama-cpp-2` (Rust bindings for llama.cpp)

| Criteria | llama-cpp-2 | candle |
|----------|-------------|--------|
| GGUF support | Native | Requires conversion to safetensors |
| CPU optimization | AVX2, AVX512, ARM NEON, Apple Accelerate — automatic | Manual compiler flags required |
| Build complexity | Requires C/C++ toolchain (cmake) | Pure Rust, cargo-only |
| Inference speed (0.5B, CPU) | 30-50 tok/s | 15-25 tok/s |
| Memory efficiency | Excellent (mmap, quantized KV cache) | Good (but no mmap for GGUF) |
| Maturity | Battle-tested (llama.cpp ecosystem) | Newer, evolving API |
| Binary size impact | +10-15 MB | +5-8 MB |

**Decision: `llama-cpp-2`** — superior CPU inference speed and native GGUF support outweigh the C++ build dependency. The build dependency is acceptable because:
1. The feature is behind a feature flag — users who don't want it never need cmake
2. CI handles the C++ toolchain setup
3. Pre-built binaries (via GitHub releases) eliminate end-user build requirements

**Fallback consideration:** If `llama-cpp-2` proves problematic for cross-compilation (especially Windows/aarch64), reconsider `candle` with safetensors format. Document the switch criteria:
- Cross-compilation fails for 2+ targets after reasonable effort
- Build times exceed 10 minutes on CI

---

## Distribution & Caching Strategy

### Principle: Never Bundle, Always Cache

The model file is **never** included in the distributed binary. It is downloaded lazily on first use.

### Download Flow

```
User runs `cqlsh-rs --ai-help -e "SELECT * FORM users"`
         │
         ▼
    ┌─────────────────┐
    │ Check cache dir  │
    │ for model file   │
    └────────┬────────┘
             │
        ┌────┴────┐
        │ Exists? │
        └────┬────┘
          No │        Yes
             ▼          ▼
    ┌─────────────┐   ┌──────────────┐
    │ Verify hash │   │ Verify hash  │
    │ (if exists  │   │              │
    │  but corrupt)│   └──────┬───────┘
    └──────┬──────┘          │
           │            ┌────┴────┐
           ▼            │ Valid?  │
    ┌─────────────┐     └────┬────┘
    │ Download     │       Yes│    No
    │ from HF Hub  │◄────────┘    │
    │ + progress   │              ▼
    │ bar          │         ┌──────────┐
    └──────┬──────┘         │ Delete & │
           │                │ redownload│
           ▼                └──────────┘
    ┌─────────────┐
    │ Verify hash │
    │ Save to     │
    │ cache dir   │
    └──────┬──────┘
           │
           ▼
    ┌─────────────┐
    │ Load model  │
    │ & run       │
    │ inference   │
    └─────────────┘
```

### Cache Directory

Use the `dirs` crate to resolve platform-appropriate cache paths:

| Platform | Path |
|----------|------|
| Linux | `$XDG_CACHE_HOME/cqlsh-rs/models/` or `~/.cache/cqlsh-rs/models/` |
| macOS | `~/Library/Caches/cqlsh-rs/models/` |
| Windows | `%LOCALAPPDATA%\cqlsh-rs\models\` |

### Download UX

- Show a progress bar with `indicatif` during download
- Display estimated file size and download speed
- Print a one-time message: `Downloading AI model for CQL diagnostics (350 MB, one-time)...`
- After successful download: `AI model cached. Future runs will start instantly.`
- If the user is in non-interactive mode (`-e` / `--execute`), still download but use a simpler progress format (no spinner animations)

### Cache Management

- **No automatic updates.** Model version is pinned. Updates happen only with cqlsh-rs version bumps.
- **Manual cache clear:** Document `cqlsh-rs --ai-clear-cache` or instruct users to delete the cache directory
- **Cache size reporting:** `cqlsh-rs --ai-cache-info` prints cache location and size (optional, low priority)

### Offline / Air-Gapped Environments

For environments without internet access:
- Users can manually download the GGUF file and place it in the cache directory
- Document the expected filename and hash for manual verification
- The `CQLSH_AI_MODEL_PATH` environment variable overrides the cache directory, pointing to a pre-placed model file

---

## CLI Integration

### New Flags

| Flag | Description | Feature-gated |
|------|-------------|---------------|
| `--ai-help` | Enable AI-powered CQL error suggestions | Yes |
| `--ai-clear-cache` | Delete cached model files and exit | Yes |
| `--ai-threads <N>` | Override inference thread count (default: 4) | Yes |

### Activation Modes

**Mode 1: Flag-based (primary)**
```
cqlsh-rs --ai-help -e "SELECT * FORM users"
```

**Mode 2: Configuration-based**
```ini
# ~/.cassandra/cqlshrc
[ai]
enabled = true
threads = 4
```

**Mode 3: Environment variable**
```
CQLSH_AI_HELP=1 cqlsh-rs -e "SELECT * FORM users"
```

### Precedence (highest to lowest)
1. CLI flag (`--ai-help`)
2. Environment variable (`CQLSH_AI_HELP`)
3. Configuration file (`[ai] enabled`)

---

## Prompt Engineering

### Chat Template (ChatML for Qwen2.5)

The model expects this exact format:

```
<|im_start|>system
{system_prompt}<|im_end|>
<|im_start|>user
{user_message}<|im_end|>
<|im_start|>assistant
```

### System Prompt

```
You are a terse CQL diagnostic assistant embedded in a Cassandra shell.
Given a failed CQL statement and its error message, provide a brief fix suggestion.
Rules:
- Maximum 2 sentences
- Show the corrected CQL if the fix is a syntax change
- Never use markdown formatting
- Never suggest anything unrelated to CQL or Cassandra
- If unsure, say "No suggestion available"
```

**Design rationale:**
- "Terse" and "Maximum 2 sentences" prevent rambling
- "Never use markdown" ensures clean terminal output
- "Never suggest anything unrelated" constrains hallucination scope
- Explicit fallback instruction ("No suggestion available") gives a safe default

### User Prompt Construction

```
CQL Statement:
{the_statement_the_user_typed}

Error:
{the_error_message_from_cassandra}

Suggest a fix:
```

### Generation Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `max_tokens` | 128 | Enough for 2 sentences + a corrected query; hard cap prevents runaway generation |
| `temperature` | 0.1 | Near-deterministic; diagnostic suggestions should be consistent |
| `top_p` | 0.9 | Standard nucleus sampling |
| `repeat_penalty` | 1.1 | Prevent token repetition loops |
| `stop_tokens` | `<\|im_end\|>`, `\n\n\n` | Stop at end of assistant turn or triple newline |

---

## Execution UX & Graceful Degradation

### Terminal Output Format

When AI help produces a suggestion:

```
SyntaxException: line 1:14 no viable alternative at input 'FORM'

💡 AI suggestion: Did you mean FROM? Try: SELECT * FROM users;
```

- Use a distinct prefix (`💡 AI suggestion:` or `[ai]` for non-Unicode terminals)
- Print to stderr so it doesn't pollute piped stdout output
- Detect Unicode support and fall back to `[ai]` prefix on terminals that don't support emoji

### Spinner During Inference

- Show a spinner on stderr while the model is generating
- Use `indicatif` with a message like `Thinking...`
- Clear the spinner before printing the suggestion

### Timeout

- **Hard timeout: 15 seconds** from model load to output completion
- If exceeded, kill the inference thread and print nothing (silent fallback)
- The timeout covers the entire inference pipeline (load + generate), not just generation

### Graceful Degradation Hierarchy

Every failure mode falls back silently to standard behavior:

| Failure | Behavior |
|---------|----------|
| Feature not compiled in | `--ai-help` flag doesn't exist; no error |
| Model not downloaded + no internet | Print: `Note: AI help unavailable (model not cached). Run with internet to enable.` |
| Download fails mid-transfer | Delete partial file, print note, continue with standard error |
| Model file corrupted | Delete file, attempt re-download once, then fall back |
| Inference panics | Catch panic at thread boundary, print nothing |
| Inference times out | Kill thread, print nothing |
| Inference produces empty/garbage output | Print nothing |
| Cache directory not writable | Print note on first attempt, then silently skip |
| Insufficient memory | Catch allocation failure, print nothing |

**Key principle:** The user's CQL error message is always printed first, before any AI processing begins. If AI fails at any point, the user already has the standard error output.

---

## Resource Management

### Thread Management

| Setting | Value | Rationale |
|---------|-------|-----------|
| Default threads | 4 | Reasonable for most laptops; doesn't monopolize CPU |
| Max threads | `num_cpus::get()` | Never exceed available cores |
| Min threads | 1 | Single-threaded fallback |
| Configurable via | `--ai-threads`, `[ai] threads`, `CQLSH_AI_THREADS` | Same precedence as other settings |

### Memory Budget

| Phase | Memory Usage |
|-------|-------------|
| Model loading (mmap) | ~50 MB resident (OS pages in as needed from the 350 MB file) |
| KV cache (128 tokens) | ~20 MB |
| Tokenizer | ~5 MB |
| Peak total | ~75 MB |

With mmap, the OS manages physical memory pressure. The 350 MB file is mapped but only the pages accessed during inference become resident.

### Lifecycle: Single-Shot

```
1. User query fails → error printed
2. Load model (mmap, near-instant if file is in OS page cache)
3. Tokenize prompt
4. Generate up to 128 tokens
5. Print suggestion
6. Drop model — process exits or returns to REPL
```

**No background model loading.** Do not speculatively load the model at startup. Load only when a query fails and AI help is active. In REPL mode, this means the model is loaded and dropped on each error — OS file caching makes subsequent loads fast (~100ms).

### REPL Mode Considerations

In interactive REPL mode, the model could potentially be kept loaded between errors to save reload time. **Decision: Do not keep loaded.** Rationale:
- 75 MB resident memory for a rarely-triggered feature is wasteful
- OS page cache provides near-instant reload for repeated errors
- Simpler lifecycle (no background resource management)
- If profiling shows reload is a bottleneck, reconsider with an LRU-style lazy holder that drops after 60 seconds of inactivity

---

## Module Architecture

```
src/
├── ai/                          # All AI code, gated with #[cfg(feature = "ai-help")]
│   ├── mod.rs                   # Public API: suggest_fix(statement, error) -> Option<String>
│   ├── model.rs                 # Model loading, caching, hash verification
│   ├── inference.rs             # Prompt formatting, generation, output parsing
│   ├── download.rs              # HF Hub download, progress bar, retry logic
│   └── config.rs                # AI-specific config (threads, cache path, etc.)
├── ...
```

### Public API

The AI module exposes exactly one function to the rest of the codebase:

```rust
/// Attempt to generate an AI suggestion for a CQL error.
/// Returns None on any failure (timeout, model unavailable, etc.)
#[cfg(feature = "ai-help")]
pub async fn suggest_fix(
    statement: &str,
    error: &str,
    config: &AiConfig,
) -> Option<String>;
```

The caller (error handler) does:
1. Print the CQL error (always, immediately)
2. If AI help is enabled, call `suggest_fix`
3. If `Some(suggestion)`, print it with the AI prefix
4. If `None`, do nothing

---

## Testing Strategy

### Unit Tests

| Component | Test Approach |
|-----------|---------------|
| Prompt formatting | Assert exact ChatML template output for known inputs |
| Output parsing | Test extraction of suggestion text, handling of empty/garbage output |
| Config resolution | Test precedence: CLI > env > config file |
| Cache path resolution | Test per-platform path generation |
| Hash verification | Test with valid file, corrupted file, missing file |

### Integration Tests

| Test | Description |
|------|-------------|
| Download mock | Use a local HTTP server (wiremock) to test download flow without hitting HF Hub |
| Inference smoke test | Load a tiny test model, verify generation produces output (CI only, with model in test fixtures) |
| Timeout test | Verify hard timeout kills inference and returns None |
| Graceful degradation | Verify each failure mode returns None without panic |

### No Model in CI by Default

The actual GGUF model is **not** stored in the repository or downloaded in standard CI runs. Integration tests that require inference use a minimal test fixture model (or mock the inference boundary).

A separate, optional CI job (`test-ai-help`) downloads the real model and runs end-to-end tests. This job is:
- Triggered manually or on release branches only
- Has a longer timeout (10 minutes)
- Cached across runs

---

## Security Considerations

| Risk | Mitigation |
|------|------------|
| Supply-chain attack via model file | SHA-256 hash verification; model ID and hash pinned in source code |
| Model produces harmful output | System prompt constrains to CQL-only; output capped at 128 tokens; output never executed, only displayed |
| Arbitrary code execution via model | GGUF is a data format, not executable; llama.cpp processes it as weights only |
| Cache directory tampering | Hash verification on every load; corrupted files are deleted |
| Network traffic concerns | Download only from Hugging Face Hub over HTTPS; document the URL for firewall allowlisting |
| Privacy | No user data is sent anywhere; inference is 100% local |

---

## Platform Support Matrix

| Platform | llama-cpp-2 Support | CPU Optimizations | Status |
|----------|--------------------|--------------------|--------|
| Linux x86_64 | Full | AVX2, AVX512 | Primary target |
| Linux aarch64 | Full | ARM NEON | Supported |
| macOS x86_64 | Full | AVX2, Accelerate | Supported |
| macOS aarch64 | Full | ARM NEON, Accelerate, Metal (CPU mode) | Supported |
| Windows x86_64 | Full | AVX2 | Supported (MSVC build) |
| Windows aarch64 | Partial | ARM NEON | Best-effort; test and document |

---

## Phased Implementation

### Phase 1: Infrastructure (Feature flag, config, download)

- Add `ai-help` feature flag to `Cargo.toml`
- Create `src/ai/` module structure with `#[cfg(feature = "ai-help")]` gating
- Implement `AiConfig` (CLI flags, env vars, cqlshrc `[ai]` section)
- Implement model download via `hf-hub` with progress bar
- Implement cache directory resolution (per-platform)
- Implement SHA-256 hash verification
- Add `--ai-clear-cache` command
- Unit tests for config, paths, hash verification

### Phase 2: Inference Engine Integration

- Add `llama-cpp-2` dependency (feature-gated)
- Implement model loading with mmap
- Implement ChatML prompt formatting for Qwen2.5
- Implement generation with parameter controls (max_tokens, temperature, etc.)
- Implement hard timeout with thread cancellation
- Implement output parsing (extract suggestion, discard garbage)
- Unit tests for prompt formatting, output parsing
- Integration test with mock/fixture model

### Phase 3: CLI Integration & UX

- Wire `suggest_fix()` into the error handling pipeline
- Implement spinner during inference
- Implement output formatting (emoji prefix, stderr, Unicode detection)
- Handle REPL mode (per-error invocation)
- Handle non-interactive mode (`-e` / `--execute`)
- Implement graceful degradation for all failure modes
- End-to-end tests with mocked inference

### Phase 4: Polish & Release

- Cross-platform testing (all targets in matrix)
- Performance profiling (load time, generation speed, memory usage)
- Documentation (README section, `--help` text, manual cache management instructions)
- Optional: `--ai-cache-info` command
- Optional CI job for real-model end-to-end tests
- Update high-level-design.md compatibility matrix

---

## Open Questions & Decisions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| Q1 | Should `--ai-help` be a persistent REPL toggle (like `TRACING ON`) or flag-only? | **Open** | Leaning flag-only for v1 simplicity. A `AI HELP ON/OFF` REPL command could be added later. |
| Q2 | Should we support user-provided models via `CQLSH_AI_MODEL_PATH`? | **Decided** | Yes — essential for air-gapped environments. |
| Q3 | What if Qwen2.5-Coder-0.5B quality is insufficient for CQL diagnostics? | **Open** | Fallback plan: try Qwen2.5-Coder-1.5B (Q4_K_M ~900 MB). If 1.5B is needed, revisit the download UX and memory budget. |
| Q4 | Should AI suggestions be logged/captured by the `CAPTURE` command? | **Open** | Leaning yes — treat AI output like any other stderr output that `CAPTURE` mirrors. |
| Q5 | Should the download respect `HTTP_PROXY` / `HTTPS_PROXY`? | **Decided** | Yes — `hf-hub` and reqwest respect standard proxy env vars by default. |
| Q6 | Should we provide a `--ai-verbose` flag for debugging model loading/inference? | **Open** | Low priority. Useful for debugging but adds flag bloat. Could use `--debug` instead. |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| llama-cpp-2 cross-compilation failures | Medium | High | Feature-gate isolates impact; fallback to candle if needed |
| Qwen2.5-Coder-0.5B produces poor CQL suggestions | Medium | Medium | Test with 50+ real CQL error scenarios; upgrade to 1.5B or improve system prompt |
| Model download blocked by corporate firewalls | High | Medium | Document `CQLSH_AI_MODEL_PATH` for manual placement; document HF Hub URL for allowlisting |
| C++ build dependency alienates contributors | Low | Medium | Feature is optional; standard builds don't need cmake |
| Model format/quantization changes upstream | Low | Low | Pin exact model revision hash; download specific file, not "latest" |
| Legal concerns with model license | Low | High | Qwen2.5-Coder is Apache 2.0; verify before release and document in LICENSE |
| Users expect ChatGPT-level quality from 0.5B model | Medium | Medium | Set expectations in docs: "This provides basic syntax suggestions, not comprehensive debugging" |
| Inference causes OOM on resource-constrained systems | Low | Medium | mmap limits resident memory; document minimum requirements (512 MB free RAM) |
