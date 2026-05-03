# =============================================================
# Round 83: Ternary in loop + conditional path stress
# =============================================================
# Functions that use ternary expressions inside loop bodies,
# creating path multiplication: each unrolled iteration with a
# ternary creates 2 sub-paths, so n iterations produce 2^n paths.
# All postconditions are provable with bounded iteration (<=4).

# --- Function 1: Ternary accumulator in for-loop ---
# Each iteration adds either 1 or 2 to accumulator based on
# a ternary condition on the loop variable. 3 iterations = 8 paths.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 10
# post: $result >= 3 && $result <= 6
sub ternary_accum_loop {
    my ($x) = @_;
    my $acc = 0;
    my $i = 0;
    for ($i = 0; $i < 3; $i++) {
        $acc += ($i < $x) ? 2 : 1;
    }
    return $acc;
}

# --- Function 2: Ternary modifying loop condition variable ---
# The ternary result modifies a variable used in the next iteration's
# ternary condition, creating dependent path chains.
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 2 && $result <= 8
sub ternary_condition_modifier {
    my ($n) = @_;
    my $acc = 0;
    my $threshold = 2;
    my $i = 0;
    my $add = 0;
    for ($i = 0; $i < 3; $i++) {
        $add = ($n > $threshold) ? 2 : 1;
        $acc += $add;
        $threshold = ($add == 2) ? $threshold + 1 : $threshold;
    }
    return $acc;
}

# --- Function 3: Nested ternary in for-loop with array access ---
# Uses a nested ternary inside the loop to select between three
# possible values based on array element comparisons.
# sig: (Array<Int>, Int) -> Int
# pre: $n >= 1 && $n <= 3 && $arr[0] >= 0 && $arr[0] <= 5 && $arr[1] >= 0 && $arr[1] <= 5 && $arr[2] >= 0 && $arr[2] <= 5
# post: $result >= 0 && $result <= 15
sub nested_ternary_array_loop {
    my ($arr, $n) = @_;
    my $sum = 0;
    my $i = 0;
    for ($i = 0; $i < $n; $i++) {
        $sum += ($arr[$i] > 3) ? 5 : ($arr[$i] > 1) ? 3 : 0;
    }
    return $sum;
}

# --- Function 4: Multiple ternaries per iteration ---
# Two ternaries per iteration create 4 sub-paths each iteration.
# 2 iterations with 2 ternaries each = 4^2 = 16 paths.
# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 0 && $result <= 14
sub multi_ternary_per_iter {
    my ($x, $y) = @_;
    my $acc = 0;
    my $i = 0;
    my $a = 0;
    my $b = 0;
    for ($i = 0; $i < 2; $i++) {
        $a = ($x > $i + 2) ? 3 : 1;
        $b = ($y > $i + 3) ? 4 : 0;
        $acc += $a + $b;
    }
    return $acc;
}

# --- Function 5: Ternary with early exit and path convergence ---
# Uses ternary in loop with last-if to prune paths early,
# testing that the verifier handles path pruning correctly.
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 6
# post: $result >= 0 && $result <= 12
sub ternary_early_exit {
    my ($n) = @_;
    my $acc = 0;
    my $i = 0;
    my $val = 0;
    for ($i = 0; $i < 4; $i++) {
        $val = ($n > $i) ? 3 : 1;
        $acc += $val;
        last if ($acc > 9);
    }
    return $acc;
}
