# =============================================================
# Round 58: Nested loop path stress (unroll combo)
# =============================================================
# Exercises nested for-loops (2 levels) with small bounds where
# inner loop has conditionals and early exit via last. Path
# counts multiply across outer x inner x branches, stressing
# the symbolic execution engine's path expansion capabilities.
# Default unroll limit is 5, so we keep each loop <= 3 iters.

# --- Nested 2x3 with if/else in inner loop ---
# Outer 2 iters, inner 3 iters, each with if/else => 2^6 = 64 paths max
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 5 && $y >= 0 && $y <= 5
# post: $result >= 0 && $result <= 30
sub nested_branch_accum {
    my ($x, $y) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < 2; $i++) {
        my $j;
        for ($j = 0; $j < 3; $j++) {
            if ($j < $y) {
                $sum = $sum + $x;
            } else {
                $sum = $sum + 1;
            }
        }
    }
    return $sum;
}

# --- Nested 2x2 with last in inner loop ---
# Inner loop exits early when accumulator exceeds threshold,
# creating divergent path counts per outer iteration.
# sig: (I64, I64) -> I64
# pre: $val >= 1 && $val <= 3 && $cap >= 1 && $cap <= 10
# post: $result >= 0 && $result <= 12
sub nested_early_exit {
    my ($val, $cap) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 2; $i++) {
        my $j;
        for ($j = 0; $j < 3; $j++) {
            $acc = $acc + $val;
            last if ($acc >= $cap);
        }
    }
    return $acc;
}

# --- Nested 3x2 with conditional accumulation ---
# Outer 3 iters, inner 2 iters. Condition depends on both
# loop counters, creating different branch patterns per iter.
# sig: (I64) -> I64
# pre: $base >= 0 && $base <= 4
# post: $result >= 0 && $result <= 30
sub nested_counter_dep {
    my ($base) = @_;
    my $total = 0;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        my $j;
        for ($j = 0; $j < 2; $j++) {
            if ($i + $j > 2) {
                $total = $total + $base;
            } else {
                $total = $total + 1;
            }
        }
    }
    return $total;
}

# --- Nested 2x2 with last and if/else combo ---
# Both last and branching in inner loop. Outer loop also
# has a condition check, creating cross-level path interaction.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 5 && $b >= 1 && $b <= 5
# post: $result >= 0 && $result <= 20
sub nested_combo_stress {
    my ($a, $b) = @_;
    my $r = 0;
    my $i;
    for ($i = 0; $i < 2; $i++) {
        my $j;
        for ($j = 0; $j < 2; $j++) {
            if ($a > $j + $i) {
                $r = $r + $b;
            } else {
                $r = $r + 1;
            }
            last if ($r >= 15);
        }
    }
    return $r;
}
