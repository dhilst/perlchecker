# =============================================================
# Round 84: Die-as-assert path pruning stress
# =============================================================
# Functions where strategically placed die statements mark paths
# as unreachable. The die conditions are provably impossible given
# preconditions and prior computation, but they help the verifier
# prune the path space, enabling tighter postconditions.

# --- Function 1: Arithmetic narrowing with redundant die guards ---
# Preconditions ensure $x and $y are both positive, so their sum
# is always >= 2. Die guards restate this, helping the verifier
# confirm tight bounds across 4 branching paths.
# sig: (Int, Int) -> Int
# pre: $x >= 1 && $x <= 50 && $y >= 1 && $y <= 50
# post: $result >= 2 && $result <= 100
sub arith_die_narrow {
    my ($x, $y) = @_;
    my $sum = $x + $y;
    die "sum too low" if ($sum < 2);
    die "sum too high" if ($sum > 100);
    my $r;
    if ($x > 25) {
        if ($y > 25) {
            $r = $sum;
        } else {
            $r = $sum;
        }
    } else {
        if ($y > 25) {
            $r = $sum;
        } else {
            $r = $sum;
        }
    }
    return $r;
}

# --- Function 2: Division safety via precondition-implied die ---
# Precondition ensures $den >= 2, so die for $den <= 0 and $den > 10
# are both unreachable. The die statements document and confirm the
# safe division range, helping the verifier bound the quotient.
# sig: (Int, Int) -> Int
# pre: $num >= 10 && $num <= 50 && $den >= 2 && $den <= 5
# post: $result >= 2 && $result <= 25
sub division_die_safe {
    my ($num, $den) = @_;
    die "zero denom" if ($den <= 0);
    die "denom too large" if ($den > 10);
    my $r = $num / $den;
    return $r;
}

# --- Function 3: Multi-branch with die confirming bounds ---
# Given $a >= 5, $b >= 5, and $c in [0,10], the results of
# max($a,$b) + $c are always >= 5 and <= 30. Die guards at each
# branch confirm this, helping verify the tight postcondition.
# sig: (Int, Int, Int) -> Int
# pre: $a >= 5 && $a <= 20 && $b >= 5 && $b <= 20 && $c >= 0 && $c <= 10
# post: $result >= 5 && $result <= 30
sub branch_die_bounds {
    my ($a, $b, $c) = @_;
    my $m;
    if ($a > $b) {
        $m = $a;
    } else {
        $m = $b;
    }
    my $r = $m + $c;
    die "below min" if ($r < 5);
    die "above max" if ($r > 30);
    return $r;
}

# --- Function 4: Loop accumulator with die confirming invariant ---
# Starting from $start in [0,3] and adding $step in [1,2] for 3
# iterations, the max is 3 + 2*3 = 9 and min is 0 + 1*3 = 3.
# Die after the loop confirms the accumulator stayed bounded.
# sig: (Int, Int) -> Int
# pre: $start >= 0 && $start <= 3 && $step >= 1 && $step <= 2
# post: $result >= 3 && $result <= 9
sub loop_die_bounded {
    my ($start, $step) = @_;
    my $acc = $start;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        $acc = $acc + $step;
    }
    die "overflow" if ($acc > 9);
    die "underflow" if ($acc < 3);
    return $acc;
}

# --- Function 5: Nested conditions with die at impossible leaves ---
# With $x in [0,20], $y in [0,20], $z in [0,20], the computations
# in each branch are bounded. Die guards at each leaf confirm the
# result is in [1,40], which is always true given the preconditions.
# sig: (Int, Int, Int) -> Int
# pre: $x >= 0 && $x <= 20 && $y >= 0 && $y <= 20 && $z >= 0 && $z <= 20
# post: $result >= 0 && $result <= 40
sub nested_die_confirm {
    my ($x, $y, $z) = @_;
    my $r;
    if ($x > 10) {
        if ($y > 10) {
            $r = $x + $y;
            die "impossible" if ($r > 40);
        } else {
            $r = $x + $z;
            die "impossible" if ($r > 40);
        }
    } else {
        if ($z > 10) {
            $r = $y + $z;
            die "impossible" if ($r > 40);
        } else {
            $r = $x + $y;
            die "impossible" if ($r > 40);
        }
    }
    die "negative" if ($r < 0);
    return $r;
}
