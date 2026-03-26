# Sub-Plan SP12: Cross-Platform & Release

> Parent: [high-level-design.md](high-level-design.md) | Phase: 6
> **Status: NOT STARTED** — Blocked on Phase 4-5 completion. All 10 tasks pending.

## Objective

Build, test, and distribute cqlsh-rs as a single static binary across all target platforms with automated release pipelines and package manager integration.

---

## Research Phase

### Tasks

1. **Cross-compilation targets** — linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64
2. **Static linking** — musl vs glibc, rustls for no OpenSSL dependency
3. **Binary size optimization** — LTO, strip, opt-level, codegen-units
4. **Release automation** — GitHub Releases, cargo-dist, cross-rs
5. **Package managers** — Homebrew, apt/deb, rpm, cargo install, Nix, Docker image

### Research Deliverables

- [ ] Cross-compilation target matrix with tool requirements
- [ ] Binary size optimization configuration
- [ ] Release pipeline design
- [ ] Package manager submission requirements

---

## Execution Phase

### Build Targets

| Target | Triple | Build Method | Notes |
|--------|--------|-------------|-------|
| Linux x86_64 | `x86_64-unknown-linux-musl` | Native or cross | Static binary, most common |
| Linux aarch64 | `aarch64-unknown-linux-musl` | cross-rs | ARM servers, Raspberry Pi |
| macOS x86_64 | `x86_64-apple-darwin` | GitHub Actions macos runner | Intel Macs |
| macOS aarch64 | `aarch64-apple-darwin` | GitHub Actions macos runner | Apple Silicon |
| Windows x86_64 | `x86_64-pc-windows-msvc` | GitHub Actions windows runner | Windows |

### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | Cargo.toml release profile optimization | `Cargo.toml` |
| 2 | CI workflow for cross-platform builds | `.github/workflows/release.yml` |
| 3 | Automated GitHub Releases on tag push | Release workflow |
| 4 | Binary naming convention (`cqlsh-rs-{version}-{target}`) | Release artifacts |
| 5 | SHA256 checksum generation | Checksum files |
| 6 | Homebrew formula | `homebrew-tap` repo |
| 7 | `cargo install cqlsh-rs` (crates.io publish) | `Cargo.toml` metadata |
| 8 | Docker image (multi-arch) | `Dockerfile` |
| 9 | Man page generation | `docs/cqlsh-rs.1` |
| 10 | Shell completions (bash, zsh, fish) via clap | Install scripts |

### Release Workflow

```
Tag push (v1.0.0)
    │
    ├─> Build linux-x86_64 (musl static)
    ├─> Build linux-aarch64 (cross)
    ├─> Build macos-x86_64
    ├─> Build macos-aarch64
    ├─> Build windows-x86_64
    │
    ├─> Run integration tests on each platform
    │
    ├─> Create GitHub Release
    │     ├─> Upload binaries
    │     ├─> Upload checksums
    │     └─> Generate release notes (from CHANGELOG)
    │
    ├─> Publish to crates.io
    ├─> Update Homebrew formula
    └─> Push Docker image
```

### Binary Size Targets

| Optimization | Expected Impact |
|-------------|----------------|
| LTO (thin) | -10-20% |
| Strip symbols | -30-50% |
| `opt-level = "z"` | -5-10% |
| `codegen-units = 1` | -5% |
| `panic = "abort"` | -5% |
| **Target** | **<15MB release binary** |

### Acceptance Criteria

- [ ] Static binaries build for all 5 targets
- [ ] Binaries run without any runtime dependencies
- [ ] GitHub Releases are automated on tag push
- [ ] SHA256 checksums are provided for all artifacts
- [ ] `cargo install cqlsh-rs` works
- [ ] Homebrew formula works on macOS
- [ ] Docker image is available
- [ ] Binary size is <15MB

---

## Skills Required

- Cross-compilation (S12)
- CI/CD with GitHub Actions (S11)
- Rust release optimization (S1)
- Package manager workflows (S11)
- Docker (S11)
