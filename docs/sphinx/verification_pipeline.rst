Verification Pipeline
=====================

perlchecker processes each annotated function through an eight-stage
pipeline. This page describes what each stage does and how they connect.

Pipeline Overview
-----------------

.. code-block:: text

   ┌─────────────────────────────────────────────────────┐
   │  Perl Source File                                   │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  1. Function Extraction                             │
   │     Scan for # sig: blocks → ExtractedFunction      │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  2. Annotation Parsing                              │
   │     Parse sig, pre, post → FunctionSpec             │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  3. PEG Parsing                                     │
   │     Parse body → AST (loops unrolled here)          │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  4. Type Checking                                   │
   │     Validate types, declarations, expressions       │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  5. SSA / IR Lowering                               │
   │     Version variables → SsaFunction                 │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  6. CFG Construction                                │
   │     Basic blocks + terminators → ControlFlowGraph   │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  7. Symbolic Execution                              │
   │     Explore all paths with symbolic inputs          │
   └──────────────────────┬──────────────────────────────┘
                          ▼
   ┌──────────────────────────────────────────────────────┐
   │  8. SMT Encoding (Z3)                               │
   │     For each path: is ¬postcondition satisfiable?   │
   └──────────────────────┬──────────────────────────────┘
                          ▼
              ┌───────────┴───────────┐
              │                       │
         ✔ Verified            ✘ Counterexample


Stage 1: Function Extraction
-----------------------------

**Module:** ``src/extractor/mod.rs``

Scans the source file for contiguous comment blocks starting with
``# sig:``, followed immediately by ``sub name { ... }``. Extracts:

- Function name
- Annotation lines (the ``#``-prefixed comments)
- Function body text (everything between ``{`` and the matching ``}``)
- Source line number

Non-annotated functions are silently skipped.


Stage 2: Annotation Parsing
-----------------------------

**Module:** ``src/annotations/mod.rs``

Parses the extracted annotation lines into a ``FunctionSpec``:

- ``# sig: (T1, T2) -> R`` → parameter types and return type
- ``# pre: expr`` → precondition expression (defaults to ``true``)
- ``# post: expr`` (or ``# pos:``) → postcondition expression

Validates that variable references in pre/post match the declared parameters
and that the parameter count in the signature matches the function body.


Stage 3: PEG Parsing
----------------------

**Module:** ``src/parser/mod.rs`` + ``src/parser/perl_subset.pest``

Parses the function body using a PEG (Parsing Expression Grammar) via the
`pest <https://pest.rs/>`_ library. Produces a ``FunctionAst`` containing
a list of statements and expressions.

**Loop unrolling happens here.** All loops (``while``, ``for``, ``until``,
``do-while``, ``do-until``) are converted to nested ``if``/``else`` chains
up to ``max_loop_unroll`` depth. For example, with a depth of 3::

   while ($x > 0) { $x--; }

becomes::

   if ($x > 0) {
       $x--;
       if ($x > 0) {
           $x--;
           if ($x > 0) {
               $x--;
               // LoopBoundExceeded marker
           }
       }
   }

**Desugaring** also occurs at this stage:

- ``$x += 1`` → ``$x = $x + 1``
- ``$x++`` → ``$x = $x + 1``
- ``unless (C)`` → ``if (!C)``
- ``until (C)`` → ``while (!C)``
- Statement modifiers → ``if`` blocks
- 2-arg ``substr($s, $off)`` → ``substr($s, $off, length($s) - $off)``


Stage 4: Type Checking
------------------------

**Module:** ``src/ast/mod.rs``

Performs flow-sensitive type checking on the AST:

- Every variable must be declared with ``my`` before use
- Declared but uninitialized variables cannot be read
- Arithmetic operators require ``Int`` operands
- String operators require ``Str`` operands
- Array indices must be ``Int``; hash keys must be ``Str``
- Precondition and postcondition expressions are type-checked too
- Reference aliases are tracked through branches and propagated to the
  postcondition (aliases created in both branches of an ``if``/``else`` are
  preserved after the merge)


Stage 5: SSA / IR Lowering
----------------------------

**Module:** ``src/ir/mod.rs``

Converts the AST to Static Single Assignment (SSA) form:

- Each variable assignment creates a new versioned name (``x__0``, ``x__1``,
  ``x__2``, ...)
- ``if``/``else`` branches produce **merge** (phi) nodes that select the
  correct version based on which branch was taken
- Function calls in expressions are **lifted** to separate assignments
- Array operations become ``store``/``select`` expressions


Stage 6: CFG Construction
---------------------------

**Module:** ``src/ir/mod.rs``

Builds a Control Flow Graph from the SSA form:

- Each basic block contains straight-line assignments
- Blocks end with a **terminator**: ``Branch`` (conditional), ``Goto``,
  ``Return``, ``Die``, or ``LoopBoundExceeded``
- Branch targets reference other block IDs
- Merge points use **block parameters** (phi functions)


Stage 7: Symbolic Execution
-----------------------------

**Module:** ``src/symexec/mod.rs``

Executes the CFG symbolically, treating all function inputs as
unconstrained symbolic variables:

- Maintains a **path condition** — accumulated constraints from branches
- At each ``Branch`` terminator, forks into two states (then/else)
- Only explores **feasible** paths (path condition is satisfiable)
- Tracks a **path budget** (default: 1024 paths) to prevent explosion
- **Array length invariants** are injected: all array length companions are
  constrained to be ≥ 0
- **Extern contracts** are strictly evaluated: if a precondition or
  postcondition expression cannot be evaluated, it is an error (not
  silently assumed true)
- For each path reaching a ``Return``:

  - Checks if the path is *valid* via a well-definedness condition
    (reflexivity check on the result expression, including collections)
  - Encodes the verification query for the SMT solver

**Path pruning:** If the path condition conjoined with a branch condition
is unsatisfiable, that branch is not explored.


Stage 8: SMT Encoding and Solving
------------------------------------

**Module:** ``src/smt/mod.rs``

For each completed path, translates symbolic expressions to Z3 formulas:

- ``Int`` expressions → Z3 ``Int`` sort
- ``Str`` expressions → Z3 ``String`` sort
- ``Bool`` expressions → Z3 ``Bool`` sort
- Arrays → Z3 ``Array`` sort with ``Select``/``Store``
- Hashes → Z3 ``Array<String, Value>`` sort

The verification query for each path is:

.. code-block:: text

   path_condition ∧ precondition ∧ ¬postcondition

- If **UNSAT**: the postcondition holds on this path (verified)
- If **SAT**: the solver model is a counterexample — concrete input values
  that violate the postcondition
- If **UNKNOWN**: the solver timed out or hit a theory limitation

**Safety constraints** are automatically injected:

- Division and modulo: ``divisor ≠ 0``
- String length: all strings bounded to 32 characters
- Substring bounds: ``offset ≥ 0``, ``offset ≤ length(str)``, ``len ≥ 0``
