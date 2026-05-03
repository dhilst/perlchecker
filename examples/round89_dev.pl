# =============================================================
# Round 89: Mixed early-return + accumulator path stress
# =============================================================
# Functions that combine early return guards at the top with
# accumulator loops below. The verifier must prove postconditions
# hold both for early-exit paths AND for loop-computed paths,
# exercising diverse path types in symbolic execution.

# --- Function 1: Guard on zero, then accumulate in loop ---
# Returns 0 early for zero input. Otherwise, accumulates $x
# in a loop of 3 iterations. The postcondition bounds the result
# for both the early-return path (0) and the loop path (3*x).
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 5
# post: $result >= 0 && $result <= 15
sub guard_then_accum {
    my ($x) = @_;
    return 0 if ($x == 0);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        $acc += $x;
    }
    return $acc;
}

# --- Function 2: Multiple guards narrowing range before loop ---
# Two early returns handle boundary cases ($n <= 1 returns 1,
# $n >= 8 returns 24). The middle range [2,7] enters a loop
# that sums 1..3. All paths produce results in [1, 24].
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 10
# post: $result >= 1 && $result <= 24
sub multi_guard_loop {
    my ($n) = @_;
    return 1 if ($n <= 1);
    return 24 if ($n >= 8);
    my $acc = $n;
    my $i;
    for ($i = 1; $i < 4; $i++) {
        $acc += $i;
    }
    return $acc;
}

# --- Function 3: Guard on both params, then conditional loop ---
# Returns early if either input is zero. Otherwise enters a loop
# with a branch that adds either $a or $b depending on comparison
# with loop var. Creates 2^3 = 8 paths in the loop plus 2 early
# return paths = 10 total paths.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 4 && $b >= 0 && $b <= 4
# post: $result >= 0 && $result <= 12
sub dual_guard_cond_loop {
    my ($a, $b) = @_;
    return 0 if ($a == 0);
    return 0 if ($b == 0);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        if ($a > $i) {
            $acc += $a;
        } else {
            $acc += $b;
        }
    }
    return $acc;
}

# --- Function 4: Guard cascade + stride-2 accumulator ---
# Three guard clauses return fixed values for special cases.
# After guards, a stride-2 loop accumulates from the input.
# The verifier must prove the bound covers both guard constants
# and the loop-computed sum.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 10
# post: $result >= 0 && $result <= 20
sub cascade_guard_stride_loop {
    my ($x) = @_;
    return 0 if ($x == 0);
    return 1 if ($x == 1);
    return 20 if ($x >= 9);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 4; $i += 2) {
        $acc += $x;
    }
    return $acc;
}

# --- Function 5: Guard + loop with early exit via last ---
# Returns -1 for negative-equivalent input. Otherwise loops
# up to 4 iterations but breaks early if accumulator exceeds
# a threshold. Both the early return, the last-exit, and the
# full-loop completion paths must satisfy the postcondition.
# sig: (Int, Int) -> Int
# pre: $val >= 0 && $val <= 5 && $limit >= 1 && $limit <= 20
# post: $result >= -1 && $result <= 20
sub guard_loop_early_exit {
    my ($val, $limit) = @_;
    return -1 if ($val == 0);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 4; $i++) {
        $acc += $val;
        last if ($acc >= $limit);
    }
    return $acc;
}
