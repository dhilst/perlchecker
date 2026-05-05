# =============================================================
# Round 91: Negative literal + abs + min/max path stress
# =============================================================
# Functions combining negative integer literals with abs(), min(),
# and max() in branching logic, creating paths where the verifier
# must reason about sign changes and absolute value properties.

# --- Function 1: Absolute value of negative-shifted input ---
# Shifts input by a negative constant, takes abs, and verifies
# the result is always non-negative and bounded.
# sig: (I64) -> I64
# pre: $x >= -5 && $x <= 5
# post: $result >= 0 && $result <= 15
sub abs_negative_shift {
    my ($x) = @_;
    my $shifted = $x - 10;
    my $r = abs($shifted);
    return $r;
}

# --- Function 2: Sign-based branching with abs and min ---
# Branches on sign of input, applies different abs/min combos.
# Negative branch: clamp abs(x) to at most 5 via min.
# Non-negative branch: clamp x to at most 3 via min.
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result >= 0 && $result <= 5
sub sign_branch_abs_min {
    my ($x) = @_;
    my $r;
    if ($x < 0) {
        $r = min(abs($x), 5);
    } else {
        $r = min($x, 3);
    }
    return $r;
}

# --- Function 3: Distance from negative bound with max clamping ---
# Computes distance from -5, then clamps the result between 0 and 8
# using max and min. Multiple branches based on input sign create
# different constraint paths.
# sig: (I64) -> I64
# pre: $x >= -8 && $x <= 8
# post: $result >= 0 && $result <= 8
sub distance_from_neg_bound {
    my ($x) = @_;
    my $dist = abs($x - (-5));
    my $r;
    if ($x >= 0) {
        $r = min($dist, 8);
    } else {
        $r = max(min($dist, 8), 0);
    }
    return $r;
}

# --- Function 4: Multi-branch sign reasoning with min/max ---
# Four branches based on sign comparisons with negative constants,
# each applying a different combination of abs/min/max. The verifier
# must track sign constraints through each branch to prove bounds.
# sig: (I64, I64) -> I64
# pre: $x >= -10 && $x <= 10 && $y >= -10 && $y <= 10
# post: $result >= 0 && $result <= 20
sub multi_branch_sign_reasoning {
    my ($x, $y) = @_;
    my $r;
    if ($x >= 0 && $y >= 0) {
        $r = min($x + $y, 20);
    } elsif ($x < 0 && $y >= 0) {
        $r = max(abs($x) - $y, 0);
    } elsif ($x >= 0 && $y < 0) {
        $r = min(abs($y) + $x, 20);
    } else {
        $r = min(abs($x) + abs($y), 20);
    }
    return $r;
}

# --- Function 5: Cascading abs with negative literal offsets ---
# Computes abs values with negative offsets at each stage, creating
# a chain of ITE expansions. Each step depends on the prior abs
# result compared against a negative threshold.
# sig: (I64) -> I64
# pre: $x >= -6 && $x <= 6
# post: $result >= 0 && $result <= 12
sub cascading_abs_neg_offsets {
    my ($x) = @_;
    my $a = abs($x + 3);
    my $b = abs($x - 3);
    my $r;
    if ($a > $b) {
        $r = min($a, 9);
    } else {
        $r = min($b, 9);
    }
    return $r;
}
