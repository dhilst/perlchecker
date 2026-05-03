# =============================================================
# Round 70: Comprehensive loop unroll stress
# =============================================================
# Functions exercising all loop types (for, while, do-while, until,
# do-until) at maximum unroll depth with complex conditional bodies,
# stressing the SSA phi-node generation and path expansion engine.

# --- Function 1: For-loop at 5 iterations with conditional body ---
# Each iteration has an if/else that depends on the loop variable,
# creating 2^5 = 32 potential paths through unrolling.
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 10
# post: $result >= 5 && $result <= 55
sub for_branch_stress {
    my ($x) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        if ($x > $i * 2) {
            $acc += $x;
        } else {
            $acc += 1;
        }
    }
    return $acc;
}

# --- Function 2: While-loop with multiple exit conditions ---
# Counts up while two conditions hold. With x in [1,5], loop runs
# at most 5 times (bounded by both x and the counter limit).
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 5
# post: $result >= 1 && $result <= 5
sub while_multi_exit {
    my ($x) = @_;
    my $count = 0;
    my $limit = $x;
    while ($count < $limit && $count < 5) {
        $count += 1;
    }
    return $count;
}

# --- Function 3: Do-while with accumulator and branch ---
# Accumulates values with different increments depending on parity.
# Runs exactly n times (n in [1,5]). Odd iterations add 3, even add 1.
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 1 && $result <= 13
sub do_while_accum_branch {
    my ($n) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        if ($i % 2 == 0) {
            $acc += 1;
        } else {
            $acc += 3;
        }
        $i++;
    } while ($i < $n);
    return $acc;
}

# --- Function 4: Until-loop with ternary in body ---
# Decrements from x down to 0, accumulating either 2 or 1 at each
# step depending on whether current value exceeds a threshold.
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 5
# post: $result >= 1 && $result <= 10
sub until_ternary_body {
    my ($x) = @_;
    my $acc = 0;
    my $cur = $x;
    until ($cur <= 0) {
        my $add = ($cur > 3) ? 2 : 1;
        $acc += $add;
        $cur -= 1;
    }
    return $acc;
}

# --- Function 5: Do-until with nested branch and early-style logic ---
# Builds up a result using nested conditions inside a do-until loop.
# With n in [1,4], iterates 1 to 4 times. Each iteration classifies
# the counter into three ranges producing different increments.
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 4
# post: $result >= 2 && $result <= 11
sub do_until_nested {
    my ($n) = @_;
    my $r = 0;
    my $i = 0;
    do {
        if ($i < 1) {
            $r += 2;
        } elsif ($i < 3) {
            $r += 3;
        } else {
            $r += 1;
        }
        $i++;
    } until ($i >= $n);
    return $r;
}
