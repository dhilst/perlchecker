# =============================================================
# Round 93: Power + shift + comparison path stress
# =============================================================
# Functions combining exponentiation (**) with shift operators
# (<< and >>) and comparison chains. The verifier must reason
# about exponential growth vs shift-based powers of 2, exercising
# cross-theory SMT reasoning (integer power vs bitvector shift).

# --- Function 1: Compare x**2 with x<<1 and branch ---
# For x in [1,4]: x**2 ranges [1,16], x<<1 ranges [2,8].
# When x==1: 1 < 2, when x==2: 4 == 4, when x>=3: x**2 > x<<1.
# Three-way branching on that comparison produces bounded results.
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 4
# post: $result >= 1 && $result <= 16
sub power_vs_shift_branch {
    my ($x) = @_;
    my $sq = $x ** 2;
    my $dbl = $x << 1;
    my $r;
    if ($sq > $dbl) {
        $r = $sq;
    } elsif ($sq == $dbl) {
        $r = $sq + $dbl;
    } else {
        $r = $dbl;
    }
    return $r;
}

# --- Function 2: Shift for pow-2, ** for general, compare magnitudes ---
# Computes 1<<n (which is 2**n) and base**2.
# For n in [1,3], base in [1,3]:
#   1<<n: 2,4,8.  base**2: 1,4,9.
# Branches on which is larger, then returns the smaller plus 1.
# sig: (Int, Int) -> Int
# pre: $n >= 1 && $n <= 3 && $base >= 1 && $base <= 3
# post: $result >= 2 && $result <= 9
sub shift_pow2_vs_general_power {
    my ($n, $base) = @_;
    my $shift_val = 1 << $n;
    my $pow_val = $base ** 2;
    my $r;
    if ($shift_val > $pow_val) {
        $r = $pow_val + 1;
    } elsif ($shift_val == $pow_val) {
        $r = $shift_val;
    } else {
        $r = $shift_val + 1;
    }
    return $r;
}

# --- Function 3: Spaceship on shift results for 3-way branching ---
# Computes a<<1 and b>>1, then uses spaceship to get -1/0/1.
# For a in [1,4], b in [2,8]: a<<1 in [2,8], b>>1 in [1,4].
# Adds the spaceship result to a base value derived from a+b.
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 4 && $b >= 2 && $b <= 8
# post: $result >= 2 && $result <= 13
sub spaceship_on_shifts {
    my ($a, $b) = @_;
    my $left = $a << 1;
    my $right = $b >> 1;
    my $cmp = $left <=> $right;
    my $base = $a + $b;
    my $r;
    if ($cmp == 1) {
        $r = $base + 1;
    } elsif ($cmp == 0) {
        $r = $base;
    } else {
        $r = $base - 1;
    }
    return $r;
}

# --- Function 4: Power in one branch, shift in another, bound both ---
# Input x in [1,4]: if x <= 2, compute x**3 (1 or 8);
# if x > 2, compute x<<2 (12 or 16). All results in [1,16].
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 4
# post: $result >= 1 && $result <= 16
sub power_or_shift_path {
    my ($x) = @_;
    my $r;
    if ($x <= 2) {
        $r = $x ** 3;
    } else {
        $r = $x << 2;
    }
    return $r;
}

# --- Function 5: Multi-level shifts and power with nested comparisons ---
# Computes x**2 and (x<<1)>>1 (which simplifies to x for positive x),
# then compares them with a cascading branch structure.
# For x in [1,4]: x**2 in [1,16], (x<<1)>>1 == x in [1,4].
# The ratio x**2 / x = x, but we express bounds via additive branches.
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 4
# post: $result >= 2 && $result <= 20
sub nested_shift_power_compare {
    my ($x) = @_;
    my $sq = $x ** 2;
    my $shifted = ($x << 1) >> 1;
    my $diff = $sq - $shifted;
    my $r;
    if ($diff > 5) {
        $r = $sq + $shifted;
    } elsif ($diff > 1) {
        $r = $sq + 1;
    } elsif ($diff == 0) {
        $r = $shifted + 1;
    } else {
        $r = $shifted + $diff + 1;
    }
    return $r;
}
