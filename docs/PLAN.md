# Incremental v1 Plan for `perlchecker`

## Summary
Build a new Rust binary crate named `perlchecker` with a single initial command, `perlchecker check <file.pl>`, and grow it in small verified stages from extraction to full symbolic checking. v1 supports only annotated Perl subs with `I64` parameters/return, assignments, `if`, `return`, arithmetic `+ - *`, comparisons, and boolean operators. Verification means partial correctness only: for every explored terminating path satisfying `# pre:`, prove the returned value satisfies `# post:`.

## Key Changes
- Bootstrap a new Cargo binary crate with `src/cli`, `extractor`, `parser`, `ast`, `ir`, `symexec`, `smt`, and `annotations`; add `clap`, `thiserror`, `tracing`, `tracing-subscriber`, `pest`, `pest_derive`, `proptest`, and a Rust Z3 binding.
- Treat Rust as an external prerequisite for implementation in this environment; the plan should assume `rustup`/`cargo` are installed before coding. Use the system `z3` already present at `/usr/bin/z3` rather than planning vendored SMT binaries.
- Lock the annotation contract to one contiguous block immediately above `sub NAME { ... }` with exactly:
  - required `# sig: (I64, I64, ...) -> I64`
  - optional single `# pre: ...`
  - required single `# post: ...`
  - reject `# pos:` and other aliases in v1
- Define the extraction boundary first:
  - scan full `.pl` source line-by-line
  - record annotation lines
  - accept only `sub NAME {` declarations
  - capture the whole body by brace matching
  - return `ExtractedFunction { name, annotations, body, start_line }`
- Define annotation parsing as a separate stage that produces:
  - `FunctionSpec { name, arg_types, ret_type, pre, post }`
  - `Type` with only `I64` in v1
  - a shared expression AST for `pre`/`post` so annotation and code expressions use the same operator semantics
- Parse extracted function bodies with `pest` into:
  - `FunctionAst { name, params, body }`
  - statements limited to parameter unpacking from `@_`, local declarations/assignments, `if`, and `return`
  - explicit parser errors with source line/column context
- Add a semantic analysis stage before IR:
  - require `my ($x, $y, ...) = @_;` as the first executable statement
  - ensure parameter count matches `# sig`
  - reject undeclared locals, unsupported constructs, and any non-`I64` usage
  - require every explored branch to end in `return` for the parser/typechecker to accept the function shape, but keep verification semantics as partial correctness only
- Lower AST to a small internal IR in two steps:
  - SSA conversion with versioned variable names and phi-free block parameters or explicit merge assignments
  - CFG construction over basic blocks with conditional edges and a single exit block per `return`
- Symbolic execution:
  - use a worklist over CFG states
  - state contains symbolic environment, path condition, and current block/program point
  - fork on branch conditions, accumulate constraints, and prune unsat paths eagerly through SMT checks
  - at each feasible return, check `path_condition ∧ pre ∧ ¬post[result := returned_expr]`
- SMT integration:
  - encode `I64` and boolean expressions directly into Z3
  - produce either `Verified` or `Counterexample { function, assignments }`
  - map models back only to user-visible parameters in v1
- CLI rollout by milestone:
  - Milestone 1: file read + extraction summary
  - Milestone 2: parsed annotations + parsed AST diagnostics
  - Milestone 3: SSA/CFG debug output behind tracing
  - Milestone 4: symbolic execution without SMT-backed proof
  - Milestone 5: full verify/counterexample results
  - Final output format:
    - `✔ foo: verified`
    - `✘ bar: counterexample found`
    - followed by concrete parameter assignments on failure

## Public Interfaces and Types
- CLI:
  - `perlchecker check <file.pl>`
  - non-zero exit if the file cannot be parsed/validated or any annotated function fails verification
- Core data types:
  - `ExtractedFunction { name: String, annotations: Vec<String>, body: String, start_line: usize }`
  - `FunctionSpec { name: String, arg_types: Vec<Type>, ret_type: Type, pre: Expr, post: Expr }`
  - `FunctionAst { name: String, params: Vec<String>, body: Vec<Stmt> }`
  - `Type::I64`
  - `VerificationResult::{Verified, Counterexample}`
- Diagnostics:
  - structured error enums per stage with `thiserror`
  - include function name and source location whenever available
  - tracing spans for extraction, parse, typecheck, SSA, CFG, symexec, and SMT query steps

## Test Plan
- Unit tests from the first commit for extraction, annotation parsing, AST parsing, type checking, SSA versioning, CFG edges, symbolic branching, SMT satisfiable/unsatisfiable cases, and model-to-parameter mapping.
- Property-based tests with `proptest` for:
  - expression parser precedence/associativity stability
  - SSA invariant: assigned SSA names are unique
  - symbolic execution consistency against direct evaluation for small generated straight-line programs
- Golden-style CLI tests for:
  - no annotated functions
  - multiple annotated functions
  - malformed annotation block
  - unsupported Perl construct
  - verified function
  - counterexample output
- Acceptance scenarios:
  - function with satisfied postcondition across both branches verifies
  - function with one failing branch yields a concrete model
  - infeasible failing branch is pruned and does not report a false counterexample

## Assumptions and Defaults
- New project: no existing codebase or repo conventions need to be preserved.
- Rust toolchain is not installed in the current environment, so implementation must begin with local toolchain setup before coding.
- v1 is intentionally strict: unsupported syntax is rejected early rather than approximated.
- Only annotated functions are analyzed; unannotated subs are ignored.
- Preconditions apply only to verification, not to parsing/typechecking; postconditions may reference `$result` only, plus parameters/locals that are in scope by the end of the function.
