# =============================================================
# Round 98: Boundary condition path explosion
# =============================================================
# Functions designed around boundary values (0, -1, 1) where
# conditional logic changes behavior at exact boundaries,
# creating paths that test the solver's ability to distinguish
# boundary cases from general cases.

# --- Function 1: Branch on exact boundary (== 0) ---
# Returns different values depending on whether $x is exactly 0,
# negative, or positive. The == 0 check creates a hard branch
# point that the solver must reason about precisely.
# sig: (Int) -> Int
# pre: $x >= -5 && $x <= 5
# post: (!($x == 0) || $result == 100) && (!($x > 0) || $result == $x + 10) && (!($x < 0) || $result == 0 - $x + 20)
sub boundary_zero {
    my ($x) = @_;
    if ($x == 0) {
        return 100;
    }
    if ($x > 0) {
        return $x + 10;
    }
    return 0 - $x + 20;
}

# --- Function 2: Three-way boundary check ---
# Tests < 0, == 0, > 0 with different computations. Each path
# produces a result in a specific range, and the overall result
# is always >= 1.
# sig: (Int) -> Int
# pre: $x >= -4 && $x <= 4
# post: $result >= 1 && $result <= 20
sub three_way_boundary {
    my ($x) = @_;
    if ($x < 0) {
        my $neg = 0 - $x;
        return $neg * 2;
    } elsif ($x == 0) {
        return 1;
    } else {
        return $x * 5;
    }
}

# --- Function 3: Multiple boundary checks in sequence ---
# Checks 0, then 1, then -1 in sequence, each adjusting the
# accumulator differently. Creates 4 distinct paths:
# (x==0), (x==1), (x==-1), (other).
# sig: (Int) -> Int
# pre: $x >= -3 && $x <= 3
# post: (!($x == 0) || $result == 50) && (!($x == 1) || $result == 30) && (!($x == -1) || $result == 40) && $result >= 5 && $result <= 50
sub multi_boundary_seq {
    my ($x) = @_;
    if ($x == 0) {
        return 50;
    }
    if ($x == 1) {
        return 30;
    }
    if ($x == -1) {
        return 40;
    }
    my $r = $x * $x + 5;
    return $r;
}

# --- Function 4: Boundary within a loop (last when counter hits value) ---
# Loop runs up to 5 iterations, but breaks when the counter equals
# the boundary value $b. The accumulator tracks how many iterations
# ran before hitting the boundary.
# sig: (Int) -> Int
# pre: $b >= 0 && $b <= 4
# post: $result == $b
sub boundary_loop_break {
    my ($b) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        last if ($i == $b);
        $acc += 1;
    }
    return $acc;
}

# --- Function 5: Compound boundary conditions ---
# Two parameters each checked against boundary 0, creating a
# 2x3 grid of paths: (a==0 vs a>0 vs a<0) x (b==0 vs b!=0).
# Each combination produces a result in a tight provable range.
# sig: (Int, Int) -> Int
# pre: $a >= -3 && $a <= 3 && $b >= -3 && $b <= 3
# post: $result >= 0 && $result <= 15
sub compound_boundary {
    my ($a, $b) = @_;
    my $r = 0;
    if ($a == 0) {
        $r += 5;
    } elsif ($a > 0) {
        $r += $a;
    } else {
        $r += 0 - $a;
    }
    if ($b == 0) {
        $r += 6;
    } else {
        my $babs = abs($b);
        $r += $babs;
    }
    return $r;
}
