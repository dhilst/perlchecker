# =============================================================
# Round 52: Loop + branch combo path stress
# =============================================================
# Exercises the symbolic execution engine with for-loops
# containing if/else branches and early exits via last/next.
# Each iteration multiplies paths by the branches within,
# creating iteration x branch path products across unrolling.

# --- For-loop with if/else in each iteration: 2^4 = 16 paths ---
# Each of 4 iterations has an if/else, doubling the path count
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $n >= 1 && $n <= 4
# post: $result >= 0 && $result <= 40
sub loop_branch_multiply {
    my ($x, $n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        if ($x > $i * 2) {
            $sum = $sum + $x;
        } else {
            $sum = $sum + $i;
        }
    }
    return $sum;
}

# --- Loop with next if: skipping iterations prunes paths ---
# 4 iterations, next skips even indices, only odd indices accumulate
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 4
# post: $result >= 0 && $result <= 4
sub loop_next_skip {
    my ($n) = @_;
    my $count = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i % 2 == 0);
        $count = $count + 1;
    }
    return $count;
}

# --- Loop with last if: early exit cuts remaining paths ---
# Accumulates until threshold hit, then breaks
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 5 && $limit >= 1 && $limit <= 10
# post: $result >= 0 && $result <= 25
sub loop_last_threshold {
    my ($x, $limit) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        $sum = $sum + $x;
        last if ($sum >= $limit);
    }
    return $sum;
}

# --- Loop + ternary + conditional assignment: combined stress ---
# Each iteration picks via ternary and adds conditionally
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 5 && $b >= 0 && $b <= 5
# post: $result >= 0 && $result <= 25
sub loop_ternary_combo {
    my ($a, $b) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        my $val = ($i < $a) ? $b : 1;
        if ($val > 2) {
            $acc = $acc + $val;
        } else {
            $acc = $acc + 1;
        }
    }
    return $acc;
}

# --- Nested conditions inside loop with last/next combo ---
# Each iteration: if/elsif/else with next and last creating complex CFG
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 3
# post: $result >= 0 && $result <= 35
sub loop_nested_control {
    my ($x, $y) = @_;
    my $r = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        if ($i < $y) {
            $r = $r + $x;
            next;
        } elsif ($i == $y) {
            $r = $r + 1;
        } else {
            last if ($r > 20);
            $r = $r + 2;
        }
    }
    return $r;
}
