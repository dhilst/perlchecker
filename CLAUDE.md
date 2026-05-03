# CLAUDE.md — perlchecker

## Project

Rust-based symbolic verification tool for a restricted Perl subset.
Pipeline: pest PEG → AST → type checker → SSA/IR → CFG → symbolic execution → SMT (Z3).

## Build & Test

```
cargo build
cargo test                          # all unit + integration tests
cargo test --test cli_check         # integration tests only
cargo run --quiet -- check FILE.pl  # verify a Perl file
```

## Git

- Use `--no-gpg-sign` for all commits in this repo.
- Commit message format: `Round N: <one-line feature description>`

## Expansion Loop Protocol

When expanding the Perl subset, follow `docs/EXPANSION-META-PLAN.md` strictly.

**Architecture:** The main session spawns ONE ORCHESTRATOR subagent per round. Each subagent runs all modes internally and is discarded after the round completes. Rounds run sequentially — never in parallel.

### ORCHESTRATOR subagent (one per round)

The subagent executes these phases in order:

**1. PERL DEV MODE** — Propose ONE small, SMT-tractable expansion. Constraints:
- Useful in real Perl
- Expressible in Z3
- No unbounded iteration or heap modeling
- No duplicates of prior rounds
- Output: one-sentence proposal

**2. CHECKER MODE** — Validate against: SMT tractability, grammar fit, implementation complexity, interaction with existing features. Output: one-sentence assessment.

**3. JUDGE MODE** — GO/NO-GO decision. GO if tractable + clean grammar + bounded effort + real utility. If NO-GO: loop back to PERL DEV MODE.

**4. IMPLEMENTER + QA MODE** — Internal loop: **PLAN → IMPLEMENT → QA → repeat until QA passes.**
- PLAN: identify minimal file set to change
- IMPLEMENT: make changes + create `examples/roundN_dev.pl` with annotated functions
- QA: run `cargo build`, `cargo test`, `cargo run --quiet -- check examples/roundN_dev.pl`
- If QA fails: fix and re-run. Max 3 retries, then report NOT IMPLEMENTED.

**5. COMMIT** — Run `cargo test` final time, commit with `Round N: <feature>`, report one-sentence outcome.

### Main session responsibilities

- Spawn one ORCHESTRATOR subagent per round with: round number, prior features list, current grammar
- Wait for completion, collect result
- Track cumulative progress across rounds

## Key Files

| Path | Role |
|------|------|
| `src/parser/perl_subset.pest` | PEG grammar |
| `src/parser/mod.rs` | Parse tree → AST |
| `src/ast/mod.rs` | AST types + type checker |
| `src/ir/mod.rs` | SSA/IR lowering |
| `src/symexec/mod.rs` | Symbolic execution |
| `src/smt/mod.rs` | Z3 encoding |
| `src/limits.rs` | Safety limits (CLI-configurable) |
| `tests/cli_check.rs` | Integration tests |
| `examples/` | Test Perl files |
| `docs/EXPANSION-META-PLAN.md` | Meta-plan template |
| `docs/ROUNDS-5-9-PLAN.md` | Current round plans |
