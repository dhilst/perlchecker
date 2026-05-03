# =============================================================
# Round 95: List assign + conditional swap path stress
# =============================================================
# Functions using list assignment ($a, $b) = ($b, $a) for swapping
# variables inside conditional branches, creating paths where
# variable values depend on whether swaps occurred. The verifier
# must track variable identities through swap operations across
# multiple conditional branches.

# --- Function 1: Conditional swap to order two values ---
# Swaps x,y so the smaller is first. Returns the smaller value.
# For a in [1,5], b in [6,10]: a < b always, so no swap occurs,
# and result == a. But we also handle a > b symmetrically.
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10
# post: $result >= 1 && $result <= 10
sub conditional_sort_pair {
    my ($a, $b) = @_;
    my ($x, $y) = ($a, $b);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    return $x;
}

# --- Function 2: Two conditional swaps creating 4 paths ---
# Given three values, conditionally swap adjacent pairs. The
# combination of two swap decisions creates 4 paths. The
# postcondition verifies the first element is bounded.
# sig: (Int, Int, Int) -> Int
# pre: $a >= 1 && $a <= 5 && $b >= 3 && $b <= 7 && $c >= 5 && $c <= 9
# post: $result >= 1 && $result <= 7
sub double_swap_paths {
    my ($a, $b, $c) = @_;
    my ($x, $y, $z) = ($a, $b, $c);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    if ($y > $z) {
        ($y, $z) = ($z, $y);
    }
    return $y;
}

# --- Function 3: Conditional swap + min equivalence ---
# Computes min(a,b) using conditional swap: swap if a > b, then
# take the first element. Verifies that the result is <= both
# original inputs (expressed as result <= a and result <= b via
# the bound that result <= min of the ranges).
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 8 && $b >= 1 && $b <= 8
# post: $result >= 1 && $result <= 8
sub swap_for_min {
    my ($a, $b) = @_;
    my ($x, $y) = ($a, $b);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    return $x;
}

# --- Function 4: Conditional swap + max equivalence ---
# Computes max(a,b) using conditional swap: swap if a > b, then
# take the second element. The second element after a conditional
# swap is always the larger value.
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 8 && $b >= 1 && $b <= 8
# post: $result >= 1 && $result <= 8
sub swap_for_max {
    my ($a, $b) = @_;
    my ($x, $y) = ($a, $b);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    return $y;
}

# --- Function 5: Three-element bubble pass using conditional swaps ---
# Performs one bubble-sort pass on three elements using two
# adjacent conditional swaps. After the pass, the largest element
# is guaranteed to be in the last position. Returns the last
# element (the maximum).
# sig: (Int, Int, Int) -> Int
# pre: $a >= 1 && $a <= 6 && $b >= 1 && $b <= 6 && $c >= 1 && $c <= 6
# post: $result >= 1 && $result <= 6
sub bubble_pass_max {
    my ($a, $b, $c) = @_;
    my ($x, $y, $z) = ($a, $b, $c);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    if ($y > $z) {
        ($y, $z) = ($z, $y);
    }
    return $z;
}
