Examples
========

This page walks through several example files to illustrate how
perlchecker works in practice.


Basic Verification
------------------

**File:** ``examples/00_identity_with_pre.pl``

.. code-block:: perl

   # sig: (I64) -> I64
   # pre: $x >= 0
   # post: $result == $x
   sub identity_with_pre {
       my ($x) = @_;
       return $x;
   }

.. code-block:: console

   $ perlchecker check examples/00_identity_with_pre.pl
   ✔ identity_with_pre: verified

The function returns its input unchanged. The precondition constrains
``$x`` to non-negative values, and the postcondition states the result
equals ``$x``. There is one path, and the solver confirms the postcondition
holds for all valid inputs.


Finding a Counterexample
------------------------

**File:** ``examples/04_counterexample.pl``

.. code-block:: perl

   # sig: (I64) -> I64
   # post: $result > $x
   sub counterexample {
       my ($x) = @_;
       if ($x >= 0) {
           return $x;
       } else {
           return $x + 1;
       }
   }

.. code-block:: console

   $ perlchecker check examples/04_counterexample.pl
   ✘ counterexample: counterexample found
   Function counterexample failed:
     x = 0

The postcondition claims the result is strictly greater than the input. On
the ``$x >= 0`` branch, the function returns ``$x`` itself — which is not
greater than ``$x``. The solver finds ``x = 0`` as a concrete witness.


Safe Division
-------------

**File:** ``examples/09_safe_division.pl``

.. code-block:: perl

   # sig: (I64, I64) -> I64
   # pre: $y != 0
   # post: $result == $x / $y
   sub safe_division {
       my ($x, $y) = @_;
       return $x / $y;
   }

.. code-block:: console

   $ perlchecker check examples/09_safe_division.pl
   ✔ safe_division: verified

The precondition ``$y != 0`` guards against division by zero. The verifier
prunes any path where ``$y == 0``, so the division is always safe. Without
the precondition, verification would fail because the zero-divisor path
would be discarded as invalid, leaving no valid paths to check.


String Operations
-----------------

**File:** ``examples/12_string_concat_length.pl``

.. code-block:: perl

   # sig: (Str, Str) -> Str
   # post: length($result) == length($x) + length($y)
   sub string_concat_length {
       my ($x, $y) = @_;
       return $x . $y;
   }

.. code-block:: console

   $ perlchecker check examples/12_string_concat_length.pl
   ✔ string_concat_length: verified

The postcondition states that concatenating two strings produces a result
whose length equals the sum of the input lengths. The Z3 string theory
confirms this holds for all strings (up to the 32-character bound).


Arrays
------

**File:** ``examples/21_array_read_verified.pl``

.. code-block:: perl

   # sig: (Array<I64>, I64) -> I64
   # post: $result == $arr[$i]
   sub array_read_verified {
       my ($arr, $i) = @_;
       return $arr[$i];
   }

.. code-block:: console

   $ perlchecker check examples/21_array_read_verified.pl
   ✔ array_read_verified: verified

Arrays are modeled using Z3's Array theory. Reading an element and returning
it trivially satisfies the postcondition ``$result == $arr[$i]``.


Function Calls
--------------

**File:** ``examples/27_function_call_verified.pl``

.. code-block:: perl

   # sig: (I64) -> I64
   # post: $result == $x + 1
   sub inc {
       my ($x) = @_;
       return $x + 1;
   }

   # sig: (I64) -> I64
   # post: $result == $x + 1
   sub function_call_verified {
       my ($x) = @_;
       return inc($x);
   }

.. code-block:: console

   $ perlchecker check examples/27_function_call_verified.pl
   ✔ inc: verified
   ✔ function_call_verified: verified

Both functions are verified. ``function_call_verified`` calls ``inc``,
which is inlined during verification. The verifier proves that ``inc($x)``
returns ``$x + 1``, satisfying the caller's postcondition.


Loops
-----

**File:** ``examples/30_while_verified.pl``

.. code-block:: perl

   # sig: (I64) -> I64
   # pre: $x >= 0 && $x <= 5
   # post: $result == 0
   sub while_verified {
       my ($x) = @_;
       while ($x > 0) {
           $x = $x - 1;
       }
       return $x;
   }

.. code-block:: console

   $ perlchecker check examples/30_while_verified.pl
   ✔ while_verified: verified

The precondition bounds ``$x`` to ``[0, 5]``, ensuring the loop terminates
within the default unroll depth of 5. The loop counts down to 0, and the
postcondition ``$result == 0`` is verified across all paths.

.. note::

   Without the precondition bounding ``$x``, this function would fail with
   a "loop unroll bound exceeded" error because the loop could iterate
   arbitrarily many times.
