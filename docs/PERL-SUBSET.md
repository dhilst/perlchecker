# Verifiable Perl Subset Specification

Tested against: **Perl v5.42.0** (x86_64-linux-thread-multi)

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

### I64 Semantics

* **Range**: Signed 64-bit, from `-(2^63)` = `-9223372036854775808` to `(2^63)-1` = `9223372036854775807`
* **Overflow**: When arithmetic exceeds 64-bit range, Perl silently promotes
  the result to floating-point (NV). Under `use integer`, overflow wraps as
  two's complement.
* **No negative zero**: `-0 == 0` always holds for integers.
* **Literal formats**: Decimal, hex (`0xFF`), binary (`0b1111`), octal (`0377`) are all supported.
* **Undef in numeric context**: An uninitialized variable numifies to `0` (with a warning).

Tested by: `t/semantics/01_primitive_types.t`

### Str Semantics

* **Encoding**: Strings are sequences of Unicode code points. `length()` counts code points, not bytes.
* **Empty string**: `""` has length 0, equals itself.
* **Escape sequences**: `\n`, `\t`, `\0`, `\\` are each a single character.
* **Null bytes**: Valid inside strings (`length("\0") == 1`).
* **Value equality**: Strings are compared by value, not by reference identity.
* **Concat identity**: `"" . $s eq $s` and `$s . "" eq $s`.
* **Undef in string context**: An uninitialized variable stringifies to `""` (with a warning).

Tested by: `t/semantics/01_primitive_types.t`

### Coercion Semantics

* **I64 → Str** (via `.` concatenation): `"" . 42` yields `"42"`,
  `"" . -5` yields `"-5"`, `"" . 0` yields `"0"`. String length equals
  the number of decimal digits (plus 1 for minus sign if negative).
  Extremes: `I64_MAX` yields `"9223372036854775807"` (19 chars),
  `I64_MIN` yields `"-9223372036854775808"` (20 chars).
* **Str → I64** (via `int()` or arithmetic context): `int("42")` yields
  `42`. Leading whitespace is stripped. Leading digits are extracted,
  trailing non-digits ignored: `int("3abc")` yields `3`. Non-numeric
  strings yield `0`: `int("abc")` yields `0`. Float strings truncated
  toward zero: `int("3.14")` yields `3`, `int("-3.14")` yields `-3`.
* **Boolean context (truthiness)**: Exactly four scalar values are false:
  `0`, `""`, `"0"`, and `undef`. Everything else is true — including
  `"00"`, `" "`, `"0E0"`, and all negative integers.
* **`!` return values**: `!false` returns the string `"1"`. `!true` returns the empty string `""`.

Tested by: `t/semantics/02_coercion.t`

### Array Semantics

* **Indexing**: Zero-based. `$a[0]` is first element, `$a[-1]` is last
  element, `$a[-N]` where N = length is first element.
* **Out-of-bounds read**: Returns `undef` (numifies to `0`, stringifies to `""`).
* **Write beyond end**: Extends the array; intermediate positions are filled with `undef`.
* **Boolean context**: Non-empty arrays are true, empty arrays are false.
* **Last index**: `$#arr` is `scalar(@arr) - 1`; `-1` for empty arrays.

Tested by: `t/semantics/16_arrays.t`

### Hash Semantics

* **Missing key read**: Returns `undef` (numifies to `0`, stringifies to `""`).
* **Write creates key**: `$h{"new"} = val` creates the key if absent, overwrites if present.
* **Read does not autovivify**: Reading `$h{"key"}` or `defined($h{"key"})` does NOT create the key.
* **exists vs defined**: A key can exist but have `undef` value. `exists`
  checks key presence; `defined` checks value definedness. Absent keys
  are neither existing nor defined.
* **Special keys**: Empty string `""` and `"0"` are valid hash keys.
* **Value truthiness**: A value of `0`, `""`, or `undef` is false but the key still exists.

Tested by: `t/semantics/17_hashes.t`

### Reference Semantics

* **Creation**: `\$x` creates a scalar reference, `\@a` an array reference, `\%h` a hash reference.
* **Dereference**: `$$ref` for scalar refs, `$ref->[i]` for array refs, `$ref->{"key"}` for hash refs.
* **Write-through**: Modifying through a reference modifies the original: `$$ref = 99` changes `$x`.
* **Aliasing**: Multiple references to the same variable see the same
  value. A write through one ref is visible through all others.
* **`ref()` type**: Returns `"SCALAR"`, `"ARRAY"`, or `"HASH"` depending on the referent type.

Tested by: `t/semantics/18_references.t`

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

### Semantics

* **Lexical scoping**: `my $x` creates a new variable scoped to the
  enclosing block. Inner blocks can shadow outer variables.
* **Compound assignment equivalence**: `$x += $y` is semantically
  `$x = $x + $y` (same for all 12 compound assignment operators:
  `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `.=`, `<<=`, `>>=`, `&=`,
  `|=`, `^=`).
* **Post-increment/decrement**: `$x++` returns the old value, then
  increments. `$x--` returns the old value, then decrements.
* **Pre-increment**: `++$x` increments first, then returns the new value.

Tested by: `t/semantics/14_variables.t`

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

### Arithmetic Semantics

* **Commutativity**: `+` and `*` are commutative. `-` and `/` are not.
* **Identity elements**: `x + 0 == x`, `x * 1 == x`, `x - 0 == x`, `x / 1 == x`.
* **Inverse**: `x + (-x) == 0`, `x - x == 0`.
* **Overflow**: When I64 arithmetic exceeds 64-bit range, Perl silently
  promotes to float. Under `use integer`, values wrap as two's complement.
* **Unary negation**: `-(-x) == x`, `-(0) == 0`.
* **I64 boundary edges**: `I64_MIN / -1` overflows to float.
  `I64_MIN * -1` wraps to `I64_MIN` under `use integer`.
  `I64_MIN % -1 == 0`. `2 ** 63` overflows to float.

### Division Semantics

* `x / y` is defined only if `y != 0`. Division by zero `die`s with "Illegal division by zero".
* Perl's `/` returns a float for integer operands: `7 / 2 == 3.5`.
* `int($x / $y)` truncates toward zero (not floor): `int(-7/2) == -3`, `int(7/-2) == -3`.

### Modulo Semantics

* `x % y` is defined only if `y != 0`. Modulo by zero `die`s with "Illegal modulus zero".
* Perl uses **floor-modulo**: the result's sign follows the divisor.
  - `7 % 3 == 1` (positive/positive)
  - `-7 % 3 == 2` (negative/positive → positive result)
  - `7 % -3 == -2` (positive/negative → negative result)
  - `-7 % -3 == -1` (negative/negative → negative result)
* **Division identity caveat**: `x == int(x/y)*y + (x%y)` holds only
  for non-negative `x`, because `int()` truncates toward zero while
  `%` uses floor-modulo (different rounding modes).

### Exponentiation Semantics

* `0 ** 0 == 1`, `x ** 0 == 1` for all `x`, `x ** 1 == x`.
* `x ** 2 == x * x`.
* Negative exponents produce fractional results (float): `2 ** -1 == 0.5`.
* `(-1) ** even == 1`, `(-1) ** odd == -1`.

Tested by: `t/semantics/03_arithmetic.t`

### String Operator Semantics

* **Concatenation (`.`)**: Not commutative. Empty string is the identity:
  `"" . $s eq $s`. Coerces I64 operands to strings: `"x" . 42` yields
  `"x42"`. Length is additive.
* **Repetition (`x`)**: `$s x N` repeats `$s` N times. `$s x 0` yields
  `""`. `$s x -1` yields `""` (negative count produces empty).
  `$s x 1 eq $s`. Length equals `length($s) * N`.

Tested by: `t/semantics/04_string_ops.t`

### Numeric Comparison Semantics

* **Return values**: True comparisons return the string `"1"`. False comparisons return `""` (empty string), NOT `0`.
* **`==`**: Reflexive (`$x == $x`), symmetric.
* **`<`**: Irreflexive, transitive.
* **Trichotomy**: For any two integers, exactly one of `<`, `==`, `>` holds.
* **`<=>`** (spaceship): Returns exactly `-1`, `0`, or `1`.
  Antisymmetric: `($a <=> $b) == -($b <=> $a)`.
  `($x <=> $y) == -1` iff `$x < $y`.

Tested by: `t/semantics/05_numeric_comparison.t`

### String Comparison Semantics

* **`eq`/`ne`**: String equality, case-sensitive. Empty strings equal only themselves.
* **`lt`/`gt`/`le`/`ge`**: Lexicographic by Unicode code point. Uppercase
  before lowercase (ASCII order). Empty string is less than any non-empty
  string. Prefix is less than longer string: `"abc" lt "abcd"`.
* **`cmp`**: Returns exactly `-1`, `0`, or `1`. Antisymmetric.
* **Return values**: `eq` true returns `"1"`, false returns `""`.
* **Numeric strings**: Lexicographic comparison differs from numeric:
  `"10" lt "9"` is true (lexicographic), but `10 < 9` is false
  (numeric). This distinction matters when comparing stringified numbers.

Tested by: `t/semantics/06_string_comparison.t`

### Logical Operator Semantics

* **`&&`**: Returns the **deciding value**, not a boolean. If left is
  false, returns left operand. If left is true, returns right operand.
  Examples: `(5 && 3) == 3`, `(0 && 3) == 0`, `("" && 3) eq ""`.
* **`||`**: Returns the **deciding value**. If left is true, returns left
  operand. If left is false, returns right operand. Examples:
  `(5 || 3) == 5`, `(0 || 3) == 3`.
* **`!`**: Returns `"1"` for false input, `""` for true input. Double negation normalizes to `0` or `1`.
* **Short-circuit**: `&&` does not evaluate the right operand if the left
  is false. `||` does not evaluate the right if the left is true.
* **`and`/`or`/`not`**: Same semantics as `&&`/`||`/`!` but lower precedence.
* **Precedence matters**: `$x = 0 || 2` assigns `2` to `$x` (`||` binds
  tighter than `=`). But `$x = 0 or 2` assigns `0` to `$x` (`or` binds
  looser than `=`, so assignment happens first).
* **Chained `||`**: Returns the first truthy value: `0 || "" || 5 || 6` returns `5`.
* **De Morgan's laws**: `!($x && $y) == (!$x || !$y)` and `!($x || $y) == (!$x && !$y)`.

Tested by: `t/semantics/07_logical_ops.t`

### Bitwise Operator Semantics

* **`&` (AND)**: Idempotent (`x & x == x`), zero annihilator (`x & 0 == 0`), commutative.
* **`|` (OR)**: Idempotent (`x | x == x`), identity (`x | 0 == x`), commutative.
* **`^` (XOR)**: Self-XOR is 0 (`x ^ x == 0`), identity (`x ^ 0 == x`),
  commutative. Double-XOR cancels (`x ^ y ^ y == x`).
* **`~` (NOT)**: Double complement is identity (`~~x == x`). Without
  `use integer`, `~0` yields max unsigned (`2^64 - 1`). Under
  `use integer`, `~0 == -1` (signed interpretation).
* **`<<` (left shift)**: `x << 0 == x`, `x << 1 == x * 2`. Negative
  shift amounts reverse direction (`1 << -1` same as `1 >> 1`).
* **`>>` (right shift)**: `x >> 0 == x`, `x >> 1 == int(x/2)` for
  non-negative. Under `use integer`, right shift is arithmetic (sign bit
  preserved): `-1 >> 1 == -1`. Negative shift amounts reverse direction.
* **Negative operands**: Under `use integer`, bitwise ops work on signed two's complement. `-1 & 0xFF == 0xFF`.
* **Shift beyond word size**: `1 << 64 == 0` and `255 >> 64 == 0` (Perl returns 0 for shifts >= 64).

Tested by: `t/semantics/08_bitwise_ops.t`

### Ternary Semantics

* `$cond ? $a : $b` evaluates to `$a` if `$cond` is true, `$b` if false.
* **Only one branch evaluated**: Side effects in the non-taken branch do not occur.
* **Truthiness**: Condition follows standard boolean context rules.
* **Nesting**: Right-associative. `0 ? 1 : 0 ? 2 : 3` evaluates to `3`.

Tested by: `t/semantics/09_ternary.t`

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

### Numeric Builtin Semantics

* **`abs($x)`**: `abs(0) == 0`. `abs(positive) == positive`.
  `abs(negative) == -negative`. Always non-negative. Edge case:
  `abs(I64_MIN)` overflows (Perl promotes to float).
* **`min($a, $b)`**: Returns the smaller value. `min(x, x) == x`.
  `min(a, b) <= max(a, b)`. Result is always one of the two inputs.
* **`max($a, $b)`**: Returns the larger value. Same properties as `min` with opposite direction.
* **`int($x)`**: Identity on integers. On floats, truncates toward zero
  (not floor): `int(3.7) == 3`, `int(-3.7) == -3`, `int(0.9) == 0`,
  `int(-0.9) == 0`.

Tested by: `t/semantics/10_builtins_numeric.t`

### String Builtin Semantics

* **`length($s)`**: Always `>= 0`. `length("") == 0`. Counts Unicode code points, not bytes.
* **`substr($s, $i)`**: Returns substring from position `$i` to end.
  `substr($s, 0)` returns whole string. Negative `$i` wraps from end:
  `substr("hello", -2) eq "lo"`.
* **`substr($s, $i, $n)`**: Returns `$n` characters starting at `$i`.
  Length 0 returns empty string. Length beyond end clamps to available.
  Negative length means "leave off N from end":
  `substr("hello", 1, -1) eq "ell"`.
* **`index($s, $t)`**: Returns position of first occurrence, or `-1` if
  not found. Empty needle returns `0`. `index("", "a")` returns `-1`.
* **`index($s, $t, $p)`**: Starts search from position `$p`. Empty
  needle at position returns that position (clamped to string length).
* **`ord($s)`**: Returns code point of first character. Multi-char
  strings: takes first char only. `ord("") == 0` (undef numified).
  `ord("A") == 65`.
* **`chr($n)`**: Returns single character with given code point.
  `chr(65) eq "A"`. `chr(0) eq "\0"`. Roundtrip:
  `chr(ord($c)) eq $c` for single-char strings.
* **`chomp($s)`**: Removes trailing `\n` from `$s` **in place**. Returns
  number of characters removed (0 or 1). Only removes one trailing
  newline: `"hello\n\n"` becomes `"hello\n"`. Does NOT remove `\r`:
  `"hello\r\n"` becomes `"hello\r"`. Standalone `\r` is not removed.
* **`reverse($s)`**: Reverses character order. `reverse("") eq ""`.
  `reverse("a") eq "a"`. Double reverse is identity. Preserves length.
* **`contains($s, $t)`**: Returns 1 if `$t` is a substring of `$s`, 0 otherwise. Every string contains `""`.
* **`starts_with($s, $t)`**: Returns 1 if `$s` starts with `$t`, 0 otherwise.
* **`ends_with($s, $t)`**: Returns 1 if `$s` ends with `$t`, 0 otherwise.
* **`replace($s, $old, $new)`**: Replaces occurrences of `$old` with
  `$new`. Returns original string if `$old` not found.
* **`char_at($s, $i)`**: Returns single character at position `$i`.
  Negative index wraps from end: `char_at("hello", -1) eq "o"`.

Tested by: `t/semantics/11_builtins_string.t`

### Array Builtin Semantics

* **`scalar(@arr)`**: Returns array length. `0` for empty arrays. Tracks correctly after `push`/`pop`.
* **`push(@arr, $v)`**: Appends `$v` to end. Increases length by 1.
  Preserves existing elements. Returns new array length.
* **`pop(@arr)`**: Returns and removes last element. Decreases length by 1. Returns `undef` for empty arrays.

Tested by: `t/semantics/12_builtins_array.t`

### Hash Builtin Semantics

* **`exists($h{"key"})`**: Returns `1` if key present, `""` if absent.
  Key with value `0`, `""`, or `undef` still exists. Reading a key does
  NOT cause it to exist.
* **`defined($expr)`**: Returns `1` if value is defined, `""` if undef.
  `defined(0)` is true. `defined("")` is true. `defined(undef)` is
  false. Missing hash values are undef.

Tested by: `t/semantics/13_builtins_hash.t`

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

### Control Flow Semantics

* **if/elsif/else**: First true branch is taken, remaining branches
  skipped. All conditions are checked in order until one is true.
* **unless**: Semantically equivalent to `if (!$cond)`.
* **while**: Condition checked before each iteration. False condition initially means zero iterations.
* **until**: Condition checked before each iteration. True condition
  initially means zero iterations. Equivalent to `while (!$cond)`.
* **do-while**: Body executes at least once, then condition is checked. Even with a false condition, the body runs once.
* **do-until**: Body executes at least once, then condition is checked. Even with a true condition, the body runs once.
* **for (C-style)**: Init runs once, condition checked before each
  iteration, increment after each iteration body. Condition false
  initially means zero iterations.
* **foreach**: Iterates over array elements in order. Empty array means zero iterations. Element order is preserved.
* **last**: Exits the innermost enclosing loop immediately. `last if ($cond)` is conditional break.
* **next**: Skips to the next iteration of the innermost loop. `next if ($cond)` is conditional continue.
* **Statement modifiers**: `$x = $y if ($cond)` is equivalent to
  `if ($cond) { $x = $y }`. Same for `unless`. Also:
  `die "msg" if ($cond)`, `die "msg" unless ($cond)`.
* **Nested loops**: `last` and `next` affect only the innermost enclosing loop.
* **`last unless`/`next unless`**: Conditional variants that break/skip when the condition is false.
* **Loop variable scoping**: `for (my $i = ...)` scopes `$i` to the loop; it does not leak to the enclosing scope.

Tested by: `t/semantics/15_control_flow.t`

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

### Regex Semantics

* **`=~`**: Returns `1` (true) if the pattern matches, `""` (empty string, false) if not.
* **`!~`**: Exact negation of `=~`. Returns `1` if no match, `""` if match.
* **Anchors**: `^` matches start of string, `$` matches end (including before trailing `\n`).
* **Character classes**: `\d` matches digits, `\w` matches word chars,
  `\s` matches whitespace, `[a-z]` matches ranges.
* **Quantifiers**: `+` (one or more), `*` (zero or more), `?` (zero or one), `{n}` (exactly n).

Tested by: `t/semantics/20_regex.t`

---

# 9. Error Handling

```perl
die "message";
croak "message";
confess "message";
warn "message";
```

### Error Handling Semantics

* **`die`**: Terminates execution with a message. Without trailing `\n`,
  Perl appends file and line info. With trailing `\n`, message is used
  verbatim. Code after `die` is not executed.
* **`warn`**: Prints message to STDERR but does NOT terminate execution. Code after `warn` continues.
* **`croak`** (Carp): Like `die` but reports the error from the caller's perspective.
* **`confess`** (Carp): Like `die` but includes a full stack trace.
* **`eval { ... }`**: Catches `die` exceptions. Returns `undef` if an
  exception was thrown, otherwise returns the last expression. The
  exception message is stored in `$@`.

Tested by: `t/semantics/19_error_handling.t`

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
