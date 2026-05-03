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
Int
Str
Bool (derived from expressions)
```

### Notes

* No implicit coercion between `Int` and `Str`
* All variables are statically typed via `# sig`

---

## Composite Types

### Arrays

```text
Array<Int>
Array<Str>
```

* Indexed by integers
* Access via: `$arr[i]`

---

### Hashes

```text
Hash<Str, Int>
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
# sig: (Int, Str) -> Str
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
Array(Int → T)
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
