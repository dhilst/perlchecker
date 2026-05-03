Limitations
===========

Bounded Verification
--------------------

perlchecker performs **bounded verification**, not full inductive
verification. Loops are unrolled to a finite depth (default: 5). A
"verified" result means the postcondition holds for all inputs *that cause
the loop to terminate within the unroll bound*.

To verify a loop, you must provide a precondition that bounds the number of
iterations. For example, ``# pre: $x >= 0 && $x <= 5`` ensures a
countdown loop terminates in at most 5 steps.

No Recursion
------------

Direct and indirect recursion is detected and rejected. Functions can only
call other functions — not themselves, and not through a cycle.

String Length Bound
-------------------

Strings are bounded to 32 characters in the SMT model. The verifier cannot
reason about strings longer than this. This limit is not configurable.

Path Explosion
--------------

Each ``if``/``else`` in a function doubles the number of symbolic execution
paths. A function with *n* independent conditionals has up to 2\ :sup:`n`
paths. The default path budget is 1024 (about 10 independent branches).

Use ``--max_paths`` to raise the budget, but be aware that verification time
grows with the number of paths.

Single-File Scope
-----------------

All functions called by a verified function must be defined in the same
source file. There is no support for modules, imports, or cross-file
verification.

Soundness Guarantees
--------------------

perlchecker enforces the following soundness invariants:

- **Extern contracts are strict.** If an extern function's precondition or
  postcondition cannot be evaluated (e.g., references an undefined variable),
  verification fails with an error rather than silently assuming the contract
  is satisfied.

- **Array lengths are non-negative.** Symbolic array length companions
  (``scalar(@arr)``) are constrained to be ≥ 0, matching Perl semantics.

- **Reference aliases propagate.** Aliases created in the function body are
  visible in the postcondition type-check, and aliases created in both
  branches of an ``if``/``else`` are preserved after the merge.

- **Collection well-definedness.** Array and hash return values undergo the
  same reflexivity check as scalar results, ensuring the SMT expression is
  evaluable.

- **Counterexample accuracy.** When the solver cannot extract a concrete
  value for a variable in a counterexample, it is reported as
  ``<unconstrained>`` rather than an arbitrary default like 0 or "".


Unsupported Perl Features
--------------------------

The following Perl features are **not** supported:

- Regular expressions (``=~``, ``m/pattern/``, ``s/from/to/``)
- References beyond scalar refs (``\@arr``, ``\%hash``, nested refs)
- ``eval`` and runtime code generation
- Implicit variables (``$_``, ``@ARGV``)
- ``@_`` outside of the initial parameter binding ``my (...) = @_;``
- Nested data structures (arrays of arrays, hashes of hashes)
- Object-oriented features (``bless``, method calls, ``->``\ )
- Modules and ``use`` / ``require`` statements
- File I/O and system calls
- ``wantarray`` and context sensitivity
- Autovivification
- Global variables
- ``local`` and ``our`` declarations
- Regular subroutine prototypes
- ``sort``, ``map``, ``grep``
- ``foreach`` loops (only C-style ``for`` is supported)
- Anonymous subroutines and closures
- ``sprintf`` / ``printf``

Solver Timeouts
---------------

Complex queries — especially those involving deep string manipulation,
nested array operations, or many bitwise operations — may cause the Z3
solver to time out. When this happens, the result is ``Unknown`` rather
than ``Verified`` or ``Counterexample``.

Increase ``--solver_timeout_ms`` if you suspect the solver needs more time
rather than the query being undecidable.
