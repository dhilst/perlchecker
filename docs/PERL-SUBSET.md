# Verifiable Perl Subset Specification

## 🎯 Goal

Define a **Perl-like subset** that is:

* Large enough to express real programs
* Small enough to be symbolically executed and verified
* Compatible with SMT-based reasoning

---

# 🧱 1. Types

## Primitive Types

```text
I64
Str
Bool (derived from expressions)
```

### Notes

* No implicit coercion between `I64` and `Str`
* All variables are statically typed via `# sig`

---

## Composite Types

### Arrays

```text
Array<I64>
Array<Str>
```

* Indexed by integers
* Access via: `$arr[i]`

---

### Hashes

```text
Hash<Str, I64>
Hash<Str, Str>
```

* Key-value maps
* Access via: `$h{"key"}`

---

## Restrictions

* No nested structures
* No references
* No autovivification

---

# 🧱 2. Functions

## Syntax

```perl
# sig: (I64, Str) -> Str
# pre: ...
# pos: ...
sub foo {
  my ($x, $y) = @_;
  ...
  return $z;
}
```

---

## Rules

* Pure functions only
* No global variables
* Fixed arity
* Explicit `return`

---

## Function Calls

```perl
$z = foo($x, $y);
```

### Constraints

* Only annotated functions
* Same file (initially)
* No recursion
* No dynamic dispatch

---

# 🧱 3. Variables and Assignment

```perl
my $x = expr;
$x = expr;
```

### Rules

* All variables declared with `my`
* No aliasing
* SSA transformation applied later

---

# 🧱 4. Expressions

---

## Numeric

```perl
+ - * / %
< <= > >= == !=
```

---

## ⚠️ Division Semantics

### Perl behavior (reference)

* Division by zero → runtime error

---

### Subset rule

```text
x / y is defined only if y ≠ 0
```

### Encoding requirement

* Add constraint:

```text
y ≠ 0
```

* If solver finds `y = 0`:
  → treat as invalid execution path (discard)

---

## ⚠️ Modulo Semantics

### Perl behavior

* Result has same sign as divisor

---

### Subset definition (must fix semantics)

Choose one:

#### Option A (recommended — SMT-friendly)

```text
x % y = remainder(x, y)
```

Where:

* matches SMT (Euclidean or implementation-defined)

#### Option B (Perl-accurate)

```text
sign(x % y) = sign(y)
```

---

### Required constraint

```text
y ≠ 0
```

---

## Boolean

```perl
&& || !
```

---

## String

```perl
$a . $b
length($x)
$x eq $y
$x ne $y
substr($x, i, n)
index($x, $y)
```

---

## Array Length via scalar(@arr)

```perl
scalar(@arr)
```

**Type signature:** `Array<T> → I64`

**Semantics:**

Returns the number of elements in an array. The scalar builtin is specifically designed for use with arrays in scalar context.

**Grammar:**

```text
scalar_call = { "scalar" ~ "(" ~ "@" ~ ident ~ ")" }
```

**Note on naming:** The `@` prefix is required in scalar() calls; it denotes array context. Variables used elsewhere in the program use the `$` prefix.

**Symbolic execution:**

```
scalar(@arr) → IntExpr::Var("arr__len")
```

The symbolic execution engine creates a companion length variable (`{array_name}__len`) for each array parameter. This variable is free/unconstrained unless bounded in a `# pre:` annotation.

**Example usage:**

```perl
# sig: (Array<I64>) -> I64
# pre: scalar(@arr) > 0
# post: $result == scalar(@arr)
sub get_array_length {
    my ($arr) = @_;
    return scalar(@arr);
}
```

**SMT encoding:**

```
scalar(@arr) encodes to (I64::new_const "arr__len")
```

The length variable is unconstrained in the SMT solver unless explicitly bounded by preconditions.

---

## Array

```perl
$arr[i]
$arr[i] = v
```

---

## Hash

```perl
$h{"key"}
$h{"key"} = v
```

---

## Restrictions

* No implicit conversions
* No mixed-type arithmetic

---

# 🧱 5. Control Flow

---

## If / Elsif / Else

```perl
if ($cond) {
  ...
} elsif ($cond2) {
  ...
} else {
  ...
}
```

---

## Semantics

* `elsif` → nested `if`
* Each branch creates a symbolic path

---

# 🧱 6. Loops (Bounded)

---

## While

```perl
while ($cond) {
  ...
}
```

---

## For

```perl
for (init; cond; step) {
  ...
}
```

---

## Constraints

```text
MAX_LOOP_UNROLL = N
```

---

## Semantics

* Loops unrolled into nested conditionals
* Execution stops after bound

---

# 🧱 7. Annotations

---

## Signature

```perl
# sig: (T1, T2, ...) -> T
```

---

## Preconditions

```perl
# pre: condition
```

---

## Postconditions

```perl
# pos: condition
```

* `$result` refers to return value

---

## Loop Invariants (future)

```perl
# inv: condition
```

---

# 🧱 8. Memory Model

---

## Arrays

```text
Array(I64 → T)
```

---

## Hashes

```text
Array(Str → T)
```

---

## Operations

* Read:

```text
select(arr, i)
```

* Write:

```text
store(arr, i, v)
```

---

## Missing Keys

* Return unconstrained symbolic value

---

# 🧱 9. Execution Semantics

---

## Symbolic Execution

* Inputs → symbolic variables
* Branches → fork states
* Path conditions tracked

---

## Function Calls

* Inlined
* SSA renaming applied

---

## Loop Handling

* Bounded unrolling only

---

## Arithmetic Safety

* Division/modulo by zero → invalid path
* Such paths must be discarded

---

# 🧱 10. Forbidden Features

---

## Dynamic Features

```perl
eval(...)
```

---

## References

```perl
\$x
\@arr
```

---

## Regex

```perl
$x =~ /.../
```

---

## Implicit Variables

```perl
$_
@_
```

---

## IO

```perl
print
<>
```

---

## Context Sensitivity

```perl
wantarray
```

---

# 🧱 11. Safety Constraints

---

## Required Limits

```text
MAX_LOOP_UNROLL
MAX_PATHS
SOLVER_TIMEOUT
```

---

## Error Conditions

* Unsupported syntax
* Type mismatch
* Recursion detected
* Division/modulo by zero path

---

# 🧭 Summary

This subset supports:

* Numeric (including `/` and `%`) and string computation
* Structured control flow
* Function composition
* Arrays and hashes

While avoiding:

* dynamic Perl semantics
* aliasing
* undecidable constructs

---

# 🧠 Final Principle

This is not full Perl.

It is:

> A statically analyzable, constraint-oriented language with Perl syntax
