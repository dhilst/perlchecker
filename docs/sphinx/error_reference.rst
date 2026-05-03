Error Reference
===============

Errors are organized by the pipeline stage that produces them.


Extraction Errors
-----------------

These occur when scanning the source file for annotated functions.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``annotation block at line N must be followed immediately by sub``
     - There is a blank line or non-comment line between the annotation
       comments and the ``sub`` keyword. Remove the blank line.
   * - ``invalid sub declaration at line N``
     - The ``sub`` line does not match the expected format
       ``sub name {``. Check for typos or missing braces.
   * - ``function NAME has unmatched braces``
     - The closing ``}`` for the function body is missing or mismatched.


Annotation Errors
-----------------

These occur when parsing the ``# sig:``, ``# pre:``, ``# post:`` lines.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``missing # sig: annotation``
     - Every annotated function must have a ``# sig:`` line declaring its
       type signature.
   * - ``missing # post: annotation``
     - Every annotated function must have a ``# post:`` (or ``# pos:``)
       line declaring its postcondition.
   * - ``duplicate directive: sig``
     - The annotation block has more than one ``# sig:`` line. Only one
       is allowed.
   * - ``duplicate directive: post``
     - The annotation block has more than one ``# post:`` line.
   * - ``unsupported type: Foo``
     - The type ``Foo`` is not recognized. Supported types: ``Int``, ``Str``,
       ``Array<Int>``, ``Array<Str>``, ``Hash<Str, Int>``, ``Hash<Str, Str>``.
   * - ``parameter count mismatch: sig declares N but body has M``
     - The number of types in the ``# sig:`` line does not match the number
       of parameters in ``my (...) = @_;``.
   * - ``references unknown variable $x``
     - The precondition or postcondition uses a variable that is not a
       function parameter and is not ``$result``.
   * - ``invalid expression in annotation``
     - The expression in ``# pre:`` or ``# post:`` has a syntax error.


Parse Errors
------------

These occur when parsing the function body.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``invalid syntax at line:column``
     - The function body contains Perl syntax that is not part of the
       supported subset. See :doc:`language_reference` for what is
       supported.


Type Check Errors
-----------------

These occur during static type analysis of the function body.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``undeclared variable $x``
     - The variable ``$x`` is used without a prior ``my $x`` declaration.
   * - ``uninitialized variable $x``
     - The variable ``$x`` was declared with ``my $x;`` but read before
       being assigned a value. Initialize it: ``my $x = 0;``.
   * - ``type mismatch``
     - An operator received the wrong type. For example, using ``+`` on
       strings or ``eq`` on integers.
   * - ``array index must be Int``
     - An array access ``$arr[$i]`` used a non-integer index.
   * - ``unsafe substring start``
     - The ``substr`` start position could be negative or out of bounds.
       Add a precondition to constrain it.


Symbolic Execution Errors
--------------------------

These occur during path exploration and verification.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``recursive call graph detected``
     - The function calls itself directly or through a chain of calls.
       Recursion is not supported.
   * - ``exceeded maximum number of symbolic paths (1024)``
     - The function has too many branches. Reduce branching or increase
       ``--max_paths``. See :doc:`configuration`.
   * - ``exceeded loop unroll bound on a feasible path``
     - A loop iterates more than ``max_loop_unroll`` times on a feasible
       input. Tighten the precondition to bound iterations or increase
       ``--max_loop_unroll``.
   * - ``can reach a die statement on a feasible path``
     - A ``die`` (or ``croak``/``confess``) statement is reachable given
       the precondition. Either the precondition is too weak or the die
       is intentional and should be guarded more tightly.
   * - ``no valid execution paths``
     - Every execution path hits an arithmetic error (like division by
       zero). The function has no valid behavior to verify.
   * - ``extern contract evaluation failed``
     - An extern function's precondition or postcondition expression could
       not be evaluated (e.g., it references an undefined variable or
       has a type error). Check that the extern annotations are correct
       and that all referenced variables are in scope.
   * - ``assertion failed on a feasible path``
     - An ``assert`` statement evaluates to false on at least one feasible
       execution path. Either the assertion is wrong or the precondition
       needs tightening.


SMT Errors
----------

These occur during Z3 solver interaction.

.. list-table::
   :header-rows: 1
   :widths: 45 55

   * - Error
     - Cause and Fix
   * - ``solver returned unknown``
     - The Z3 solver could not determine satisfiability within the timeout.
       Try increasing ``--solver_timeout_ms`` or simplifying the function.
