# Perl Symbolic Execution Engine (Rust) — Incremental Implementation Plan

## 🎯 Goal

Build a symbolic execution engine that:

* Reads `.pl` files
* Extracts annotated functions
* Parses a restricted Perl subset
* Converts to IR (SSA + CFG)
* Symbolically executes paths
* Uses Z3 to verify correctness
* Outputs counterexamples

---

# 🧱 Phase 0 — Scope Definition (LOCK THIS)

## Supported Language (v1)

* Functions:

  ```
  sub foo {
    my ($x, $y) = @_;
    ...
  }
  ```

* Types:

  * `I64` only

* Control flow:

  * `if`
  * `return`
  * assignments

* Expressions:

  * `+ - *`
  * comparisons: `< <= > >= == !=`
  * boolean: `&& || !`

## Forbidden (v1)

* loops
* arrays / hashes
* function calls
* division
* strings
* globals
* implicit variables (`$_`)

---

# 🧱 Phase 1 — CLI + Project Skeleton (START HERE)

## Tools

* clap
* thiserror
* tracing

---

## CLI Command

```
tool check file.pl
```

---

## Behavior (initial)

* Read file
* Print:

  ```
  Found N annotated functions
  ```

---

## Project Structure

```
src/
  cli/
  extractor/
  parser/
  ast/
  ir/
  symexec/
  smt/
  annotations/
```

---

# 🧪 Testing Setup (FROM DAY 1)

## Unit Testing

Built-in Rust:

```rust
#[test]
fn test_extraction() { ... }
```

---

## Property-Based Testing

Use: proptest

---

## Use Cases

* Expression parser correctness
* AST → SSA invariants
* Symbolic execution consistency

Example:

```rust
proptest! {
  #[test]
  fn addition_is_commutative(a in any::<i64>(), b in any::<i64>()) {
      assert_eq!(a + b, b + a);
  }
}
```

---

# 🧱 Phase 2 — Function Extraction

## Goal

Extract annotated functions from full `.pl` files.

---

## Tasks

* Scan file line-by-line
* Detect:

  ```
  # sig:
  ```
* Collect annotation block
* Detect `sub NAME {`
* Extract body via brace matching

---

## Output

```rust
struct ExtractedFunction {
    name: String,
    annotations: Vec<String>,
    body: String,
    start_line: usize,
}
```

---

## CLI Output

```
Found 2 annotated functions:
  - foo
  - bar
```

---

# 🧱 Phase 3 — Annotation Parsing

## Goal

Parse:

```
# sig: (I64, I64) -> I64
# pre: $x > 0
# pos: $result > $x
```

---

## Output

```rust
struct FunctionSpec {
    name: String,
    arg_types: Vec<Type>,
    ret_type: Type,
    pre: Expr,
    post: Expr,
}
```

---

## Add Tests

* Valid signatures
* Invalid formats
* Missing variables

---

# 🧱 Phase 4 — Parsing (pest)

## Tool

Use: pest

---

## Goal

Parse extracted function body into AST.

---

## Output

```rust
struct FunctionAST {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}
```

---

## Tests

* Valid programs parse
* Invalid syntax rejected

---

# 🧱 Phase 5 — Type Checking

## Goal

Attach types from `# sig`

---

## Tasks

* Validate variables
* Ensure type correctness

---

## Tests

* Type mismatch detection
* Undeclared variables

---

# 🧱 Phase 6 — SSA Transformation

## Goal

Convert AST → SSA

---

## Output

```rust
enum SSAStmt {
    Assign(String, SSAExpr),
    If { ... },
    Return(SSAExpr),
}
```

---

## Tests

* No variable reassignment
* Correct versioning

---

# 🧱 Phase 7 — CFG Construction

## Goal

Build control flow graph

---

## Optional Tool

* petgraph

---

## Tests

* Correct branching
* Proper merges

---

# 🧱 Phase 8 — Symbolic Execution Engine

## State

```rust
struct State {
    env: HashMap<String, SymExpr>,
    path_condition: BoolExpr,
}
```

---

## Algorithm

* Worklist-based exploration
* Fork on `if`

---

## Tests

* Path splitting correctness
* Deterministic execution

---

# 🧱 Phase 9 — SMT Encoding

## Tool

Use: Z3
via Rust bindings

---

## Check

```
PC ∧ ¬PostCondition
```

---

## Tests

* Known satisfiable/unsatisfiable cases

---

# 🧱 Phase 10 — Counterexample Output

## Output

```
Function foo failed:
  x = 1
  y = -5
```

---

## Tests

* Model correctly mapped to variables

---

# 🧱 Phase 11 — CLI Integration (Final Form)

## Command

```
tool check file.pl
```

---

## Output

```
✔ foo: verified
✘ bar: counterexample found
```

---

# 🧱 Phase 12 — Logging & Debugging

Use: tracing

---

## Add logs for:

* extraction
* AST
* SSA
* CFG
* symbolic states
* SMT queries

---

# 🧭 Phase 13 — Future Extensions

## Add:

* loops + invariants
* arrays (Z3 arrays)
* function calls
* path pruning
* state merging

---

# ⚖️ Key Design Principles

1. CLI works from day 1
2. Build incrementally
3. Test every phase
4. Keep subset strict
5. Own your semantics

---

# 🚀 Milestones

## Milestone 1

CLI + extraction

## Milestone 2

Annotations + AST parsing

## Milestone 3

SSA + CFG

## Milestone 4

Symbolic execution

## Milestone 5

Z3 integration

## Milestone 6

Full verification pipeline

---

# 🧠 Final Note

This approach ensures:

* Early feedback loop
* Continuous validation
* Reduced integration risk

---

