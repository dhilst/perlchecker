# =============================================================
# Domain: Ternary/Conditional Expression (inline if-else)
# =============================================================
# Perl supports the ternary operator ($cond ? $then : $else),
# which is extremely common in real Perl code for inline value
# selection. This is a pure expression with no side effects.
#
# BOUNDARY PUSH: Ternary conditional expression as a value.
# =============================================================

# --- Absolute value using ternary ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result >= 0
sub abs_val {
    my ($x) = @_;
    my $r = ($x >= 0) ? $x : (0 - $x);
    return $r;
}

# --- Min of two integers using ternary ---
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $y >= 0
# post: $result <= $x && $result <= $y
sub min_of {
    my ($x, $y) = @_;
    my $r = ($x <= $y) ? $x : $y;
    return $r;
}

# --- Clamp to range using nested ternary ---
# sig: (I64, I64, I64) -> I64
# pre: $lo <= $hi
# post: $result >= $lo && $result <= $hi
sub clamp {
    my ($x, $lo, $hi) = @_;
    my $r = ($x < $lo) ? $lo : (($x > $hi) ? $hi : $x);
    return $r;
}
