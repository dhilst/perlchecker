# =============================================================
# Domain: min() and max() builtins
# =============================================================
# Perl's List::Util provides min/max. Having these as verified
# builtins lets users express clamping and range constraints
# directly in contracts.
#
# BOUNDARY PUSH: min() and max() as builtin functions taking
# two Int arguments and returning Int, desugared to ite.
# =============================================================

# --- min returns the smaller of two values ---
# sig: (Int, Int) -> Int
# post: $result <= $x && $result <= $y
sub test_min_le {
    my ($x, $y) = @_;
    return min($x, $y);
}

# --- max returns the larger of two values ---
# sig: (Int, Int) -> Int
# post: $result >= $x && $result >= $y
sub test_max_ge {
    my ($x, $y) = @_;
    return max($x, $y);
}

# --- min is one of its arguments ---
# sig: (Int, Int) -> Int
# post: $result == $x || $result == $y
sub test_min_is_arg {
    my ($x, $y) = @_;
    return min($x, $y);
}

# --- max is one of its arguments ---
# sig: (Int, Int) -> Int
# post: $result == $x || $result == $y
sub test_max_is_arg {
    my ($x, $y) = @_;
    return max($x, $y);
}

# --- clamping pattern: max(lo, min(hi, x)) ---
# sig: (Int, Int, Int) -> Int
# pre: $lo <= $hi
# post: $result >= $lo && $result <= $hi
sub clamp {
    my ($x, $lo, $hi) = @_;
    my $r = max($lo, min($hi, $x));
    return $r;
}

# --- min and max together bound the range ---
# sig: (Int, Int) -> Int
# post: $result >= 0
sub min_max_nonneg {
    my ($x, $y) = @_;
    my $r = max(0, min($x, $y));
    return $r;
}
