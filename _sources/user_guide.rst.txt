User Guide
==========

File Structure
--------------

A perlchecker input file contains one or more Perl functions. Each function
that should be verified must have an annotation block — a contiguous group of
``#``-prefixed comment lines — placed *immediately* above the ``sub`` keyword.
Functions without annotations are ignored.

.. code-block:: perl

   # sig: (I64) -> I64
   # pre: $x >= 0
   # post: $result >= 0
   sub my_function {
       my ($x) = @_;
       return $x;
   }

.. warning::

   There must be **no blank lines** between the annotation block and the
   ``sub`` line. A blank line causes an extraction error.

Function Rules
--------------

Every annotated function must follow these rules:

1. **Parameter binding first.** The first statement must be
   ``my ($param1, $param2, ...) = @_;``.

2. **Explicit return.** Every execution path must end with a ``return``
   statement.

3. **No recursion.** Direct or indirect recursion is rejected.

4. **Same-file calls only.** Functions can call other annotated functions
   in the same file. The callee must appear before the caller.

5. **Pure functions.** No global variables, no I/O side effects (``print``,
   ``say``, and ``warn`` are accepted but treated as no-ops for verification).

Running the Tool
----------------

.. code-block:: console

   $ perlchecker check FILE.pl [OPTIONS]

Options:

.. list-table::
   :header-rows: 1
   :widths: 30 10 60

   * - Flag
     - Default
     - Description
   * - ``--max_loop_unroll N``
     - 5
     - Maximum loop unroll depth
   * - ``--max_paths N``
     - 1024
     - Maximum symbolic execution paths per function
   * - ``--solver_timeout_ms N``
     - 5000
     - Z3 solver timeout in milliseconds

See :doc:`configuration` for tuning guidance.

Understanding Output
--------------------

Verified
^^^^^^^^

.. code-block:: console

   ✔ function_name: verified

The postcondition holds for all inputs satisfying the precondition, across
all execution paths (up to the loop unroll bound).

Counterexample Found
^^^^^^^^^^^^^^^^^^^^

.. code-block:: console

   ✘ function_name: counterexample found
   Function function_name failed:
     x = 42
     y = -1

The solver found concrete input values that satisfy the precondition but
violate the postcondition. These values are a *witness* showing the function
is incorrect.

No Annotated Functions
^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: console

   Found 0 annotated functions

The file contains no functions with annotation blocks.

Errors
^^^^^^

Parse errors, type errors, and verification errors are printed to stderr.
See :doc:`error_reference` for a complete list.

Multi-Function Files
--------------------

A file can contain multiple annotated functions. They are verified in source
order. When function ``B`` calls function ``A``, the verifier inlines ``A``
at the call site — it does not re-verify ``A`` separately for each caller.

.. code-block:: perl

   # sig: (I64) -> I64
   # post: $result == $x + 1
   sub inc {
       my ($x) = @_;
       return $x + 1;
   }

   # sig: (I64) -> I64
   # post: $result == $x + 1
   sub call_inc {
       my ($x) = @_;
       return inc($x);
   }

.. code-block:: console

   $ perlchecker check multi.pl
   ✔ inc: verified
   ✔ call_inc: verified

Debugging with Tracing
-----------------------

Set ``RUST_LOG`` to see internal pipeline details:

.. code-block:: console

   $ RUST_LOG=debug perlchecker check file.pl

Levels: ``error``, ``warn``, ``info``, ``debug``, ``trace``.
