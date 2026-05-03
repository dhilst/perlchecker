# =============================================================
# Round 62: Die-with-message path pruning stress
# =============================================================
# Exercises die with descriptive string messages in complex guard
# scenarios. The preconditions already guarantee die is unreachable,
# but the die statements help the symbolic executor prune infeasible
# paths. Complex branching after die guards produces many paths
# that are only provable because die eliminates impossible states.

# --- Multi-guard with descriptive messages: 4 die prune impossible paths ---
# Precondition guarantees $x in [10,90], so all die statements are
# unreachable. The die messages document the invariant. After pruning,
# the tight postcondition on $x * 2 is provable.
# sig: (Int) -> Int
# pre: $x >= 10 && $x <= 90
# post: $result >= 20 && $result <= 180
sub multi_guard_narrow {
    my ($x) = @_;
    die "x too small: below minimum threshold of 10" if ($x < 10);
    die "x too large: exceeds maximum threshold of 90" if ($x > 90);
    my $r = $x * 2;
    return $r;
}

# --- Die-unless with nested branches: 4 paths after pruning ---
# Precondition gives $a in [1,50] and $b in [1,50], making die-unless
# unreachable. Nested if/else then creates 4 paths, all bounded.
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 50 && $b >= 1 && $b <= 50
# post: $result >= 2 && $result <= 100
sub die_unless_with_branches {
    my ($a, $b) = @_;
    die "a must be positive: got non-positive value" unless ($a >= 1);
    die "b must be positive: got non-positive value" unless ($b >= 1);
    my $r;
    if ($a > 25) {
        if ($b > 25) {
            $r = $a + $b;
        } else {
            $r = $a + $b;
        }
    } else {
        if ($b > 25) {
            $r = $a + $b;
        } else {
            $r = $a + $b;
        }
    }
    return $r;
}

# --- Cascading die to document tight arithmetic invariants ---
# Precondition restricts $x in [5,15], $y in [10,20], $z in [1,5].
# Six die statements document and assert these bounds. After all
# guards, $x + $y + $z is provably in [16, 40].
# sig: (Int, Int, Int) -> Int
# pre: $x >= 5 && $x <= 15 && $y >= 10 && $y <= 20 && $z >= 1 && $z <= 5
# post: $result >= 16 && $result <= 40
sub cascading_die_tight_bound {
    my ($x, $y, $z) = @_;
    die "x below minimum: must be at least 5" if ($x < 5);
    die "x above maximum: must be at most 15" if ($x > 15);
    die "y below minimum: must be at least 10" if ($y < 10);
    die "y above maximum: must be at most 20" if ($y > 20);
    die "z below minimum: must be at least 1" if ($z < 1);
    die "z above maximum: must be at most 5" if ($z > 5);
    my $r = $x + $y + $z;
    return $r;
}

# --- Die in if-block followed by ternary branching ---
# Precondition ensures $n in [1,10], making die unreachable.
# After die prunes the impossible path, ternary creates 2 paths
# both provably bounded by the narrowed range.
# sig: (Int, Int) -> Int
# pre: $n >= 1 && $n <= 10 && $m >= 1 && $m <= 10
# post: $result >= 2 && $result <= 100
sub die_in_block_with_ternary {
    my ($n, $m) = @_;
    if ($n <= 0) {
        die "n must be positive for computation";
    }
    die "m out of safe range: must be positive" if ($m <= 0);
    my $base = ($n > 5) ? $n * $m : $n + $m;
    return $base;
}

# --- Complex: multiple die + loop + conditional ---
# Precondition restricts $start in [1,5] and $factor in [1,3].
# Die guards document these invariants. A bounded loop accumulates
# values; the postcondition is tight because die establishes the
# range for symbolic reasoning about the loop body.
# sig: (Int, Int) -> Int
# pre: $start >= 1 && $start <= 5 && $factor >= 1 && $factor <= 3
# post: $result >= 4 && $result <= 60
sub die_guard_with_loop {
    my ($start, $factor) = @_;
    die "start must be at least 1: invalid input" unless ($start >= 1);
    die "start exceeds maximum of 5" if ($start > 5);
    die "factor must be at least 1: invalid multiplier" unless ($factor >= 1);
    die "factor exceeds maximum of 3" if ($factor > 3);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 4; $i++) {
        $acc = $acc + $start * $factor;
    }
    return $acc;
}
