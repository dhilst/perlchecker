Annotation Reference
====================

Annotations are Perl comments placed in a contiguous block immediately above
the ``sub`` keyword. Three directives are supported: ``sig``, ``pre``, and
``post`` (or ``pos``).

``# sig:`` — Type Signature
----------------------------

**Required.** Declares parameter types and return type.

.. code-block:: perl

   # sig: (Type1, Type2, ...) -> ReturnType

Supported types:

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Type
     - Description
   * - ``I64``
     - 64-bit signed integer
   * - ``Str``
     - UTF-8 string
   * - ``Array<I64>``
     - Array of integers
   * - ``Array<Str>``
     - Array of strings
   * - ``Hash<Str, I64>``
     - String-keyed hash with integer values
   * - ``Hash<Str, Str>``
     - String-keyed hash with string values

Examples:

.. code-block:: perl

   # sig: (I64) -> I64
   # sig: (I64, I64) -> I64
   # sig: (Str, Str) -> Str
   # sig: (Array<I64>, I64) -> I64
   # sig: (Hash<Str, I64>, Str) -> I64

The number of types in the signature must match the number of parameters in the
``my (...) = @_;`` binding.

``# pre:`` — Precondition
--------------------------

**Optional.** Constrains valid inputs. If omitted, defaults to ``true``
(all inputs are valid).

.. code-block:: perl

   # pre: boolean_expression

The expression can reference function parameters (with ``$`` prefix) and use
any operators or built-in functions the language supports.

Examples:

.. code-block:: perl

   # pre: $x >= 0
   # pre: $x > 0 && $y > 0
   # pre: $x >= 0 && $x <= 5
   # pre: length($s) >= 6 && length($s) <= 12
   # pre: scalar(@arr) > 0
   # pre: $y != 0

During verification, paths that violate the precondition are pruned — the tool
only checks paths where the precondition holds.

``# post:`` — Postcondition
-----------------------------

**Required.** States what must be true about the return value. The alias
``# pos:`` is also accepted.

.. code-block:: perl

   # post: boolean_expression

Use ``$result`` to refer to the function's return value. The expression can
also reference function parameters.

Examples:

.. code-block:: perl

   # post: $result == $x
   # post: $result >= 0
   # post: $result == $x + $y
   # post: length($result) == length($x) + length($y)
   # post: $result eq $x . $y
   # post: ($x <= $y && $result == $y - $x) || ($x > $y && $result == $x - $y)

``# extern:`` — External Function Contracts
---------------------------------------------

Declares the type signature and contract of a function defined outside the
verified file. Place these as standalone comment lines anywhere in the file
(they do not need to be attached to a ``sub``).

.. code-block:: perl

   # extern: NAME (Type1, Type2, ...) -> ReturnType pre: EXPR post: EXPR

The ``pre:`` and ``post:`` clauses are optional (default to ``true``).
Parameters are referenced positionally as ``$a``, ``$b``, ``$c``, etc. in
pre/postconditions. Use ``$result`` in the postcondition to refer to the
return value.

Examples:

.. code-block:: perl

   # extern: abs_val (I64) -> I64 post: $result >= 0
   # extern: clamp (I64, I64, I64) -> I64 pre: $b <= $c post: $result >= $b && $result <= $c
   # extern: lookup (Hash<Str, I64>, Str) -> I64

When a verified function calls an extern, perlchecker:

1. **Checks** the precondition is satisfied at the call site (given the
   caller's path condition)
2. **Assumes** the postcondition holds for the fresh symbolic return value
3. Continues verification with those assumptions

If the precondition or postcondition expression cannot be evaluated (e.g.,
references an undefined variable), verification fails with an error rather
than silently assuming the contract is satisfied.


``# ghost:`` — Ghost Variables
-------------------------------

Ghost variables are specification-only variables that exist for verification
purposes but produce no runtime code. They let you capture intermediate
values and write assertions about relationships between program states.

Place ghost annotations as comments inside the function body:

.. code-block:: perl

   # ghost: $varname = EXPR

The variable name must start with ``$``. The expression can reference any
in-scope variable (parameters or previously declared locals). Ghost variables
are visible to subsequent ``# assert:`` annotations and to other ghost
assignments.

Examples:

.. code-block:: perl

   # sig: (I64, I64) -> I64
   # post: $result == $x + $y
   sub add_with_ghost {
       my ($x, $y) = @_;
       # ghost: $expected = $x + $y
       my $sum = $x + $y;
       # assert: $sum == $expected
       return $sum;
   }

   # sig: (I64) -> I64
   # pre: $n >= 0
   # post: $result >= $n
   sub double_ghost {
       my ($n) = @_;
       # ghost: $original = $n
       my $result = $n * 2;
       # assert: $result >= $original
       return $result;
   }

Ghost variables are lowered to regular SSA assignments in the IR and
participate in symbolic execution like any other variable, but they have no
effect on the program's runtime behavior.


Common Patterns
---------------

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Pattern
     - Meaning
   * - ``# post: $result == $x``
     - Returns the input unchanged (identity)
   * - ``# post: $result >= 0``
     - Result is non-negative
   * - ``# pre: $y != 0``
     - Guards against division by zero
   * - ``# pre: $x >= 0 && $x <= 5``
     - Bounds input for loop unrolling
   * - ``# post: $result == $arr[$i]``
     - Returns the element at index ``$i``
   * - ``# post: length($result) == length($x)``
     - Preserves string length

Validation Rules
----------------

The tool rejects annotations that violate these rules:

**Function annotations (sig/pre/post):**

- Missing ``# sig:`` → error
- Missing ``# post:`` (or ``# pos:``) → error
- Duplicate ``# sig:``, ``# pre:``, or ``# post:`` → error
- Unknown directive (not sig/pre/post/pos) → error
- Parameter count in signature does not match function parameters → error
- Precondition or postcondition references an unknown variable → error
- Postcondition does not reference ``$result`` → allowed but likely a mistake
- Blank line between annotation block and ``sub`` → extraction error

**Extern declarations:**

- Missing ``->`` in the type signature → error
- Unsupported type in parameter list or return type → error
- Invalid expression in ``pre:`` or ``post:`` clause → error
- Precondition/postcondition that fails to evaluate at call site → error

**Ghost annotations:**

- Must start with ``$variable_name`` → error if missing
- Must contain ``=`` separating variable from expression → error
- Expression must be syntactically valid → error if it cannot be parsed
- Ghost variable is type-checked like any other assignment
