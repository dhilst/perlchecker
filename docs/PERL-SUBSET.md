# Verifiable Perl Subset Specification

## Goal

Define a Perl-like subset that is:

* Large enough to express real programs
* Small enough to be symbolically executed and verified
* Compatible with SMT-based reasoning (Z3 with BV(64) + strings + arrays)

---

# 1. Types

## Primitive Types

```text
I64
Str
```

* No implicit coercion between `I64` and `Str`
* All variables are statically typed via `# sig:`

## Composite Types

### Arrays

```text
Array<I64>
Array<Str>
```

* Indexed by integers
* Access via: `$arr[i]`

### Hashes

```text
Hash<Str, I64>
Hash<Str, Str>
```

* Key-value maps with string keys
* Access via: `$h{"key"}`

## Reference Types

```text
\$scalar    -> RefI64, RefStr
\@array     -> RefArrayI64, RefArrayStr
\%hash      -> RefHashI64, RefHashStr
```

* Dereference via: `$$ref`
* Arrow access: `$ref->[i]`, `$ref->{"key"}`

## Restrictions

* No nested structures beyond one level of reference
* No autovivification

---

# 2. Functions

## Syntax

```perl
# sig: (I64, Str) -> Str
# pre: ...
# post: ...
sub foo {
  my ($x, $y) = @_;
  ...
  return $z;
}
```

## Rules

* Pure functions only
* No global variables
* Fixed arity
* Explicit `return`

## Function Calls

```perl
$z = foo($x, $y);
```

### Constraints

* Only annotated functions
* Same file
* No recursion
* No dynamic dispatch

---

# 3. Variables and Assignment

```perl
my $x = expr;
$x = expr;
$x++;
$x--;
```

## Compound Assignment

```perl
$x += expr;  $x -= expr;  $x *= expr;
$x /= expr;  $x %= expr;  $x **= expr;
$x .= expr;
$x <<= expr;  $x >>= expr;
$x &= expr;  $x |= expr;  $x ^= expr;
```

### Rules

* All variables declared with `my`
* No aliasing
* SSA transformation applied internally

---

# 4. Expressions

## Arithmetic Operators

```perl
+ - * / % **
```

## String Operators

```perl
.     # concatenation
x     # repetition
```

## Numeric Comparison

```perl
< <= > >= == !=
<=>   # spaceship (three-way comparison)
```

## String Comparison

```perl
eq ne lt gt le ge
cmp   # three-way string comparison
```

## Logical Operators

```perl
&& || !
and or not    # low-precedence forms
```

## Bitwise Operators

```perl
& | ^ ~
<< >>
```

## Ternary

```perl
$cond ? $a : $b
```

## Division Semantics

* `x / y` is defined only if `y != 0`
* Division by zero paths are treated as invalid (discarded)
* Use `int($x / $y)` for truncating integer division

## Modulo Semantics

* `x % y` is defined only if `y != 0`
* Result follows SMT semantics

---

# 5. Builtin Functions

## Numeric

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs($x)` | `I64 -> I64` | Absolute value |
| `min($a, $b)` | `(I64, I64) -> I64` | Minimum |
| `max($a, $b)` | `(I64, I64) -> I64` | Maximum |
| `int($x)` | `I64 -> I64` | Integer truncation |

## String

| Function | Signature | Description |
|----------|-----------|-------------|
| `length($s)` | `Str -> I64` | String length |
| `substr($s, $i)` | `(Str, I64) -> Str` | Substring from position |
| `substr($s, $i, $n)` | `(Str, I64, I64) -> Str` | Substring with length |
| `index($s, $t)` | `(Str, Str) -> I64` | Find substring position |
| `index($s, $t, $p)` | `(Str, Str, I64) -> I64` | Find substring from position |
| `ord($s)` | `Str -> I64` | Character to ordinal |
| `chr($n)` | `I64 -> Str` | Ordinal to character |
| `chomp($s)` | `Str -> Str` | Remove trailing newline |
| `reverse($s)` | `Str -> Str` | Reverse string |
| `contains($s, $t)` | `(Str, Str) -> I64` | Substring containment (0/1) |
| `starts_with($s, $t)` | `(Str, Str) -> I64` | Prefix check (0/1) |
| `ends_with($s, $t)` | `(Str, Str) -> I64` | Suffix check (0/1) |
| `replace($s, $old, $new)` | `(Str, Str, Str) -> Str` | String replacement |
| `char_at($s, $i)` | `(Str, I64) -> Str` | Character at index |

## Array

| Function | Signature | Description |
|----------|-----------|-------------|
| `scalar(@arr)` | `Array<T> -> I64` | Array length |
| `push(@arr, $v)` | statement | Append element |
| `pop(@arr)` | `Array<T> -> T` | Remove and return last element |

## Hash

| Function | Signature | Description |
|----------|-----------|-------------|
| `exists($h{"key"})` | `Hash -> I64` | Check if key exists (0/1) |
| `defined($expr)` | `any -> I64` | Check if value is defined (0/1) |

---

# 6. Control Flow

## If / Elsif / Else

```perl
if ($cond) { ... }
elsif ($cond2) { ... }
else { ... }
```

## Unless

```perl
unless ($cond) { ... }
```

## While / Until

```perl
while ($cond) { ... }
until ($cond) { ... }
```

## Do-While / Do-Until

```perl
do { ... } while ($cond);
do { ... } until ($cond);
```

## For (C-style)

```perl
for (my $i = 0; $i < $n; $i++) { ... }
```

## Foreach

```perl
foreach my $x (@arr) { ... }
```

## Loop Control

```perl
last;                    # break
last if ($cond);         # conditional break
last unless ($cond);
next;                    # continue
next if ($cond);         # conditional continue
next unless ($cond);
```

## Statement Modifiers

```perl
return $x if ($cond);
return $x unless ($cond);
$x = $y if ($cond);
$x = $y unless ($cond);
die "msg" if ($cond);
die "msg" unless ($cond);
```

## Semantics

* Loops are bounded-unrolled (configurable via `--max_loop_unroll`)
* Each branch creates a symbolic path

---

# 7. Annotations

## Signature (required)

```perl
# sig: (T1, T2, ...) -> T
```

## Precondition (optional)

```perl
# pre: condition
```

## Postcondition (required)

```perl
# post: condition
```

* `$result` refers to return value

## External Function Contracts

```perl
# extern: func_name (T1, T2) -> T pre: ... post: ...
```

## Ghost Variables

```perl
# ghost: $var = expr
```

Specification-only variables for capturing intermediate state.

## Assertions

```perl
# assert: condition
```

## Loop Invariants

```perl
# inv: condition
```

---

# 8. Regex (Limited)

```perl
$x =~ /pattern/
$x !~ /pattern/
```

Desugared to string operations at the parser level.

---

# 9. Error Handling

```perl
die "message";
croak "message";
confess "message";
warn "message";
```

## Output

```perl
print expr;
say expr;
```

---

# 10. Memory Model

## Arrays

```text
Array(I64 -> T)
```

## Hashes

```text
Array(Str -> T)
```

## Operations

* Read: `select(arr, i)`
* Write: `store(arr, i, v)`
* Missing keys return unconstrained symbolic values

---

# 11. Execution Semantics

## Symbolic Execution

* Inputs become symbolic variables
* Branches fork states
* Path conditions tracked per path

## Function Calls

* Inlined at call site
* SSA renaming applied

## Loop Handling

* Bounded unrolling only (configurable limit)

## Arithmetic Safety

* Division/modulo by zero creates an invalid path (discarded)

---

# 12. Forbidden Features

* `eval(...)` — dynamic code execution
* `$_`, `@_` (beyond parameter binding) — implicit variables
* `wantarray` — context sensitivity
* Unbounded recursion
* Global variables
* Dynamic dispatch
* Implicit type coercions
* Nested data structures (beyond single reference level)
* File/network IO (beyond `print`/`say`/`warn`)

---

# 13. Safety Constraints

## Required Limits

```text
MAX_LOOP_UNROLL   (default: 9)
MAX_PATHS         (default: 1024)
SOLVER_TIMEOUT    (default: 5000ms)
```

## Error Conditions

* Unsupported syntax
* Type mismatch
* Recursion detected
* Division/modulo by zero path

---

# Summary

This subset supports:

* 64-bit integer (I64) and string computation
* Bitwise operations
* Structured control flow (if/elsif/else, unless, loops, foreach)
* Function composition (with inlining)
* Arrays and hashes (with references)
* Rich string operations
* Contract-based verification (pre/post/assert/invariant)

While avoiding:

* Dynamic Perl semantics
* Aliasing
* Unbounded constructs
* Undecidable features
