# Post-Publication Operations: Library Health & Maintenance

> **Scope**: This document defines the operational practices for Alaya as a published Rust crate.
> Alaya is not a deployed service. It has no servers, no dashboards, no uptime monitors.
> Every section in a traditional post-deployment playbook has been reframed for the reality
> of maintaining an embeddable library: the "infrastructure" is CI pipelines, the "users"
> are downstream crate consumers, the "incidents" are semver violations and data corruption
> bugs, and the "cost" is compile time and binary size.

**Version**: 0.1.0 (pre-publication)
**Last Updated**: 2026-02-26
**Cross-References**: [ARCHITECTURE_BLUEPRINT](ARCHITECTURE_BLUEPRINT.md) | [SECURITY_ARCHITECTURE](SECURITY_ARCHITECTURE.md) | [ADR](ADR.md) | [COMPETITIVE_LANDSCAPE](COMPETITIVE_LANDSCAPE.md)

---

## Table of Contents

1. [Library Health and Quality Metrics](#1-library-health-and-quality-metrics)
2. [Release Quality Gates](#2-release-quality-gates)
3. [Community Feedback Pipeline](#3-community-feedback-pipeline)
4. [Dependency and Algorithm Updates](#4-dependency-and-algorithm-updates)
5. [Breaking Change and Regression Response](#5-breaking-change-and-regression-response)
6. [Compile Time and Binary Size Budget](#6-compile-time-and-binary-size-budget)
7. [Roadmap Checkpoints](#7-roadmap-checkpoints)
8. [Contributor Onboarding](#8-contributor-onboarding)

---

## 1. Library Health and Quality Metrics

### 1.1 The Monitoring Reframe

A deployed service has Grafana dashboards and PagerDuty alerts. A library has CI pipelines and periodic audits. The principle is the same: continuous visibility into quality, with automated detection of regressions. For Alaya, "health" means: tests pass, benchmarks hold, dependencies are clean, the public API is stable, and the SQLite file format is forwards-compatible.

### 1.2 Metric Categories

#### 1.2.1 Test Coverage

Alaya's codebase at v0.1.0 consists of approximately 3,050 lines of Rust across 25 source files (excluding `target/` build artifacts). The module structure is:

| Module | Files | Lines | Purpose |
|--------|-------|-------|---------|
| `lib.rs` | 1 | 276 | Public API, `AlayaStore` struct, integration tests |
| `types.rs` | 1 | 402 | All public types: IDs, enums, structs, reports |
| `schema.rs` | 1 | 238 | SQLite schema, PRAGMAs, FTS5, triggers |
| `store/` | 5 | 819 | Episodic, semantic, implicit, embeddings, strengths |
| `graph/` | 3 | 292 | Hebbian links, spreading activation |
| `retrieval/` | 5 | 427 | BM25, vector, RRF fusion, rerank, pipeline |
| `lifecycle/` | 4 | 487 | Consolidation, perfuming, transformation, forgetting |
| `provider.rs` | 1 | 78 | `ConsolidationProvider` trait, `NoOpProvider`, `MockProvider` |
| `error.rs` | 1 | 21 | `AlayaError` enum, `Result` alias |

Coverage tracking should measure:

- **Line coverage** via `cargo-tarpaulin` or `cargo-llvm-cov`. Target: 80% minimum at v0.1.0, 90% by v0.2.0.
- **Module-level coverage**: every module must have at least one test. Currently, every module includes `#[cfg(test)] mod tests` blocks. The weakest coverage is in `retrieval/vector.rs` (27 lines, no dedicated test module -- relies on `embeddings.rs` tests) and `retrieval/rerank.rs` (85 lines, tested only through `pipeline.rs` integration tests).
- **Doctest coverage**: zero compilable doctests currently exist on public methods. This is a known gap from the architecture review. Every `pub fn` on `AlayaStore` must have a compilable doctest before v0.1.0 publication.

**CI Integration** (GitHub Actions):

```yaml
# .github/workflows/ci.yml (coverage job)
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: taiki-e/install-action@cargo-llvm-cov
    - run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    - uses: codecov/codecov-action@v4
      with:
        files: lcov.info
        fail_ci_if_below: 80
```

#### 1.2.2 Benchmark Tracking

Alaya's core performance claims (sub-ms BM25 retrieval, <10ms hybrid retrieval at 10K episodes) must be continuously verified. Currently, no benchmark suite exists. This is a known gap, with `divan` benchmarks planned for v0.2.0.

**Pre-v0.2 minimum benchmark suite** (using `divan` or `criterion`):

| Benchmark | Measures | Baseline Target |
|-----------|----------|-----------------|
| `bench_store_episode` | Single episode insertion (content + FTS5 trigger + strength init) | < 500us |
| `bench_store_episode_with_embedding` | Insertion with 384-dim f32 embedding BLOB | < 1ms |
| `bench_query_bm25_100` | BM25-only query against 100 episodes | < 200us |
| `bench_query_bm25_10k` | BM25-only query against 10,000 episodes | < 2ms |
| `bench_query_hybrid_100` | Full pipeline (BM25 + vector + graph + RRF + rerank) against 100 episodes with embeddings | < 5ms |
| `bench_query_hybrid_10k` | Full pipeline against 10,000 episodes with embeddings | < 20ms |
| `bench_vector_cosine_10k` | Brute-force cosine search over 10,000 384-dim embeddings | < 15ms |
| `bench_consolidation_batch_10` | `consolidate()` with MockProvider on 10-episode batch | < 1ms (excluding provider time) |
| `bench_forget_sweep_1k` | `forget()` decay pass over 1,000 node strengths | < 5ms |
| `bench_spreading_activation_1k_links` | 3-hop activation from 3 seeds over 1,000 links | < 10ms |
| `bench_rrf_merge_3_lists_100` | RRF fusion of 3 ranked lists, 100 candidates each | < 50us |
| `bench_fts5_sanitize` | FTS5 input sanitization on adversarial string | < 1us |

**Regression detection**: Use `cargo-criterion` with `--save-baseline` on each CI run. Compare against stored baselines. Fail CI if any benchmark regresses by more than 20% from the stored baseline. This threshold accounts for CI runner variability while catching genuine regressions.

**Storage in CI**: Benchmark results should be committed to a `benches/results/` directory as JSON, with GitHub Actions caching the baseline across runs. Alternatively, use the `github-action-benchmark` action to track results over time and post comments on PRs with regression warnings.

#### 1.2.3 Clippy and Static Analysis

```yaml
# CI job: lint
lint:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
    - run: cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic
    - run: cargo fmt --check
```

The `-D clippy::pedantic` level is intentional. Alaya is a library consumed by other developers. Pedantic lints catch API design issues (missing `#[must_use]`, inconsistent naming, unnecessary allocations) that matter disproportionately in a library context. Specific pedantic lints that are too noisy for Alaya's codebase can be `#[allow]`-ed at the module level with a comment explaining why.

Current known clippy issues to address before v0.1.0 publication:

- Missing `#[non_exhaustive]` on `Role`, `SemanticType`, `LinkType`, `PurgeFilter`, `AlayaError` (ADR constraint)
- Missing `#[must_use]` on `AlayaStore::open()`, `AlayaStore::open_in_memory()`, `Query::simple()`
- `NodeRef::from_parts()` returns `Option` but could use a more descriptive error
- `Role::from_str()` and similar should implement `std::str::FromStr` trait instead of inherent method

#### 1.2.4 Dependency Auditing

Alaya's dependency tree is deliberately minimal (ADR-009: Zero Network Calls). The current dependencies are:

| Crate | Version | Contains `unsafe` | Network? | Role |
|-------|---------|:-----------------:|:--------:|------|
| `rusqlite` | 0.32 | Yes (FFI to SQLite C) | No | Core storage engine |
| `serde` | 1.x | Yes (proc macro) | No | Serialization derives |
| `serde_json` | 1.x | No | No | JSON for context/metadata |
| `thiserror` | 2.x | No | No | Error derive macro |

Audit tooling:

```bash
# Run on every CI build
cargo audit

# Run weekly (scheduled CI job)
cargo deny check advisories licenses sources
```

**`cargo-deny` configuration** (`deny.toml`):

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib"]
copyleft = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"

[bans]
# Enforce zero network dependencies in the dependency tree
deny = [
    { name = "reqwest" },
    { name = "hyper" },
    { name = "tokio", wrappers = ["rusqlite"] },  # rusqlite may pull tokio for async, deny it
    { name = "openssl" },
    { name = "rustls" },
]
```

The `[bans]` section is critical for enforcing ADR-009. If any transitive dependency pulls in a networking crate, the build must fail. This is the automated enforcement of "zero network calls in core crate" -- structural, not policy.

#### 1.2.5 MSRV (Minimum Supported Rust Version) Tracking

Alaya specifies `edition = "2021"` in Cargo.toml but does not declare an explicit MSRV. Before publication:

1. Add `rust-version = "1.70"` to `Cargo.toml` (or whatever the actual minimum is, determined by testing).
2. Add a CI job that tests against the declared MSRV:

```yaml
msrv:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@1.70.0
    - run: cargo check --all-features
```

3. Test MSRV whenever dependencies are updated. `rusqlite 0.32` requires at least Rust 1.70.0 due to its use of `rusqlite::Connection::open` with modern SQLite features.

#### 1.2.6 SQLite File Format Compatibility

Alaya stores all state in a single SQLite file. The schema (7 tables, 1 FTS5 virtual table, 3 triggers, 9 indexes) is initialized by `schema::init_db()` using `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`. This means:

- **Forward compatibility**: new versions can add tables/indexes without breaking old files.
- **Schema version tracking**: currently absent. Before v0.1.0, add a `schema_version` table or use SQLite's `user_version` PRAGMA.

```sql
-- Set during init_db()
PRAGMA user_version = 1;

-- Check on open
-- If user_version < current, run migrations
-- If user_version > current, return AlayaError::SchemaVersion
```

Track schema version in CI tests: open a database created by the previous release, verify it works with the current code. This prevents silent schema incompatibilities.

### 1.3 Metric Collection Summary

| Metric | Tool | Frequency | Threshold | Action on Breach |
|--------|------|-----------|-----------|------------------|
| Test coverage | `cargo-llvm-cov` | Every PR | < 80% | Block merge |
| Benchmark regression | `divan` + baseline comparison | Every PR | > 20% regression | Block merge |
| Clippy pedantic | `cargo clippy -D warnings` | Every PR | Any warning | Block merge |
| Format check | `cargo fmt --check` | Every PR | Any diff | Block merge |
| Dependency audit | `cargo audit` | Every PR + weekly | Any vulnerability | Block merge / issue |
| License check | `cargo deny` | Every PR | Copyleft or unknown | Block merge |
| Network dep ban | `cargo deny` `[bans]` | Every PR | Any banned crate | Block merge |
| MSRV | `cargo check` on MSRV toolchain | Every PR | Compile failure | Block merge |
| Doc build | `cargo doc --no-deps` | Every PR | Any warning | Block merge |
| Schema compat | Migration test against prior release DB | Every release | Open failure | Block release |

---

## 2. Release Quality Gates

### 2.1 The Gate Philosophy

`cargo publish` is irreversible. A yanked version still appears in lock files. A semver violation forces every downstream consumer through a painful upgrade cycle. The cost of a bad release for a library is orders of magnitude higher than a bad deployment for a service (which can be rolled back in minutes). Every release gate exists to prevent publishing something that cannot be unpublished.

### 2.2 Pre-Publication Checklist (v0.1.0)

This is the one-time checklist for the initial crates.io publication:

- [ ] All `pub fn` methods on `AlayaStore` have compilable doctests
- [ ] `#[non_exhaustive]` on all public enums (`Role`, `SemanticType`, `LinkType`, `PurgeFilter`, `AlayaError`)
- [ ] `#[must_use]` on constructor methods and methods returning `Result`
- [ ] `rust-version` field in `Cargo.toml`
- [ ] `CHANGELOG.md` exists with v0.1.0 section
- [ ] `LICENSE` file exists (MIT, matching Cargo.toml)
- [ ] `cargo doc --no-deps` produces no warnings
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] `cargo audit` reports no vulnerabilities
- [ ] `cargo deny check` passes
- [ ] `cargo publish --dry-run` succeeds
- [ ] README.md code examples compile (`skeptic` or manual verification)
- [ ] Schema version PRAGMA set to 1
- [ ] BEGIN IMMEDIATE for all write transactions (currently missing, known gap)
- [ ] Input validation at API boundary (currently missing, known gap)
- [ ] No `TODO` comments in public-facing code paths

### 2.3 Per-Release Quality Gates

Every subsequent release must pass these gates before `cargo publish`:

#### 2.3.1 Automated Gates (CI must pass)

| Gate | Command | Pass Criteria |
|------|---------|---------------|
| **Tests** | `cargo test --all-features` | 0 failures |
| **Doctests** | `cargo test --doc` | 0 failures |
| **Clippy** | `cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic` | 0 warnings |
| **Format** | `cargo fmt --check` | No diff |
| **Audit** | `cargo audit` | 0 vulnerabilities |
| **Deny** | `cargo deny check` | 0 denials |
| **Doc build** | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | 0 warnings |
| **MSRV** | `cargo +<msrv> check --all-features` | Compiles |
| **Dry run** | `cargo publish --dry-run` | Success |
| **Coverage** | `cargo llvm-cov --all-features` | >= 80% |
| **Benchmarks** | `cargo bench` vs stored baseline | No regression > 20% |
| **Schema compat** | Open prior-version DB, run status() | Success |

#### 2.3.2 Manual Gates (Maintainer review)

| Gate | Reviewer Action |
|------|----------------|
| **CHANGELOG.md updated** | Verify entry exists for new version |
| **Semver correctness** | Review diff for breaking changes; bump major/minor/patch appropriately |
| **Public API diff** | Run `cargo public-api diff` against prior release; verify no unintended changes |
| **Cross-reference docs** | Verify docs match current behavior (especially schema, API signatures) |
| **Feature flag matrix** | Test with each feature flag independently and all combined |

#### 2.3.3 Semver Verification

Use `cargo-semver-checks` to automate semver validation:

```bash
cargo semver-checks check-release --baseline-version <previous>
```

This catches:
- Removed public items
- Changed function signatures
- Removed trait implementations
- Changed enum variants (mitigated by `#[non_exhaustive]`)
- Changed struct fields

Run this as a required CI check on release branches.

### 2.4 Release Process

```
1. Create release branch: git checkout -b release/v0.X.Y
2. Update version in Cargo.toml
3. Update CHANGELOG.md
4. Run full gate suite locally: cargo test && cargo clippy && cargo doc ...
5. Push branch, wait for CI green
6. Create GitHub Release (tag v0.X.Y)
7. cargo publish
8. Verify crates.io page renders correctly
9. Verify docs.rs builds successfully
10. Post release announcement (if applicable)
```

### 2.5 Yanking Policy

Yank a release only when:

1. **Security vulnerability** in Alaya's own code (not upstream -- that is handled by advisory)
2. **Data corruption bug** that silently corrupts the SQLite file
3. **Semver violation** where a patch release contains breaking changes

Never yank for:
- Performance regressions (fix forward)
- Non-security bugs (fix forward)
- Documentation errors (fix forward)

---

## 3. Community Feedback Pipeline

### 3.1 The Feedback Reframe

A deployed service has Sentry for crashes, Mixpanel for usage analytics, and a support inbox. Alaya has none of these. ADR-009 (Zero Network Calls) means no telemetry, no crash reporting, no usage tracking. The North Star metric -- Monthly Active Crate Consumers (MACC) -- is measured through proxy signals, not direct instrumentation.

### 3.2 Proxy Signals for MACC

| Signal | Source | What It Indicates | Frequency |
|--------|--------|-------------------|-----------|
| crates.io recent downloads | `crates.io/crates/alaya` | Raw download volume (includes CI re-downloads) | Weekly |
| GitHub stars | `github.com/h4x0r/alaya` | Developer interest (not usage) | Weekly |
| GitHub dependents | GitHub dependency graph | Projects with `alaya` in Cargo.toml | Monthly |
| `lib.rs` stats | `lib.rs/crates/alaya` | Alternative download tracking | Monthly |
| GitHub issues opened | Issue tracker | Active consumer engagement | Continuous |
| Reverse dependency search | `cargo-crev` / GitHub code search | `use alaya::` in public repos | Monthly |
| Discussion forum activity | GitHub Discussions | Community questions and patterns | Continuous |

**MACC estimation formula**: MACC roughly equals GitHub dependents with recent commits (active projects depending on Alaya). This is imprecise but is the best available signal without telemetry.

### 3.3 Issue Triage Process

#### 3.3.1 Labels

| Label | Meaning | Response SLA |
|-------|---------|:------------:|
| `bug/data-corruption` | SQLite file corruption or silent data loss | 24 hours |
| `bug/semver-violation` | Public API changed in patch/minor release | 24 hours |
| `bug/security` | Security vulnerability (use private reporting) | 24 hours |
| `bug/incorrect-result` | Retrieval returns wrong results, lifecycle produces wrong output | 48 hours |
| `bug/panic` | Library panics (should never happen) | 48 hours |
| `bug/other` | Other bugs | 1 week |
| `feature/request` | New capability request | Acknowledge in 1 week |
| `feature/accepted` | Accepted for roadmap | - |
| `question` | Usage question | 1 week |
| `performance` | Performance regression or concern | 1 week |
| `docs` | Documentation issue | 1 week |
| `good-first-issue` | Suitable for new contributors | - |

#### 3.3.2 Severity Classification

**Critical (24-hour response)**:
- Data corruption: any code path where `store_episode()`, `consolidate()`, `transform()`, `forget()`, or `purge()` leaves the SQLite database in an inconsistent state. Example: FTS5 index out of sync with episodes table after a crash during `purge(PurgeFilter::All)`.
- Semver violations: a published patch release where `cargo update` breaks a downstream project.
- Security: memory poisoning amplification, FTS5 injection bypass, cross-user leakage when consumers follow documented isolation guidance.

**High (48-hour response)**:
- Panics: Alaya should never panic. Every error is returned as `Result<T, AlayaError>`. A panic in library code (not in tests) is a high-severity bug. The current codebase has one `unwrap()` in `lib.rs` line 35 (`path.as_ref().to_str().unwrap_or("alaya.db")`) which should be replaced with an `AlayaError::InvalidInput`.
- Incorrect retrieval results: BM25 returning irrelevant results, RRF fusion producing wrong rankings, Bjork strengths decaying incorrectly.

**Medium (1-week response)**:
- Performance regressions: query takes 10x longer than expected. Benchmark suite detects these before release, but consumers may have different workloads.
- Documentation inaccuracies: API docs that do not match behavior.

**Low (best-effort)**:
- Feature requests, cosmetic issues, CI improvements.

#### 3.3.3 Downstream Breakage Reports

When a consumer reports that upgrading Alaya broke their build or changed behavior:

1. **Reproduce**: Create a minimal reproduction case.
2. **Classify**: Is it a semver violation (our fault) or an expected breaking change in a major version (their upgrade path)?
3. **If semver violation**:
   - Publish a patch release restoring the old behavior.
   - Add a regression test.
   - Add the case to `cargo-semver-checks` CI if it was not caught.
4. **If expected major-version change**:
   - Ensure the migration guide covers the change.
   - Offer assistance in the issue thread.

### 3.4 Feature Request Evaluation

Every feature request is evaluated against the project axioms (from North Star):

1. **Privacy > Features**: Does the feature require network calls? Reject for core crate. Suggest as separate crate.
2. **Process > Storage**: Does the feature add cognitive lifecycle capabilities? Prioritize.
3. **Correctness > Speed**: Does the feature have research grounding? Prioritize. Is it ad-hoc? Deprioritize.
4. **Simplicity > Completeness**: Does the feature increase the public API surface significantly? Weigh carefully.
5. **Kill list check**: Is the request for cloud deployment, enterprise features, standalone service, procedural memory, or framework coupling? Politely decline with explanation.

Template response for kill-list requests:

> Thank you for the suggestion. Alaya is deliberately scoped as an embeddable library with zero external dependencies. [Feature X] falls outside that scope because [reason]. You might find [alternative] better suited for this use case. If you are interested in building this as a separate crate that depends on Alaya, I would be happy to discuss the integration points.

### 3.5 crates.io Reviews and Ecosystem Reputation

Monitor:
- crates.io page for the crate description, keywords, and categories accuracy
- docs.rs build status (must always be green)
- `lib.rs` quality score
- Mentions in Rust community spaces (r/rust, Rust users forum, Zulip, Discord)

---

## 4. Dependency and Algorithm Updates

### 4.1 The Update Reframe

A deployed service updates its container images and runtime dependencies. Alaya updates its crate dependencies. The stakes are different: an upstream crate update can change Alaya's MSRV, increase compile times, introduce `unsafe` code, or break API compatibility. Every dependency update is a deliberate decision.

### 4.2 Dependency Update Policy

#### 4.2.1 rusqlite (Critical Path)

rusqlite is Alaya's most important dependency. It provides the SQLite C library (via the `bundled` feature) and the Rust API. Updates to rusqlite can:

- Change the bundled SQLite version (affecting FTS5 behavior, WAL performance, PRAGMA behavior)
- Change the Rust API (breaking Alaya's internal code)
- Change MSRV requirements
- Change the `bundled` feature's compile behavior (affecting binary size and compile time)

**Update triggers**:
- SQLite security advisory affecting the bundled version: update immediately
- New SQLite version with FTS5 improvements or WAL bug fixes: evaluate within 2 weeks
- rusqlite minor/patch version with bug fixes: evaluate within 1 month
- rusqlite major version: plan migration, test extensively, release as Alaya minor (if no API break) or major version

**Update process**:
1. Read rusqlite changelog and linked SQLite changelog
2. Run full test suite with new version
3. Run benchmark suite, compare against baseline
4. Check MSRV impact
5. Check compile time impact (see Section 6)
6. Check binary size impact (see Section 6)
7. Test schema compatibility (open old DB with new rusqlite)

#### 4.2.2 serde / serde_json (Stable)

serde 1.x has a strong stability guarantee. Updates are low risk.

**Update triggers**: `cargo audit` advisory, or needed for a new derive feature.
**Process**: Update, run tests, publish.

#### 4.2.3 thiserror (Stable)

thiserror 2.x is compile-time only (proc macro). Updates are very low risk.

**Update triggers**: `cargo audit` advisory.
**Process**: Update, run tests, publish.

#### 4.2.4 Planned Dependencies (v0.2+)

| Dependency | Feature Flag | Version | Risk Level | Notes |
|------------|-------------|---------|------------|-------|
| `sqlite-vec` | `vec-sqlite` | v0.2 | Medium | SIMD vector search; replaces brute-force cosine |
| `ort` | `embed-ort` | v0.2 | High | ONNX Runtime; large binary, complex build |
| `fastembed-rs` | `embed-fastembed` | v0.2 | High | Wraps ort; simpler API but same binary cost |
| `tokio` | `async` | v0.2 | Medium | Async wrapper via `spawn_blocking` |
| `cbindgen` | build-time | v0.2 | Low | C header generation; build-only |

Each planned dependency must be gated behind a feature flag (ADR-004: Trait-Based Extension Model). The core crate with no feature flags must remain at the current dependency count (rusqlite + serde + serde_json + thiserror). This is non-negotiable. The `[bans]` section in `deny.toml` must be updated for each new feature flag to allow the new dependencies only when that flag is active.

### 4.3 Algorithm Updates

Alaya's retrieval and lifecycle processes are grounded in published research. Algorithm updates are changes to the mathematical models, not code refactoring.

#### 4.3.1 Retrieval Pipeline

| Component | Current Algorithm | Research Basis | Update Trigger |
|-----------|------------------|----------------|----------------|
| Text search | FTS5 BM25 (porter stemmer) | Robertson & Zaragoza (2009) | New stemmer or tokenizer improves recall on LoCoMo |
| Vector search | Brute-force cosine similarity | Standard | Scale ceiling hit (>50K embeddings); migrate to sqlite-vec |
| Fusion | Reciprocal Rank Fusion, k=60 | Cormack et al. (2009) | Benchmark shows alternative fusion outperforms |
| Rerank | `base * (1 + 0.3*context_sim) * (1 + 0.2*recency)` | Custom formula | LoCoMo benchmark tuning |
| Graph | Collins & Loftus spreading activation, 3 hops | Collins & Loftus (1975) | Benchmark shows different hop count or decay improves recall |

**Update process for algorithms**:
1. Implement the new algorithm behind a feature flag or as an alternative code path.
2. Benchmark against the current algorithm on a standardized dataset (LoCoMo or equivalent).
3. If the new algorithm wins on the target metric without regressing other metrics, replace the default.
4. Document the change in CHANGELOG.md with benchmark numbers.
5. If the change affects the retrieval API (different scores, different ordering), consider semver impact.

#### 4.3.2 Lifecycle Parameters

| Parameter | Current Value | Source | Update Trigger |
|-----------|--------------|--------|----------------|
| Consolidation batch size | 10 | Heuristic | Benchmark shows different batch size improves throughput |
| Consolidation min episodes | 3 | Heuristic | User feedback on "too few" or "too many" consolidations |
| Perfuming crystallization threshold | 5 impressions | Heuristic | User feedback on preference emergence speed |
| Transformation dedup threshold | 0.95 cosine similarity | Heuristic | False-positive or false-negative dedup reports |
| Transformation link prune threshold | 0.02 weight | Heuristic | Graph becomes too sparse or too dense |
| Preference half-life | 30 days | Heuristic | Preferences decay too fast or persist too long |
| Impression max age | 90 days | Heuristic | Stale impressions affect preference quality |
| Forgetting RS decay factor | 0.95 per cycle | Bjork (1992) | Memories decay too fast or not fast enough |
| Archive SS threshold | 0.1 | Bjork (1992) | Too many or too few nodes archived |
| Archive RS threshold | 0.05 | Bjork (1992) | Too many or too few nodes archived |
| LTP rate | 0.1, `w += 0.1 * (1-w)` | Hebbian learning | Co-retrieval strengthening too fast or too slow |
| RRF k parameter | 60 | Cormack et al. (2009) | Benchmark tuning |

**Exposure strategy**: These parameters are currently hardcoded constants. The v0.2 roadmap includes `AlayaConfig::builder()` to expose them as configuration. Until then, changes to these values are Alaya-version-level changes.

### 4.4 Automated Dependency Management

Use `dependabot` or `renovate` for automated PR creation when dependencies have updates. Configuration:

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
    reviewers:
      - "h4x0r"
```

Every dependabot PR must pass the full CI suite (tests, clippy, audit, benchmarks, MSRV check) before merge.

---

## 5. Breaking Change and Regression Response

### 5.1 The Incident Reframe

A deployed service has incidents: outages, data loss, performance degradation. An embeddable library has a different class of incidents: semver violations, data corruption bugs, performance regressions, and schema incompatibilities. The response process parallels incident management but operates on crate-publication timescales rather than minutes-to-restore timescales.

### 5.2 Severity Levels

| Level | Description | Response Time | Resolution Time | Example |
|-------|-------------|:------------:|:---------------:|---------|
| **SEV-1** | Data corruption in the SQLite file | 4 hours | 24 hours (yank + hotfix) | FTS5 trigger fails to cascade delete, leaving orphaned FTS entries that corrupt rebuild |
| **SEV-2** | Semver violation in published release | 4 hours | 24 hours (yank + hotfix) | Patch release removes a public method |
| **SEV-3** | Security vulnerability in Alaya code | 24 hours | 48 hours | FTS5 sanitization bypass |
| **SEV-4** | Incorrect lifecycle behavior | 48 hours | 1 week | `forget()` archives nodes that should be retained |
| **SEV-5** | Performance regression >3x | 1 week | Next release | BM25 query time doubles due to unintended table scan |

### 5.3 Semver Policy

Alaya follows Cargo semver conventions strictly:

| Version Component | When to Bump | Examples |
|-------------------|-------------|----------|
| **Major** (0.x -> 1.0, or 1.x -> 2.0) | Removing public items, changing signatures, changing behavior in incompatible ways, schema changes requiring migration | Remove `AlayaStore::purge()`, change `query()` return type, change SQLite schema without migration |
| **Minor** (0.1 -> 0.2) | New public API, new feature flags, new report fields, deprecations | Add `AlayaStore::compact()`, add `vec-sqlite` feature flag |
| **Patch** (0.1.0 -> 0.1.1) | Bug fixes, performance improvements, documentation, internal refactors | Fix FTS5 sanitization edge case, improve rerank formula |

**Special note on 0.x semver**: While Alaya is pre-1.0, Cargo allows minor versions to contain breaking changes. Alaya treats minor versions as if they were major versions anyway: no breaking changes in patch releases, and breaking changes are clearly documented in CHANGELOG.md even in minor releases. This builds trust with early adopters.

### 5.4 Data Migration Policy

The SQLite schema is Alaya's most critical compatibility surface. Schema changes must follow this protocol:

#### 5.4.1 Additive Changes (Non-Breaking)

Adding new tables, new indexes, new columns with defaults. These are handled by `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` in `schema::init_db()`.

Example: adding a `tombstones` table (planned for v0.1):
- Add `CREATE TABLE IF NOT EXISTS tombstones (...)` to `init_db()`
- Old databases gain the table on next `AlayaStore::open()`
- No migration needed; the `IF NOT EXISTS` pattern handles it

#### 5.4.2 Destructive Changes (Breaking)

Removing columns, renaming tables, changing column types. These require explicit migration:

1. Increment `PRAGMA user_version`
2. Write a migration function: `fn migrate_v1_to_v2(conn: &Connection) -> Result<()>`
3. Run migrations automatically on `AlayaStore::open()` when `user_version < CURRENT_SCHEMA_VERSION`
4. Test migration with a fixture database created by the prior version
5. Document in CHANGELOG.md with migration instructions for consumers who use raw SQLite access

#### 5.4.3 FTS5 Rebuild

FTS5 virtual tables cannot be altered. If the FTS5 schema changes (different tokenizer, different columns):

1. Drop and recreate the virtual table
2. Rebuild from source table: `INSERT INTO episodes_fts(episodes_fts) VALUES('rebuild')`
3. This is a potentially slow operation on large databases. Document expected time.

### 5.5 Hotfix Process

When a SEV-1 or SEV-2 issue is discovered in a published release:

```
1. Acknowledge the issue publicly (GitHub issue comment)
2. Create hotfix branch from the release tag: git checkout -b hotfix/v0.1.1 v0.1.0
3. Write a failing test that reproduces the bug
4. Fix the bug
5. Run full gate suite
6. Bump patch version in Cargo.toml
7. Update CHANGELOG.md
8. cargo publish
9. Yank the broken version (if data corruption or semver violation)
10. Post update to the issue thread with the fix version
```

**Post-mortem**: every SEV-1 or SEV-2 gets a brief post-mortem added to `docs/post-mortems/` covering:
- What happened
- Why it was not caught by existing gates
- What gate or test is being added to prevent recurrence

### 5.6 Known Regression Vectors

Based on the current codebase analysis, these are the most likely sources of regressions:

| Vector | Risk | Mitigation |
|--------|------|------------|
| FTS5 trigger sync | Episode deletion may leave FTS5 orphans if trigger fails | Integration test: insert, delete, verify FTS5 count matches |
| Schema init idempotency | `init_db()` runs on every `open()`. If a migration alters existing tables, re-running must be safe | Test: call `init_db()` twice on same connection |
| Embedding BLOB format | `f32` little-endian BLOBs. If serialization changes, old embeddings become garbage | Test: serialize/deserialize roundtrip with known bytes |
| Bjork strength decay | `RS *= 0.95` per cycle. Changing the constant or the formula silently changes forgetting behavior | Benchmark: verify decay curve over N cycles matches expected values |
| RRF fusion ordering | RRF is deterministic for a given set of ranked lists. Changing k or the merge logic changes result ordering | Test: fixed input lists, verify exact output ordering |
| Graph co-retrieval LTP | `on_co_retrieval()` creates or strengthens links between all retrieved pairs. O(n^2) in result set size. | Benchmark: query with max_results=10 should not create >45 link operations |
| `PurgeFilter::All` | Deletes from 7 tables in a single `execute_batch()`. No transaction boundary. | Wrap in explicit transaction; test: purge then status shows all zeros |

---

## 6. Compile Time and Binary Size Budget

### 6.1 The Cost Reframe

A deployed service has infrastructure costs: compute, storage, bandwidth. A library has developer-experience costs: compile time and binary size. Every dependency added to Alaya increases both. For agent developers who `cargo add alaya`, the compile time of Alaya becomes part of their development inner loop. Binary size matters for edge deployment, embedded systems, and WASM targets.

### 6.2 Current Baseline

Measure and record the baseline before v0.1.0 publication:

```bash
# Clean build time (no cache)
cargo clean && time cargo build --release 2>&1

# Incremental build time (touch lib.rs)
touch src/lib.rs && time cargo build --release 2>&1

# Binary size (release, stripped)
cargo build --release
strip target/release/libalaya.rlib  # or check .so/.dylib size
ls -lh target/release/libalaya.rlib

# Dependency count
cargo tree --depth 1 | wc -l
cargo tree | wc -l  # total transitive deps
```

**Expected baseline** (estimated, must be measured):

| Metric | Target | Notes |
|--------|--------|-------|
| Clean build (release) | < 60s | Dominated by rusqlite/SQLite C compilation |
| Incremental build (release) | < 10s | Only Alaya's Rust code |
| Library size (release, stripped) | < 5 MB | Most is bundled SQLite |
| Direct dependencies | 4 | rusqlite, serde, serde_json, thiserror |
| Transitive dependencies | < 30 | Mostly from rusqlite + serde |

### 6.3 Feature Flag Impact Analysis

Each planned feature flag adds dependencies and compile time. Budget allocation:

| Feature Flag | Added Dependencies | Estimated Compile Impact | Estimated Size Impact | Budget Decision |
|-------------|-------------------|--------------------------|----------------------|-----------------|
| `vec-sqlite` | sqlite-vec | +5s clean build | +200KB | Acceptable: replaces brute-force cosine |
| `embed-ort` | ort, ndarray | +30-60s clean build | +15-40MB (ONNX runtime) | Gated: heavy, document compile impact |
| `embed-fastembed` | fastembed-rs (wraps ort) | +30-60s clean build | +15-40MB | Gated: same as ort, simpler API |
| `async` | tokio (minimal features) | +15-20s clean build | +500KB | Acceptable: common in Rust async ecosystem |

**Rule**: No feature flag combination may exceed 120s clean build time on a 4-core CI runner. If a feature flag pushes past this budget, investigate:
1. Can the dependency be made optional within the feature flag?
2. Can build.rs pre-build some artifacts?
3. Should the feature be a separate crate instead of a feature flag?

### 6.4 Binary Size Budget by Target

| Target | Max Acceptable Size | Notes |
|--------|:-------------------:|-------|
| `x86_64-unknown-linux-gnu` (release, stripped) | 5 MB (core) | Edge servers, containers |
| `x86_64-apple-darwin` (release, stripped) | 5 MB (core) | macOS desktop agents |
| `aarch64-apple-darwin` (release, stripped) | 5 MB (core) | Apple Silicon |
| `wasm32-wasi` (if supported in future) | 3 MB (core) | Browser/WASM agents |

With `embed-ort` or `embed-fastembed` enabled, the ONNX Runtime adds 15-40MB. This is acceptable only because it is gated behind a feature flag that consumers explicitly opt into. The default (no feature flags) must stay under the 5MB budget.

### 6.5 Compile Time Monitoring in CI

Add a CI job that measures and reports compile time:

```yaml
compile-time:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Clean build timing
      run: |
        cargo clean
        /usr/bin/time -v cargo build --release 2>&1 | tee build-timing.txt
    - name: Check budget
      run: |
        # Extract wall clock time and compare against budget
        elapsed=$(grep "Elapsed" build-timing.txt | grep -oP '\d+:\d+\.\d+')
        # Fail if > 120 seconds
```

Track compile times over releases. If a release increases clean build time by more than 15% without a corresponding feature addition, investigate before publishing.

### 6.6 Dependency Tree Hygiene

Periodically audit the dependency tree for unnecessary transitive dependencies:

```bash
# Show full tree
cargo tree --all-features

# Show duplicate crates (same crate, different versions)
cargo tree --duplicates

# Show features enabled for each dependency
cargo tree --format '{p} {f}'
```

**Rules**:
- Zero duplicate crate versions in the dependency tree (indicates version conflicts)
- No transitive dependency should pull in a networking crate (enforced by `cargo deny`)
- Transitive dependencies with known `unsafe` code should be audited via `cargo crev`

---

## 7. Roadmap Checkpoints

### 7.1 The Checkpoint Reframe

A deployed service has quarterly business reviews. A library has roadmap checkpoints -- decision points where the maintainer evaluates whether to add features, stabilize the API, or change direction. Each checkpoint has explicit entry criteria (when to trigger the review) and exit criteria (what decision is made).

### 7.2 Phase Overview

From the North Star document:

| Phase | Focus | Duration | MACC Target |
|-------|-------|----------|:-----------:|
| **v0.1 MVP** | Core library, CRUD completeness, LoCoMo baseline, publication | 4-6 weeks | 5 |
| **v0.2 Ecosystem** | MCP server, benchmarks >75%, config, async, FFI, sqlite-vec | 6-8 weeks after v0.1 | 25 |
| **v0.3 Growth** | Python bindings, community, DEF CON, formal security audit | 3-6 months after v0.2 | 100 |
| **v1.0 Stability** | API freeze, LTS commitment, production-grade documentation | After v0.3 stabilizes | 500 |

### 7.3 Checkpoint 1: Pre-Publication (Now)

**Entry criteria**: Codebase compiles, tests pass, architecture documented.

**Review questions**:
1. Are all pre-publication gates satisfied? (Section 2.2)
2. Are the known gaps acceptable for v0.1.0? Specifically:
   - Missing `BEGIN IMMEDIATE` for write transactions (risk: deadlock under concurrent access)
   - Missing input validation at API boundary (risk: SQL injection is mitigated by parameterized queries, but garbage-in-garbage-out for content)
   - Missing `#[non_exhaustive]` on public enums (risk: adding variants is a breaking change)
   - Missing compilable doctests (risk: poor docs.rs experience)
   - Missing schema versioning (risk: silent schema incompatibility on upgrade)
3. Is the README accurate and compelling?
4. Have benchmark claims been verified? (Currently: no, benchmarks do not exist yet.)

**Exit criteria**: All pre-publication gates green. `cargo publish --dry-run` succeeds. Decision to publish or delay.

**Honest assessment**: Several known gaps (especially `#[non_exhaustive]`, BEGIN IMMEDIATE, and doctests) should be fixed before publishing v0.1.0. Publishing without these creates immediate technical debt that is hard to fix without a breaking change. Recommendation: fix these four items, then publish.

### 7.4 Checkpoint 2: Post-v0.1.0, 2-Week Review

**Entry criteria**: 2 weeks after initial publication.

**Review questions**:
1. How many GitHub issues have been filed? What categories?
2. Any data corruption or panic reports?
3. crates.io download count -- is there initial interest?
4. docs.rs building correctly?
5. Any consumer feedback on API ergonomics?
6. Did any consumer hit the known gaps that were deferred?

**Exit criteria**: Prioritized backlog for v0.1.x patch releases and v0.2 feature work.

### 7.5 Checkpoint 3: v0.2 Planning

**Entry criteria**: v0.1.x stable (no open SEV-1/SEV-2 issues), MACC >= 5 or strong interest signals.

**Review questions**:
1. Which feature flags should ship first? Priority order:
   - `AlayaConfig::builder()` (configuration) -- highest consumer demand, lowest risk
   - sqlite-vec (`vec-sqlite`) -- removes the 50K embedding ceiling
   - Async API (`async`) -- enables tokio-based agents
   - MCP server (`alaya-mcp` separate crate) -- universal agent integration
   - C FFI (`alaya-ffi` separate crate) -- cross-language embedding
   - Benchmarks (divan suite) -- credibility for Marcus persona
2. Has the LoCoMo benchmark been run? What is the current score?
3. Is the API surface correct? Any methods that should be renamed, removed, or restructured before v0.2?
4. Have consumers implemented `ConsolidationProvider`? What patterns emerged?

**Exit criteria**: Scoped v0.2 plan with prioritized feature list and timeline.

### 7.6 Checkpoint 4: v0.3 / Growth Decision

**Entry criteria**: v0.2 stable, MACC >= 25 or strong interest signals, benchmark results published.

**Review questions**:
1. Is the API stable enough to commit to 1.0?
2. Should Python bindings be a priority? (Depends on whether Rust-native consumers or Python agent developers are the primary audience.)
3. Is a formal security audit justified by adoption level?
4. DEF CON submission timeline and readiness.
5. Should Alaya remain a solo-maintainer project, or is it time to actively recruit contributors?

**Exit criteria**: Decision on v0.3 scope and 1.0 timeline.

### 7.7 Checkpoint 5: 1.0 Stability Decision

**Entry criteria**: v0.3 stable, MACC >= 100, no open API design concerns.

**Review questions**:
1. Is the public API surface finalized? Every public item will be a semver commitment.
2. Is the SQLite schema finalized? Schema migrations become permanent maintenance burden.
3. Is the documentation comprehensive enough for production users?
4. Is there a clear LTS (Long-Term Support) policy?
5. Has a formal security audit been completed?

**Exit criteria**: 1.0 release or decision to continue iterating at 0.x.

### 7.8 When to Add vs. When to Stabilize

The tension in every library is between adding features (to attract users) and stabilizing (to retain users). The decision framework:

**Add features when**:
- MACC is below target for the current phase
- The feature is on the kill-list of a competitor (competitive differentiation)
- Multiple consumers have requested the same capability
- The feature enables a new consumer category (MCP, Python, C FFI)
- The feature does not increase the default dependency count

**Stabilize when**:
- MACC exceeds the current phase target (consumers are happy with what exists)
- Open bug reports exceed open feature requests (quality debt)
- The API surface has grown beyond 20 public methods on `AlayaStore` (cognitive load ceiling)
- A major version bump is being considered (stabilize first, then release)
- The benchmark suite is incomplete (cannot verify stability without measurement)

### 7.9 Deprecation Policy

When a public item must be removed:

1. Mark with `#[deprecated(since = "0.X.0", note = "Use Y instead")]` in version N
2. Keep the deprecated item functional for at least one minor version
3. Remove in version N+1 (or N+2 for heavily-used items)
4. Document in CHANGELOG.md migration path
5. If the deprecated item is a method on `AlayaStore`, add a compile-time warning that guides the consumer to the replacement

---

## 8. Contributor Onboarding

### 8.1 The Onboarding Reframe

A deployed service onboards operators who manage infrastructure. A library onboards contributors who write code, documentation, and tests. The goal is to take a developer from "I want to contribute" to "my PR is merged" in a single session for small changes, or within a week for feature work.

### 8.2 Repository Structure

```
alaya/
  Cargo.toml                    # Package metadata, dependencies, features
  src/
    lib.rs                      # AlayaStore struct, public API surface (276 lines)
    types.rs                    # All public types (402 lines)
    error.rs                    # AlayaError enum (21 lines)
    schema.rs                   # SQLite schema, PRAGMAs, init (238 lines)
    provider.rs                 # ConsolidationProvider trait, NoOpProvider (78 lines)
    store/
      mod.rs                    # Module declarations
      episodic.rs               # Episode CRUD (177 lines)
      semantic.rs               # Semantic node CRUD (139 lines)
      implicit.rs               # Impressions + preferences CRUD (182 lines)
      embeddings.rs             # Embedding storage, cosine similarity (176 lines)
      strengths.rs              # Bjork dual-strength tracking (145 lines)
    graph/
      mod.rs                    # Module declarations
      links.rs                  # Hebbian link CRUD, LTP, co-retrieval (164 lines)
      activation.rs             # Spreading activation via recursive CTE (126 lines)
    retrieval/
      mod.rs                    # Module declarations
      bm25.rs                   # FTS5 BM25 search with sanitization (104 lines)
      vector.rs                 # Vector similarity delegation (27 lines)
      fusion.rs                 # Reciprocal Rank Fusion (68 lines)
      rerank.rs                 # Context-weighted reranking (85 lines)
      pipeline.rs               # Full query orchestration (143 lines)
    lifecycle/
      mod.rs                    # Module declarations
      consolidation.rs          # CLS replay (118 lines)
      perfuming.rs              # Vasana impression/preference pipeline (135 lines)
      transformation.rs         # Dedup, prune, decay (134 lines)
      forgetting.rs             # Bjork RS decay + archival (95 lines)
  north-star-advisor/           # Strategic documentation (this directory)
    docs/                       # Generated strategic documents
    .work-in-progress/          # Phase artifacts and structured outputs
    ai-context.yml              # Progressive context for document generation
```

### 8.3 Key Concepts a Contributor Must Understand

#### 8.3.1 Architectural Invariants

Before writing any code, a contributor must internalize these invariants:

1. **Single entry point**: All consumer interaction goes through `AlayaStore`. No public functions exist outside of `AlayaStore` methods and type constructors. Internal modules (`store/`, `graph/`, `retrieval/`, `lifecycle/`) expose `pub` functions, but these are `pub` for intra-crate use, not for consumers. (Rust's module visibility handles this correctly since they are not re-exported from `lib.rs`.)

2. **Everything returns `Result<T, AlayaError>`**: No panics in library code. No `unwrap()` except in tests. The single `unwrap_or` in `lib.rs:35` (`path.as_ref().to_str().unwrap_or("alaya.db")`) is a known exception that should be fixed.

3. **Connection is &self, not &mut self**: `AlayaStore` methods take `&self` because `rusqlite::Connection` handles internal mutability. This means the API surface looks immutable, even though writes are happening inside SQLite. This is a deliberate design choice for ergonomics but means contributors must not assume `&self` methods are read-only.

4. **Zero network calls**: No dependency in the core crate makes network calls. Enforced by `cargo deny`. A contributor who needs to add a dependency must verify it does not transitively pull in any networking crate.

5. **Provider trait boundary**: The `ConsolidationProvider` trait is the extension point. Alaya never calls an LLM. The consuming agent implements the trait and passes it to `consolidate()` and `perfume()`. Contributors should not add LLM logic to Alaya.

6. **SQL safety**: All SQL uses parameterized queries (`?1`, `?2`, etc.). No string interpolation in SQL. FTS5 MATCH input is sanitized by stripping non-alphanumeric characters. Contributors modifying SQL must maintain these invariants.

#### 8.3.2 The Three Stores

| Store | Table(s) | Input Type | Output Type | Lifecycle Process |
|-------|----------|------------|-------------|-------------------|
| Episodic | `episodes`, `episodes_fts` | `NewEpisode` | `Episode` | Source for consolidation |
| Semantic | `semantic_nodes` | `NewSemanticNode` | `SemanticNode` | Created by consolidation |
| Implicit | `impressions`, `preferences` | `NewImpression` | `Impression`, `Preference` | Created by perfuming |

Shared infrastructure spans all stores:
- `embeddings` table: polymorphic (any `node_type` + `node_id`)
- `links` table: polymorphic (any `NodeRef` source + target)
- `node_strengths` table: polymorphic (any `NodeRef`)

#### 8.3.3 The Retrieval Pipeline

```
Query
  |
  +-- BM25 (FTS5 MATCH on episodes_fts)
  |
  +-- Vector (brute-force cosine on embeddings table)
  |
  +-- Graph (spreading activation from top BM25+vector seeds)
  |
  v
RRF Fusion (k=60, merges ranked lists)
  |
  v
Rerank (context similarity + recency decay)
  |
  v
Post-retrieval (strength updates + co-retrieval LTP)
  |
  v
Vec<ScoredMemory>
```

Each stage is implemented in a separate module under `retrieval/`. The pipeline orchestrator is `retrieval/pipeline.rs::execute_query()`. Contributors adding a new retrieval signal should:
1. Implement the signal as a new module returning `Vec<(NodeRef, f64)>`
2. Add it to the pipeline in `execute_query()`
3. Include it in the RRF fusion input
4. Add a benchmark

#### 8.3.4 The Lifecycle Processes

| Process | Module | Entry Point | What It Does |
|---------|--------|-------------|--------------|
| Consolidation | `lifecycle/consolidation.rs` | `consolidate(conn, provider)` | Batch unconsolidated episodes, call `provider.extract_knowledge()`, store `SemanticNode`s, create links |
| Perfuming | `lifecycle/perfuming.rs` | `perfume(conn, interaction, provider)` | Call `provider.extract_impressions()`, store impressions, crystallize preferences from accumulated impressions |
| Transformation | `lifecycle/transformation.rs` | `transform(conn)` | Deduplicate semantic nodes, prune weak links, decay preferences, prune old impressions |
| Forgetting | `lifecycle/forgetting.rs` | `forget(conn)` | Decay retrieval strength for all nodes, archive nodes below both thresholds |

All lifecycle processes are explicit calls (ADR-008: Sync-First API). The agent controls when they run. Contributors should never add background threads or timers.

### 8.4 Development Environment Setup

```bash
# Clone
git clone https://github.com/h4x0r/alaya.git
cd alaya

# Verify build (first build compiles SQLite C from source, takes ~30s)
cargo build

# Run tests
cargo test

# Run specific module tests
cargo test --lib store::episodic
cargo test --lib lifecycle::forgetting

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build docs locally
cargo doc --no-deps --open
```

**No additional setup required.** The `bundled` feature on rusqlite means SQLite is compiled from source. No system-level SQLite installation needed. No database server to start. No configuration files to create.

### 8.5 Common Contribution Paths

#### 8.5.1 Adding a Test (Good First Issue)

The easiest contribution. Pick a module with low test coverage:

1. Open `src/retrieval/rerank.rs` (85 lines, no dedicated test module)
2. Add `#[cfg(test)] mod tests { ... }`
3. Write tests for edge cases: empty candidate list, single candidate, all same score, extreme recency values
4. Run `cargo test --lib retrieval::rerank`
5. Submit PR

#### 8.5.2 Adding a Doctest

Every `pub fn` on `AlayaStore` needs a compilable doctest:

```rust
/// Store a conversation episode with full context.
///
/// # Examples
///
/// ```
/// use alaya::{AlayaStore, NewEpisode, Role, EpisodeContext};
///
/// let store = AlayaStore::open_in_memory()?;
/// let id = store.store_episode(&NewEpisode {
///     content: "I prefer dark mode".to_string(),
///     role: Role::User,
///     session_id: "session-1".to_string(),
///     timestamp: 1700000000,
///     context: EpisodeContext::default(),
///     embedding: None,
/// })?;
/// # Ok::<(), alaya::AlayaError>(())
/// ```
pub fn store_episode(&self, episode: &NewEpisode) -> Result<EpisodeId> {
```

#### 8.5.3 Fixing a Known Gap

The architecture document lists known gaps. Each is a well-scoped task:

| Gap | Estimated Effort | Files to Change |
|-----|:----------------:|-----------------|
| Add `#[non_exhaustive]` to public enums | 30 min | `types.rs`, `error.rs` |
| Add `BEGIN IMMEDIATE` for writes | 2 hours | `store/*.rs`, `lifecycle/*.rs`, `graph/links.rs` |
| Add input validation at API boundary | 4 hours | `lib.rs` (validate in each public method) |
| Add schema versioning via `PRAGMA user_version` | 2 hours | `schema.rs` |
| Add tombstone mechanism | 1 day | New `store/tombstones.rs`, modify `purge()`, `consolidate()` |
| Add WAL checkpoint management | 1 hour | `schema.rs` (add PRAGMAs) |
| Call LTD from retrieval pipeline | 2 hours | `retrieval/pipeline.rs`, `graph/links.rs` |

#### 8.5.4 Adding a Feature Flag

For v0.2+ contributions:

1. Add the feature to `Cargo.toml` under `[features]`
2. Gate the new dependency with `#[cfg(feature = "...")]`
3. Gate the new code paths with `#[cfg(feature = "...")]`
4. Add CI matrix entry to test the feature flag independently
5. Update `deny.toml` to allow the new dependency under the feature flag
6. Update README.md with feature flag documentation
7. Measure compile time and binary size impact (Section 6)

### 8.6 Code Style and Conventions

| Convention | Rule |
|------------|------|
| Naming | `snake_case` functions, `CamelCase` types, `SCREAMING_SNAKE` constants |
| SQL | Uppercase keywords, lowercase identifiers, `?1` parameters (never string interpolation) |
| Error handling | Return `Result<T, AlayaError>`. Use `?` operator. No `unwrap()` in library code |
| Comments | Explain "why", not "what". Inline comments for non-obvious SQL or algorithms |
| Module tests | Every module has `#[cfg(test)] mod tests { ... }` at the bottom |
| Test naming | `test_<what>_<scenario>` (e.g., `test_forget_empty_db`, `test_cosine_similarity_orthogonal`) |
| Imports | Use `crate::` for internal imports, not `super::` (except in test modules) |
| Visibility | `pub` only on `AlayaStore` methods and types. Internal modules use `pub(crate)` or module-scoped `pub` |

### 8.7 Pull Request Process

1. Fork the repository
2. Create a branch from `main`
3. Make changes
4. Run `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt`
5. Write a clear PR description explaining what and why
6. Link to the relevant issue (if any)
7. Wait for CI to pass
8. Address review feedback
9. Maintainer merges via squash-merge

**PR size guideline**: Prefer small, focused PRs. A PR should touch one logical concern. "Add doctests to all AlayaStore methods" is one PR. "Add BEGIN IMMEDIATE + input validation + schema versioning" is three PRs.

### 8.8 Research Context for Contributors

Alaya's architecture is grounded in published research. Contributors who want to understand the "why" behind design decisions should read:

| Topic | Paper | How Alaya Uses It |
|-------|-------|-------------------|
| Complementary Learning Systems | McClelland et al. (1995) | Consolidation: fast episodic store + slow semantic extraction |
| Bjork Dual-Strength Model | Bjork & Bjork (1992) | Forgetting: storage strength vs retrieval strength |
| Spreading Activation | Collins & Loftus (1975) | Graph overlay: associative retrieval via recursive CTE |
| Reciprocal Rank Fusion | Cormack, Clarke & Buettcher (2009) | Retrieval: merging BM25 + vector + graph ranked lists |
| Retrieval-Induced Forgetting | Anderson, Bjork & Bjork (1994) | Post-retrieval: competitors of retrieved memory suppressed |
| Hebbian Learning | Hebb (1949) | Graph: links strengthen on co-activation (co-retrieval) |
| Vasana (Yogacara) | Vasubandhu, Trimsika-vijnaptimatrata | Perfuming: gradual impression accumulation, preference crystallization |
| Asraya-paravrtti (Yogacara) | Vasubandhu, Trimsika-vijnaptimatrata | Transformation: periodic restructuring toward clarity |

Contributors do not need to read these papers to contribute tests, documentation, or bug fixes. But understanding them helps for feature work on the retrieval pipeline, lifecycle processes, or graph overlay.

---

## Appendix A: CI Configuration Reference

The complete GitHub Actions workflow combining all quality gates:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo fmt --check

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v1

  msrv:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.70.0
      - run: cargo check --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info
```

---

## Appendix B: Incident Response Checklist

For SEV-1 or SEV-2 incidents (data corruption or semver violation):

```
[ ] Acknowledge the issue (GitHub comment within 4 hours)
[ ] Reproduce with minimal test case
[ ] Write failing test
[ ] Fix the bug
[ ] Verify fix against the minimal test case
[ ] Run full CI gate suite
[ ] Bump patch version in Cargo.toml
[ ] Update CHANGELOG.md
[ ] cargo publish
[ ] Yank broken version (if applicable)
[ ] Update issue with fix version
[ ] Write post-mortem (what happened, why not caught, what gate is added)
[ ] Add regression test to CI
```

---

## Appendix C: Dependency Update Checklist

When updating any dependency:

```
[ ] Read upstream changelog
[ ] Check for MSRV changes
[ ] cargo test --all-features
[ ] cargo clippy --all-targets --all-features -- -D warnings
[ ] cargo audit
[ ] cargo deny check
[ ] Compare benchmark results against baseline
[ ] Compare compile time against baseline
[ ] Compare binary size against baseline
[ ] Test schema compatibility (open prior-version DB)
[ ] Update CHANGELOG.md if user-visible impact
```

---

## Appendix D: Release Cadence

| Release Type | Cadence | Examples |
|-------------|---------|---------|
| Patch (0.1.x) | As needed for bug fixes | Security fix, crash fix, doc fix |
| Minor (0.x.0) | Every 6-8 weeks during active development | New feature flag, new API method, algorithm improvement |
| Major (x.0.0) | When API stability demands it | 1.0 commitment, schema-breaking migration |

There is no fixed release schedule. Releases happen when there is something worth releasing and all gates pass. A release with zero user-visible changes should not be published.

---

## Appendix E: Glossary of Alaya-Specific Terms

| Term | Meaning | Context |
|------|---------|---------|
| **Episode** | A single conversation turn stored with full context | Episodic store |
| **Semantic node** | Extracted knowledge distilled from episodes | Semantic store, created by consolidation |
| **Impression** | A behavioral observation extracted from an interaction | Implicit store, raw input to perfuming |
| **Preference** | A crystallized behavioral pattern with confidence score | Implicit store, output of perfuming |
| **NodeRef** | Polymorphic reference to any node type (episode, semantic, preference) | Graph overlay |
| **Consolidation** | CLS-inspired process converting episodes to semantic knowledge | Lifecycle |
| **Perfuming** | Vasana-inspired process accumulating impressions into preferences | Lifecycle |
| **Transformation** | Deduplication, pruning, and decay of stored knowledge | Lifecycle |
| **Forgetting** | Bjork dual-strength decay and archival of weak nodes | Lifecycle |
| **Dream cycle** | The combination of consolidate + forget + transform | Consumer pattern |
| **Provider** | Agent-implemented trait supplying LLM-dependent logic | Extension boundary |
| **Storage strength (SS)** | How well-learned a memory is (monotonically increases) | Bjork model |
| **Retrieval strength (RS)** | How accessible a memory is right now (decays over time) | Bjork model |
| **LTP** | Long-Term Potentiation: graph link strengthening on co-retrieval | Hebbian learning |
| **LTD** | Long-Term Depression: graph link weakening through disuse | Hebbian learning |
| **RRF** | Reciprocal Rank Fusion: merging multiple ranked lists | Retrieval |
| **RIF** | Retrieval-Induced Forgetting: suppressing competitors of retrieved memories | Post-retrieval |
| **MACC** | Monthly Active Crate Consumers: North Star metric | Strategy |
