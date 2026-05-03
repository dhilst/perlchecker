# =============================================================
# Round 66: Accumulator + last/next combo path stress
# =============================================================
# Functions implementing classic accumulator algorithms using loops
# with BOTH last AND next in the same loop body, creating complex
# break/skip interaction paths for the symbolic execution engine.

# --- Function 1: Sum with skip and cap ---
# Accumulates a running sum, skipping values at even indices (next),
# and stopping early when the sum exceeds a cap (last).
# sig: (Int, Int) -> Int
# pre: $step >= 1 && $step <= 3 && $cap >= 5 && $cap <= 15
# post: $result >= 1 && $result <= 15
sub sum_skip_cap {
    my ($step, $cap) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        next if ($i % 2 == 0);
        $acc = $acc + $step;
        last if ($acc >= $cap);
    }
    my $r = ($acc > 0) ? $acc : $step;
    return $r;
}

# --- Function 2: Find first value above threshold ---
# Iterates indices, skipping negative-step indices with next,
# exits with last on first match. Returns the matching index + 1.
# sig: (Int, Int) -> Int
# pre: $thresh >= 1 && $thresh <= 4 && $offset >= 0 && $offset <= 2
# post: $result >= 1 && $result <= 5
sub find_first_above {
    my ($thresh, $offset) = @_;
    my $found = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        my $val = $i + $offset;
        next if ($val <= $thresh);
        $found = $i + 1;
        last;
    }
    my $r = ($found > 0) ? $found : 5;
    return $r;
}

# --- Function 3: Count odd values skipping small ones ---
# Counts how many values pass a filter, using next to skip
# even values and last to stop after finding enough.
# sig: (Int, Int) -> Int
# pre: $base >= 0 && $base <= 2 && $max_count >= 1 && $max_count <= 3
# post: $result >= 0 && $result <= 3
sub count_odd_limited {
    my ($base, $max_count) = @_;
    my $count = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        my $val = $i + $base;
        next if ($val % 2 == 0);
        $count = $count + 1;
        last if ($count >= $max_count);
    }
    return $count;
}

# --- Function 4: Running minimum with skip and early exit ---
# Finds the minimum of computed values, skipping some indices
# with next, and exiting early with last when min is already 0.
# sig: (Int) -> Int
# pre: $start >= 2 && $start <= 8
# post: $result >= 0 && $result <= 8
sub running_min_skip_exit {
    my ($start) = @_;
    my $min = $start;
    my $i;
    for ($i = 0; $i < 4; $i++) {
        next if ($i == 0);
        my $candidate = $start - $i - 1;
        my $val = ($candidate >= 0) ? $candidate : 0;
        last if ($val == 0);
        $min = ($val < $min) ? $val : $min;
    }
    return $min;
}

# --- Function 5: Accumulate with alternating skip/break conditions ---
# Complex interleaving: next skips when accumulator is odd,
# last exits when accumulator exceeds limit. Tests both flags
# interacting on the same variable.
# sig: (Int, Int) -> Int
# pre: $inc >= 1 && $inc <= 2 && $limit >= 3 && $limit <= 8
# post: $result >= 0 && $result <= 8
sub alternating_skip_break {
    my ($inc, $limit) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        next if ($acc % 2 == 1);
        $acc = $acc + $inc;
        last if ($acc >= $limit);
    }
    return $acc;
}
