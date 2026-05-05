# =============================================================
# Round 88: Complex for-loop stride patterns
# =============================================================
# Functions using for-loops with non-unit strides (i += 2, i += 3,
# i *= 2) to exercise the symbolic execution engine's ability to
# reason about non-trivial iteration sequences. All loops complete
# within the default unroll limit of 5 iterations.

# --- Function 1: Stride-2 loop summing even indices ---
# Iterates with stride 2 over range [0,8), visiting 0,2,4,6.
# Accumulates the index values. 4 iterations within unroll limit.
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 10
# post: $result == 12
sub stride2_sum_even {
    my ($x) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 8; $i += 2) {
        $acc += $i;
    }
    return $acc;
}

# --- Function 2: Stride-3 loop with accumulator and conditional ---
# Steps by 3 over [0,12): visits 0,3,6,9. Each iteration adds
# either $i or 1 depending on whether $i > $threshold.
# Creates 2^4 = 16 paths.
# sig: (I64) -> I64
# pre: $threshold >= 0 && $threshold <= 12
# post: $result >= 4 && $result <= 19
sub stride3_conditional_accum {
    my ($threshold) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 12; $i += 3) {
        if ($i > $threshold) {
            $acc += $i;
        } else {
            $acc += 1;
        }
    }
    return $acc;
}

# --- Function 3: Multiplicative stride (i *= 2) for log iteration ---
# Starts at 1, multiplies by 2 each step: visits 1,2,4,8.
# Stops when i >= 16. 4 iterations. Accumulates a count.
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 20
# post: $result == 4
sub mult_stride_count {
    my ($n) = @_;
    my $count = 0;
    my $i;
    for ($i = 1; $i < 16; $i *= 2) {
        $count += 1;
    }
    return $count;
}

# --- Function 4: Stride-4 loop with branch on parity ---
# Steps by 4 over [0,16): visits 0,4,8,12. Each iteration
# checks if $val > $i and adds different amounts.
# sig: (I64) -> I64
# pre: $val >= 0 && $val <= 10
# post: $result >= 4 && $result <= 40
sub stride4_parity_branch {
    my ($val) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 16; $i += 4) {
        if ($val > $i) {
            $acc += $val;
        } else {
            $acc += 1;
        }
    }
    return $acc;
}

# --- Function 5: Mixed strides with nested condition ---
# Stride-2 loop with nested conditionals that depend on both
# the loop variable and the parameter. Creates rich path space.
# Loop visits 1,3,5: 3 iterations with 3 branches each = 27 paths.
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 3 && $result <= 60
sub mixed_stride_nested {
    my ($x, $y) = @_;
    my $acc = 0;
    my $i;
    for ($i = 1; $i < 6; $i += 2) {
        if ($x > $i) {
            if ($y > $i) {
                $acc += $x + $y;
            } else {
                $acc += $x;
            }
        } else {
            $acc += 1;
        }
    }
    return $acc;
}
