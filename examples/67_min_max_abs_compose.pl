# =============================================================
# Round 67: Min/max/abs composition path stress
# =============================================================
# Functions using nested min(max(...)), abs(min(...)), and other
# compositions of these builtins in conditional branches, creating
# paths where the verifier must reason about piecewise-linear functions.

# --- Function 1: Clamp value to range using nested min/max ---
# Classic clamping pattern: min(max(x, lo), hi)
# sig: (I64, I64, I64) -> I64
# pre: $lo >= 0 && $lo <= 5 && $hi >= 6 && $hi <= 10 && $x >= -10 && $x <= 20
# post: $result >= 0 && $result <= 10
sub clamp_value {
    my ($x, $lo, $hi) = @_;
    my $r = min(max($x, $lo), $hi);
    return $r;
}

# --- Function 2: Absolute distance with clamped inputs ---
# Computes abs(min(a,b) - max(c,d)), a distance between composed values.
# sig: (I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 5 && $b >= 0 && $b <= 5 && $c >= 0 && $c <= 5 && $d >= 0 && $d <= 5
# post: $result >= 0 && $result <= 5
sub abs_min_max_distance {
    my ($a, $b, $c, $d) = @_;
    my $lo = min($a, $b);
    my $hi = max($c, $d);
    my $diff = $lo - $hi;
    my $r = abs($diff);
    return $r;
}

# --- Function 3: Branch-dependent clamping ---
# Different clamp ranges depending on a condition, creating
# separate ITE trees per branch.
# sig: (I64, I64) -> I64
# pre: $x >= -10 && $x <= 10 && $mode >= 0 && $mode <= 1
# post: $result >= 0 && $result <= 8
sub branch_clamp {
    my ($x, $mode) = @_;
    my $r;
    if ($mode == 0) {
        $r = min(max($x, 0), 5);
    } else {
        $r = min(max($x, 2), 8);
    }
    return $r;
}

# --- Function 4: Abs result in further branching ---
# Uses abs() result in a comparison that creates further branches,
# multiplying ITE paths.
# sig: (I64, I64) -> I64
# pre: $x >= -5 && $x <= 5 && $y >= -5 && $y <= 5
# post: $result >= 0 && $result <= 10
sub abs_branch_cascade {
    my ($x, $y) = @_;
    my $d = abs($x - $y);
    my $r;
    if ($d > 3) {
        $r = min($d, 10);
    } else {
        $r = max($d, 0);
    }
    return $r;
}

# --- Function 5: Triple composition with nested abs ---
# Nests abs inside min/max creating deep ITE trees.
# sig: (I64, I64, I64) -> I64
# pre: $a >= -4 && $a <= 4 && $b >= -4 && $b <= 4 && $c >= -4 && $c <= 4
# post: $result >= 0 && $result <= 8
sub triple_abs_composition {
    my ($a, $b, $c) = @_;
    my $ab = abs($a - $b);
    my $bc = abs($b - $c);
    my $r = min(max($ab, $bc), 8);
    return $r;
}
