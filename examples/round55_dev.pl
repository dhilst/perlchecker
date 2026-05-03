# =============================================================
# Round 55: Array bounds + loop + conditional path stress
# =============================================================
# Exercises path expansion by combining array access with computed
# indices inside loops, conditional branches based on array values,
# push/pop with loop early exit, and scalar(@arr) in conditions.
# The verifier must track array contents through loop iterations
# and branches, stressing the symbolic execution engine.

# --- Computed index array scan with conditional accumulation ---
# Iterates over array elements, adds only positive values to sum.
# The verifier must track array reads at computed indices through
# both the positive and non-positive branches per iteration.
# sig: (Array<Int>, Int) -> Int
# pre: $len >= 1 && $len <= 4 && $arr[0] >= 0 && $arr[0] <= 5 && $arr[1] >= 0 && $arr[1] <= 5 && $arr[2] >= 0 && $arr[2] <= 5 && $arr[3] >= 0 && $arr[3] <= 5
# post: $result >= 0 && $result <= 20
sub sum_positive_elements {
    my ($arr, $len) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $len; $i++) {
        my $val = $arr[$i];
        if ($val > 0) {
            $sum = $sum + $val;
        }
    }
    return $sum;
}

# --- Array search with early exit (last) ---
# Searches for a target value in the array, returns 1 if found, 0 otherwise.
# Combines array access at computed index with loop early exit.
# sig: (Array<Int>, Int, Int) -> Int
# pre: $len >= 1 && $len <= 4 && $target >= 0 && $target <= 10
# post: $result >= 0 && $result <= 1
sub find_in_array {
    my ($arr, $len, $target) = @_;
    my $found = 0;
    my $i;
    for ($i = 0; $i < $len; $i++) {
        if ($arr[$i] == $target) {
            $found = 1;
            last;
        }
    }
    return $found;
}

# --- Push elements then pop and verify ---
# Pushes multiple computed values into array, then pops the last,
# verifying the verifier tracks array contents through push/pop.
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == 0 && $x >= 1 && $x <= 5
# post: $result == $x + 2
sub push_then_pop {
    my ($arr, $x) = @_;
    push(@arr, $x);
    push(@arr, $x + 1);
    push(@arr, $x + 2);
    my $top = pop(@arr);
    return $top;
}

# --- Scalar length drives branch logic ---
# Uses scalar(@arr) in conditions to create multiple paths,
# verifying the checker can reason about array length in branches.
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) == $len && $len >= 1 && $len <= 3 && $arr[0] >= 0 && $arr[0] <= 10 && $arr[1] >= 0 && $arr[1] <= 10 && $arr[2] >= 0 && $arr[2] <= 10
# post: $result >= 0 && $result <= 30
sub scalar_branching {
    my ($arr, $len, $extra) = @_;
    my $result = 0;
    if ($len >= 3) {
        $result = $arr[0] + $arr[1] + $arr[2];
    } elsif ($len >= 2) {
        $result = $arr[0] + $arr[1];
    } else {
        $result = $arr[0];
    }
    return $result;
}

# --- Multi-path array max with early exit ---
# Finds the maximum value in an array, exits early if a value
# exceeds a threshold. Combines array reads, comparisons,
# conditional updates, and loop control.
# sig: (Array<Int>, Int, Int) -> Int
# pre: $len >= 1 && $len <= 4 && $threshold >= 1 && $threshold <= 100 && $arr[0] >= 0 && $arr[0] <= 50 && $arr[1] >= 0 && $arr[1] <= 50 && $arr[2] >= 0 && $arr[2] <= 50 && $arr[3] >= 0 && $arr[3] <= 50
# post: $result >= 0 && $result <= 50
sub bounded_max_search {
    my ($arr, $len, $threshold) = @_;
    my $max_val = 0;
    my $i;
    for ($i = 0; $i < $len; $i++) {
        my $elem = $arr[$i];
        if ($elem > $max_val) {
            $max_val = $elem;
        }
        last if ($max_val >= $threshold);
    }
    return $max_val;
}
