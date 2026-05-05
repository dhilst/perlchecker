# =============================================================
# Round 90: 16+ distinct paths stress test
# =============================================================
# Functions with many sequential binary decisions creating large
# path spaces. Each path computes a different result, and the
# postcondition must bound all of them. This exercises the
# symbolic execution engine's path expansion capabilities.

# --- Function 1: 4 sequential if/else = 16 paths ---
# Each binary decision adds a different power of 2, creating
# 16 distinct results from 0 (all else) to 15 (all if).
# sig: (I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10 && $c >= 0 && $c <= 10 && $d >= 0 && $d <= 10
# post: $result >= 0 && $result <= 15
sub sixteen_paths {
    my ($a, $b, $c, $d) = @_;
    my $r = 0;
    if ($a > 5) {
        $r += 1;
    } else {
        $r += 0;
    }
    if ($b > 5) {
        $r += 2;
    } else {
        $r += 0;
    }
    if ($c > 5) {
        $r += 4;
    } else {
        $r += 0;
    }
    if ($d > 5) {
        $r += 8;
    } else {
        $r += 0;
    }
    return $r;
}

# --- Function 2: 5 decisions with die pruning = ~20 feasible paths ---
# 5 sequential if/else blocks. The first decision includes a die
# on a narrow condition, pruning some paths. Each surviving path
# contributes different addends.
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10 && $c >= 0 && $c <= 10 && $d >= 0 && $d <= 10 && $e >= 0 && $e <= 10
# post: $result >= 0 && $result <= 31
sub pruned_thirty_two_paths {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 5) {
        $r += 1;
    } else {
        die "bad" if ($a == 0 && $b == 0);
        $r += 0;
    }
    if ($b > 5) {
        $r += 2;
    } else {
        $r += 0;
    }
    if ($c > 5) {
        $r += 4;
    } else {
        $r += 0;
    }
    if ($d > 5) {
        $r += 8;
    } else {
        $r += 0;
    }
    if ($e > 5) {
        $r += 16;
    } else {
        $r += 0;
    }
    return $r;
}

# --- Function 3: Loop (3 iters) x branch (2-way) = 8 paths ---
# A 3-iteration loop with a binary branch each iteration creates
# 2^3 = 8 distinct paths. Each path yields a different sum
# depending on whether $x > $i at each step.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 5
# post: $result >= 3 && $result <= 18
sub loop_branch_eight_paths {
    my ($x) = @_;
    my $acc = 0;
    my $i;
    for ($i = 1; $i < 4; $i++) {
        if ($x > $i) {
            $acc += $x;
        } else {
            $acc += 1;
        }
    }
    return $acc;
}

# --- Function 4: 4 decisions with weighted additions = 16 paths ---
# Similar to function 1 but with non-power-of-2 addends, making
# the range computation more interesting. Each decision adds either
# a larger or smaller value. The verifier must compute the global
# min/max across all 16 paths.
# sig: (I64, I64, I64, I64) -> I64
# pre: $w >= 0 && $w <= 10 && $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10 && $z >= 0 && $z <= 10
# post: $result >= 4 && $result <= 40
sub weighted_sixteen_paths {
    my ($w, $x, $y, $z) = @_;
    my $r = 0;
    if ($w > 5) {
        $r += 10;
    } else {
        $r += 1;
    }
    if ($x > 5) {
        $r += 10;
    } else {
        $r += 1;
    }
    if ($y > 5) {
        $r += 10;
    } else {
        $r += 1;
    }
    if ($z > 5) {
        $r += 10;
    } else {
        $r += 1;
    }
    return $r;
}
