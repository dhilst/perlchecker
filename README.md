# perlchecker

Symbolic verification for a restricted Perl subset.

perlchecker formally verifies annotated Perl functions using symbolic execution and SMT solving (Z3). Write preconditions and postconditions as comments above your functions, and perlchecker either proves they hold for all valid inputs or returns a concrete counterexample.

```perl
# sig: (I64, I64) -> I64
# pre: $y != 0
# post: $result == $x / $y
sub safe_division {
    my ($x, $y) = @_;
    return $x / $y;
}
```

```console
$ perlchecker check division.pl
✔ safe_division: verified
```

## Installation

Requires Rust and a Z3 installation (the `z3` crate links against libz3).

```sh
cargo build --release
```

The binary is at `target/release/perlchecker`.

## Usage

```sh
perlchecker check <file.pl>
```

Options:

| Flag | Default | Description |
|------|---------|-------------|
| `--max_loop_unroll` | 9 | Maximum loop unrolling depth |
| `--max_paths` | 1024 | Maximum symbolic execution paths |
| `--solver_timeout_ms` | 5000 | Z3 solver timeout per query |

## Annotations

### `# sig:` — Type Signature (required)

```perl
# sig: (I64, Str) -> I64
```

Supported types: `I64`, `Str`, `Array<I64>`, `Array<Str>`, `Hash<Str, I64>`, `Hash<Str, Str>`.

### `# pre:` — Precondition (optional)

```perl
# pre: $x >= 0 && $x <= 10
```

### `# post:` — Postcondition (required)

```perl
# post: $result >= 0
```

Use `$result` to refer to the return value.

### `# extern:` — External Function Contracts

Declare contracts for functions defined outside the file:

```perl
# extern: abs_val (I64) -> I64 post: $result >= 0
# extern: clamp (I64, I64, I64) -> I64 pre: $b <= $c post: $result >= $b && $result <= $c
```

### `# ghost:` — Ghost Variables

Specification-only variables for capturing intermediate state:

```perl
sub double {
    my ($n) = @_;
    # ghost: $original = $n
    my $result = $n * 2;
    # assert: $result >= $original
    return $result;
}
```

## Verification Pipeline

```
Perl Source → Function Extraction → Annotation Parsing → PEG Parsing
  → Type Checking → SSA/IR Lowering → CFG Construction
    → Symbolic Execution → SMT Encoding (Z3) → Verified / Counterexample
```

## Documentation

Full documentation: https://dhilst.github.io/perlchecker/

## License

Apache License 2.0 — see [LICENSE](LICENSE).
