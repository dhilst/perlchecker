# Expansion Plan: Rounds 5–9

Instantiation of `EXPANSION-META-PLAN.md` for the next 5 rounds.
Each round follows: PERL DEV → CHECKER → JUDGE → IMPLEMENTER+QA → ORCHESTRATOR commit.

---

## Round 5: Compound Assignment Operators

**Feature:** `+=`, `-=`, `*=`, `/=`, `%=`, `.=`

**Rationale:** Ubiquitous in real Perl (accumulators, string building, loop counters). Pure syntactic sugar — desugars to `$x = $x OP expr` at parse time.

**Layers touched:** Parser only (grammar + `parse_assign` + `parse_for_assign`).

**Changes:**
- `perl_subset.pest`: Add `assign_op` rule with ordered alternatives (`"+=" | "-=" | "*=" | "/=" | "%=" | ".=" | "="`)
- `parser/mod.rs`: `parse_assign` detects compound op, wraps RHS in `Expr::Binary { left: Var(name), op, right: rhs }`
- `for_scalar_assign` grammar: use `assign_op` instead of `"="`
- `parse_for_assign`: same desugaring logic

**Example (`examples/round5_dev.pl`):**
```perl
# sig: (Int, Int) -> Int
# pre: $n >= 0 && $n <= 5 && $step > 0 && $step <= 10
# post: $result == $n * $step
sub mul_by_add {
    my ($n, $step) = @_;
    my $acc = 0;
    for (my $i = 0; $i < $n; $i += 1) {
        $acc += $step;
    }
    return $acc;
}
```

**QA:** `cargo test` + `cargo run --quiet -- check examples/round5_dev.pl` → verified.

---

## Round 6: `unless` Statement

**Feature:** `unless (COND) { ... } else { ... }` — negated if.

**Rationale:** Idiomatic Perl for guard clauses. Desugars to `if (!(COND)) { ... } else { ... }` at parse time.

**Layers touched:** Parser only (grammar + statement dispatch).

**Changes:**
- `perl_subset.pest`: Add `unless_stmt = { "unless" ~ "(" ~ expr ~ ")" ~ block ~ else_clause? }` to `stmt`
- `parser/mod.rs`: `parse_unless` wraps condition in `Expr::Unary { op: Not, expr: condition }`, reuses existing `Stmt::If`

**Example (`examples/round6_dev.pl`):**
```perl
# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result > 0
sub ensure_positive {
    my ($x) = @_;
    my $r = $x;
    unless ($x > 0) {
        $r = 1;
    }
    return $r;
}
```

**QA:** `cargo test` + `cargo run --quiet -- check examples/round6_dev.pl` → verified.

---

## Round 7: `min()` / `max()` Builtins

**Feature:** `min($a, $b)` and `max($a, $b)` as built-in functions.

**Rationale:** Extremely common in clamping, bounds checking, range logic. Desugars to ternary: `min(a,b)` → `(a <= b) ? a : b`.

**Layers touched:** Parser (grammar + builtin call recognition) + AST (new `Builtin::Min`, `Builtin::Max`) + IR (desugar to `SsaExpr::Ite`) + symexec (handle in builtin dispatch).

**Changes:**
- `perl_subset.pest`: Add `min_call = { "min" ~ "(" ~ expr ~ "," ~ expr ~ ")" }` and `max_call` to `atom`
- `ast/mod.rs`: Add `Builtin::Min`, `Builtin::Max`
- `ir/mod.rs`: Lower `Builtin::Min(a,b)` → `Ite(a <= b, a, b)` (already have Ite support)
- `symexec/mod.rs`: Handle via existing Ite path after IR desugaring

**Example (`examples/round7_dev.pl`):**
```perl
# sig: (Int, Int, Int) -> Int
# pre: $lo <= $hi
# post: $result >= $lo && $result <= $hi
sub clamp_minmax {
    my ($x, $lo, $hi) = @_;
    my $r = max(min($x, $hi), $lo);
    return $r;
}
```

**QA:** `cargo test` + `cargo run --quiet -- check examples/round7_dev.pl` → verified.

---

## Round 8: `unless` used as Expression Guard (early return pattern)

**Feature:** Multiple `return` statements in different branches (already syntactically supported — this round validates and tests the pattern with `unless`+`return`).

Actually — **Revised Feature:** `last` statement for loop early exit.

**Feature:** `last;` inside while/for loops to break out early.

**Rationale:** Common pattern for search loops ("find first match, break"). In the unrolled-loop model, `last` can be modeled as a conditional that skips remaining iterations.

**Layers touched:** Parser (grammar) + parser desugaring (transform `last` into a flag-and-skip pattern during unrolling).

**Complexity concern:** This is more complex. The unroll strategy must track a `$__broke` flag. When `last` is hit, set flag; each subsequent unrolled iteration wraps in `if (!$__broke) { ... }`.

**Changes:**
- `perl_subset.pest`: Add `last_stmt = { "last" ~ ";" }` to block-level statements
- `ast/mod.rs`: Add `Stmt::Last`
- `parser/mod.rs`: In `unroll_while`, wrap body iterations with a break-flag check. When `Stmt::Last` appears in body, replace with `Assign { __broke = 1 }` and guard subsequent iterations.

**Example (`examples/round8_dev.pl`):**
```perl
# sig: (Int, Int) -> Int
# pre: $n >= 1 && $n <= 5 && $target >= 0 && $target <= 10
# post: $result >= 0 && $result < $n
sub find_index {
    my ($n, $target) = @_;
    my $found = 0;
    for (my $i = 0; $i < $n; $i += 1) {
        if ($i == $target) {
            $found = $i;
            last;
        }
    }
    return $found;
}
```

**QA:** `cargo test` + `cargo run --quiet -- check examples/round8_dev.pl` → verified.

**Risk:** Medium-high. If desugaring is too complex for haiku, mark as NOT IMPLEMENTED and move on.

---

## Round 9: Negative Integer Literals

**Feature:** Negative integer literals as atoms (e.g., `my $x = -1;` without needing `0 - 1`).

**Rationale:** Currently `-1` parses as `UnaryOp::Neg(Int(1))` which works but `-1` as a literal in annotations/preconditions may behave differently. This round validates that negative constants work end-to-end and adds explicit test coverage. If they already work, this is a "validation round" with no code changes — just examples.

**Layers touched:** Possibly none (validate existing behavior) or grammar (allow `-` in `int` rule for literal form).

**Changes:**
- Verify `my $x = -1;` parses and verifies correctly
- If it does: just add example file, no code changes
- If it doesn't: extend `int` rule to `@{ "-"? ~ ASCII_DIGIT+ }` or handle in AST

**Example (`examples/round9_dev.pl`):**
```perl
# sig: (Int) -> Int
# pre: $x >= -50 && $x <= 50
# post: $result >= 0
sub distance_from_origin {
    my ($x) = @_;
    my $neg = -1;
    my $r = ($x >= 0) ? $x : $x * $neg;
    return $r;
}
```

**QA:** `cargo test` + `cargo run --quiet -- check examples/round9_dev.pl` → verified.

---

## Execution Order

| Round | Feature | Complexity | Risk |
|-------|---------|-----------|------|
| 5 | Compound assignment (+=, -=, etc.) | Low (parser only) | Low |
| 6 | `unless` statement | Low (parser only) | Low |
| 7 | min()/max() builtins | Medium (parser+AST+IR) | Low |
| 8 | `last` loop break | High (unroll redesign) | Medium-High |
| 9 | Negative integer literals | Low (validation/grammar) | Low |

Rounds 5, 6, 9 are near-certain to succeed. Round 7 is moderate. Round 8 is the stretch goal.
