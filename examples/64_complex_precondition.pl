# =============================================================
# Round 64: Complex precondition interaction path stress
# =============================================================
# Exercises functions where preconditions constrain multiple
# parameters with interdependencies (e.g., $a + $b <= 10,
# $a < $b, $x * $y > 0), creating complex feasible-path sets
# that the solver must navigate. Postconditions are only provable
# because of the precondition interactions.

# --- Sum-bounded pair with ordering constraint ---
# Precondition: $a + $b <= 10, $a >= 1, $b >= 1, $a < $b.
# This means $a in [1,4] and $b in [2,9] with $a < $b.
# The branch on $a + $b > 8 is sometimes feasible, sometimes not.
# But $a * $b is always <= 36 (max: $a=4, $b=9... but $a+$b<=10
# so max product is e.g. $a=4,$b=6 => 24, or $a=3,$b=7 => 21).
# Actually max is $a=4,$b=6 => 24, or $a=5... no $a<$b and $a+$b<=10
# means $a<=4. With $a=4,$b=6: product=24. With $a=3,$b=7: 21.
# Tightest: $a=4,$b=6 => 24. So result <= 34 (24+10).
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $b >= 1 && $a + $b <= 10 && $a < $b
# post: $result >= 2 && $result <= 34
sub sum_bounded_pair {
    my ($a, $b) = @_;
    my $product = $a * $b;
    my $r;
    if ($a + $b > 8) {
        $r = $product + $a + $b;
    } else {
        $r = $product + $a;
    }
    return $r;
}

# --- Ordered triple with sum constraint ---
# Precondition: $x < $y, $y < $z, $x + $y + $z <= 12,
# $x >= 1, $y >= 1, $z >= 1.
# Since $x < $y < $z and all >= 1, minimum is $x=1,$y=2,$z=3.
# With sum <= 12: max is e.g. $x=1,$y=2,$z=9 or $x=2,$y=3,$z=7.
# The ordering guarantees $z > $y > $x, so $z - $x >= 2 always.
# sig: (Int, Int, Int) -> Int
# pre: $x >= 1 && $y >= 1 && $z >= 1 && $x < $y && $y < $z && $x + $y + $z <= 12
# post: $result >= 2 && $result <= 11
sub ordered_triple_diff {
    my ($x, $y, $z) = @_;
    my $diff = $z - $x;
    if ($diff > $y) {
        return $diff;
    } else {
        return $y;
    }
}

# --- Product sign constraint with bounded sum ---
# Precondition: $a * $b > 0 means both same sign (and nonzero).
# Combined with $a >= 1 && $b >= 1, both are positive.
# With $a + $b <= 8: max individual is 7 (if other is 1).
# The branch $a > $b is sometimes true, sometimes false,
# but result is always the larger value which is <= 7.
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $b >= 1 && $a * $b > 0 && $a + $b <= 8
# post: $result >= 1 && $result <= 7
sub product_sign_bounded {
    my ($a, $b) = @_;
    my $r;
    if ($a > $b) {
        $r = $a;
    } else {
        $r = $b;
    }
    return $r;
}

# --- Interdependent bounds narrowing ---
# Precondition: $p >= 2, $q >= 2, $p + $q <= 9, $p <= $q.
# So $p in [2,4] (since $p <= $q and $p+$q<=9 means 2*$p<=9).
# And $q in [2,7] but constrained by $p+$q<=9.
# The result $p * $q + $p - $q:
# Min: $p=2,$q=2 => 4+2-2=4. Actually $p=2,$q=7 => 14+2-7=9.
# Max: $p=4,$q=5 => 20+4-5=19. $p=4,$q=4 => 16+4-4=16.
# $p=3,$q=6 => 18+3-6=15. $p=2,$q=7 => 14+2-7=9.
# So range includes [4..19]. Let's use a safe bound.
# Min: $p=2,$q=2 => 4. Max: $p=4,$q=5 => 19.
# sig: (Int, Int) -> Int
# pre: $p >= 2 && $q >= 2 && $p + $q <= 9 && $p <= $q
# post: $result >= 4 && $result <= 19
sub interdependent_bounds {
    my ($p, $q) = @_;
    my $r = $p * $q + $p - $q;
    if ($r > 10) {
        return $r;
    } else {
        return $r;
    }
}

# --- Diamond constraint: two paths both provable ---
# Precondition: $m >= 1, $n >= 1, $m + $n <= 6, $m <= $n.
# So $m in [1,3], $n in [1,5] with $m+$n<=6.
# If $m == $n: result = $m * 2 (range 2..6 since $m+$n<=6 means $m<=3)
# If $m != $n (i.e., $m < $n): result = $m + $n (range 3..6)
# Overall: result in [2, 6].
# sig: (Int, Int) -> Int
# pre: $m >= 1 && $n >= 1 && $m + $n <= 6 && $m <= $n
# post: $result >= 2 && $result <= 6
sub diamond_constraint {
    my ($m, $n) = @_;
    my $r;
    if ($m == $n) {
        $r = $m + $n;
    } else {
        $r = $m + $n;
    }
    return $r;
}
