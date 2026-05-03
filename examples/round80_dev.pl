# =============================================================
# Round 80: Postcondition disjunction path stress
# =============================================================
# Functions where the postcondition uses || (disjunction), requiring
# the verifier to prove that at least one disjunct holds on every
# path, rather than proving a single tight equality. Different paths
# satisfy different disjuncts.

# --- Function 1: Three-way classifier with disjunctive post ---
# Returns one of three sentinel values depending on input range.
# The postcondition is a 3-way disjunction; each path satisfies
# exactly one disjunct.
# sig: (Int) -> Int
# pre: $x >= -100 && $x <= 100
# post: $result == -1 || $result == 0 || $result == 1
sub sign_classify {
    my ($x) = @_;
    if ($x > 0) {
        return 1;
    } elsif ($x < 0) {
        return -1;
    } else {
        return 0;
    }
}

# --- Function 2: Nested branches with 4-way disjunction ---
# Two boolean conditions create 4 paths; each path returns a
# different value from the disjunction set {10, 20, 30, 40}.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 50 && $b >= 0 && $b <= 50
# post: $result == 10 || $result == 20 || $result == 30 || $result == 40
sub quadrant_value {
    my ($a, $b) = @_;
    if ($a > 25) {
        if ($b > 25) {
            return 40;
        } else {
            return 30;
        }
    } else {
        if ($b > 25) {
            return 20;
        } else {
            return 10;
        }
    }
}

# --- Function 3: Loop with early exit, disjunctive result ---
# Searches for a threshold crossing in a bounded loop. Returns
# either the iteration index where crossing happens or -1 if not
# found. Postcondition: result is -1 OR in valid index range.
# sig: (Int, Int) -> Int
# pre: $start >= 0 && $start <= 5 && $step >= 1 && $step <= 3
# post: $result == -1 || ($result >= 0 && $result <= 4)
sub find_threshold {
    my ($start, $step) = @_;
    my $val = $start;
    my $found = -1;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        if ($val > 10) {
            $found = $i;
            last;
        }
        $val = $val + $step;
    }
    return $found;
}

# --- Function 4: Cascading conditions with 5-way disjunction ---
# Classifies input into 5 buckets using elsif chain. Each bucket
# maps to a different disjunct in the postcondition.
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 100
# post: $result == 1 || $result == 2 || $result == 3 || $result == 4 || $result == 5
sub bucket_classify {
    my ($n) = @_;
    if ($n <= 20) {
        return 1;
    } elsif ($n <= 40) {
        return 2;
    } elsif ($n <= 60) {
        return 3;
    } elsif ($n <= 80) {
        return 4;
    } else {
        return 5;
    }
}

# --- Function 5: Compound paths with arithmetic disjunction ---
# Computes a value through multiple branches where each path
# produces a result satisfying one of two disjuncts: either the
# result equals a+b or the result equals a-b (when a>=b).
# The verifier must reason about which disjunct holds per path.
# sig: (Int, Int, Int) -> Int
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20 && $mode >= 0 && $mode <= 3
# post: $result == $a + $b || $result == $a - $b || $result == $a * 2 || $result == $b * 2
sub multi_op_select {
    my ($a, $b, $mode) = @_;
    if ($mode == 0) {
        return $a + $b;
    } elsif ($mode == 1) {
        return $a - $b;
    } elsif ($mode == 2) {
        return $a * 2;
    } else {
        return $b * 2;
    }
}
