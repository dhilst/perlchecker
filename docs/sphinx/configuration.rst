Configuration
=============

CLI Syntax
----------

.. code-block:: console

   $ perlchecker check FILE.pl [OPTIONS]

Options
-------

.. list-table::
   :header-rows: 1
   :widths: 30 12 58

   * - Option
     - Default
     - Description
   * - ``--max_loop_unroll N``
     - ``5``
     - Maximum number of times each loop is unrolled. Higher values let you
       verify loops that iterate more times but increase path count and
       verification time.
   * - ``--max_paths N``
     - ``1024``
     - Maximum number of distinct symbolic execution paths per function.
       Verification aborts with an error if this limit is exceeded.
   * - ``--solver_timeout_ms N``
     - ``5000``
     - Z3 solver timeout in milliseconds per query. If the solver does not
       return within this time, the result is ``Unknown``.

Environment Variables
---------------------

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Variable
     - Description
   * - ``RUST_LOG``
     - Controls tracing output. Values: ``error``, ``warn``, ``info``,
       ``debug``, ``trace``. Example: ``RUST_LOG=debug perlchecker check f.pl``

Internal Limits
---------------

These are not configurable via the CLI:

.. list-table::
   :header-rows: 1
   :widths: 30 12 58

   * - Limit
     - Value
     - Description
   * - ``MAX_STR_LEN``
     - ``32``
     - Maximum string length in the SMT model. Strings longer than 32
       characters cannot be reasoned about.

Tuning Guidance
---------------

**When to increase ``--max_loop_unroll``:**

If you see "exceeded loop unroll bound on a feasible path", your loop
iterates more than the default 5 times. Increase the bound and tighten
the precondition so the loop terminates within the new bound.

.. code-block:: console

   $ perlchecker check file.pl --max_loop_unroll 10

**When to increase ``--max_paths``:**

If you see "exceeded maximum number of symbolic paths", the function has
too many branches. Each ``if``/``else`` doubles the path count. For
example, 11 independent ``if``/``else`` blocks produce 2048 paths.

.. code-block:: console

   $ perlchecker check file.pl --max_paths 4096

**When to increase ``--solver_timeout_ms``:**

If you see "solver returned unknown", the Z3 solver timed out on a complex
query. This can happen with deep string manipulation or large array
operations.

.. code-block:: console

   $ perlchecker check file.pl --solver_timeout_ms 30000
