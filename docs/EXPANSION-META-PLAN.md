# Expansion Meta-Plan: Round <ROUND>

This is a parameterized plan template for expanding the perlchecker Perl subset.
Instantiate by replacing `<ROUND>` with the round number and `<FEATURE>` with
the proposed expansion.

---

## Phase 1: PERL DEV MODE

**Agent:** Opus 4.6 (shared with CHECKER and JUDGE)

**Input:** Current grammar, existing features, prior rounds' outcomes.

**Task:** Propose ONE small, SMT-tractable expansion to the Perl subset.

**Constraints:**
- Must be useful in real Perl code
- Must be expressible in Z3 (integer arithmetic, bitvectors, strings, arrays theory)
- Must not require unbounded iteration or complex heap modeling
- Must not duplicate anything already implemented or proposed in prior rounds

**Output:** One-sentence proposal of `<FEATURE>`.

---

## Phase 2: CHECKER MODE

**Agent:** Same as PERL DEV (Opus 4.6, same context).

**Input:** The proposal from PERL DEV MODE.

**Task:** Validate the proposal against:
1. **SMT tractability** — Can Z3 handle this without timeouts or undecidability?
2. **Grammar fit** — Does it extend the PEG grammar cleanly?
3. **Implementation complexity** — How many layers need changes (parser only? AST+IR+symexec+SMT?)
4. **Interaction with existing features** — Does it break or conflict with anything?

**Output:** One-sentence assessment of feasibility and complexity.

---

## Phase 3: JUDGE MODE

**Agent:** Same as PERL DEV and CHECKER (Opus 4.6, same context).

**Input:** The proposal and the checker's assessment.

**Task:** Make a GO/NO-GO decision.

**Decision criteria:**
- GO if: tractable, clean grammar fit, bounded implementation effort, real-world utility
- NO-GO if: undecidable, requires major refactoring, marginal utility, conflicts with existing design

**Output:** `GO` or `NO-GO` with one-sentence justification.

**If NO-GO:** Return to PERL DEV MODE with feedback. PERL DEV proposes a different feature. Repeat until GO.

---

## Phase 4: IMPLEMENTER MODE (with QA loop)

**Agent:** Haiku (dedicated agent, separate from DEV/CHECKER/JUDGE).

**Input:** The approved `<FEATURE>` proposal, current codebase state, grammar, and architecture.

**Internal loop:** PLAN → IMPLEMENT → QA → repeat until QA passes.

### 4a: PLAN sub-phase

Decide which files need changes:
- `src/parser/perl_subset.pest` — grammar rules
- `src/parser/mod.rs` — parse tree → AST conversion
- `src/ast/mod.rs` — AST types, type checker
- `src/ir/mod.rs` — SSA/IR lowering
- `src/symexec/mod.rs` — symbolic execution
- `src/smt/mod.rs` — Z3 encoding
- `tests/cli_check.rs` — integration tests
- `examples/round<ROUND>_*.pl` — example Perl files

Identify the minimal set of changes. Prefer parser-level desugaring when possible (zero changes to IR/SMT).

### 4b: IMPLEMENT sub-phase

Make the code changes. Requirements:
- Each expansion MUST include at least one example file (`examples/round<ROUND>_dev.pl`) with annotated functions that exercise the new feature
- Example must have `# sig:`, `# pre:`, `# post:` annotations
- Functions must be verifiable (postconditions must hold)

### 4c: QA sub-phase

Run verification:
1. `cargo build` — must compile without errors or warnings
2. `cargo test` — all existing unit tests must pass
3. `cargo test --test cli_check` — all existing integration tests must pass
4. `cargo run --quiet -- check examples/round<ROUND>_dev.pl` — new examples must produce `verified`

**If QA fails:** Identify the failure, return to IMPLEMENT sub-phase with the error context. Fix and re-run QA.

**If QA passes:** Report success. Round <ROUND> is complete.

---

## Phase 5: ORCHESTRATOR MODE

**Agent:** Dedicated subagent (spawned fresh for each round by the main session).

Each round gets its own ORCHESTRATOR subagent. The main session spawns one subagent per round and waits for it to complete before spawning the next.

**The ORCHESTRATOR subagent executes all phases internally:**
1. Run PERL DEV MODE — propose a feature
2. Run CHECKER MODE — validate feasibility
3. Run JUDGE MODE — GO/NO-GO decision (loop back to step 1 if NO-GO)
4. Run IMPLEMENTER+QA MODE — PLAN → IMPLEMENT → QA loop until passing
5. Run final `cargo test` to confirm nothing is broken
6. Commit with message: `Round <ROUND>: <one-line feature description>`
7. Report round outcome in one sentence back to main session

**Main session responsibilities:**
- Spawn one ORCHESTRATOR subagent per round, sequentially
- Pass the round number, current feature list, and grammar to the subagent
- Collect results and track cumulative progress
- If a round fails after 3 QA retries, mark as NOT IMPLEMENTED and move on

---

## Instantiation Template

To run round N, replace:
- `<ROUND>` → the round number (e.g., `5`)
- `<FEATURE>` → filled in by PERL DEV MODE during execution

## Round Outcome Record

| Round | Feature | Status | Notes |
|-------|---------|--------|-------|
| 1 | scalar(@arr) | IMPLEMENTED | Array length via companion __len vars |
| 2 | Ternary (?:) | IMPLEMENTED | ITE in IR/SMT |
| 3 | String comparison (lt/gt/le/ge) | IMPLEMENTED | BoolExpr::StrCmp |
| 4 | abs() builtin | IMPLEMENTED | IntExpr::Abs → Z3 ite |
| <ROUND> | <FEATURE> | PENDING | |
