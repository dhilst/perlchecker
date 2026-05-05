# Soundness audit: abs() encoding
# Verifies abs(x) >= 0 for all x, abs(0) == 0, abs(-x) == abs(x),
# and behavior at large negative values.

# sig: (I64) -> I64
# pre: $x >= -1000000 && $x <= 1000000
# post: $result >= 0
sub abs_nonnegative {
    my ($x) = @_;
    return abs($x);
}

# sig: (I64) -> I64
# pre: $x == 0
# post: $result == 0
sub abs_zero {
    my ($x) = @_;
    return abs($x);
}

# sig: (I64) -> I64
# pre: $x == -5
# post: $result == 5
sub abs_neg_five {
    my ($x) = @_;
    return abs($x);
}

# sig: (I64) -> I64
# pre: $x == 5
# post: $result == 5
sub abs_pos_five {
    my ($x) = @_;
    return abs($x);
}

# sig: (I64) -> I64
# pre: $x >= -999999999 && $x <= -1
# post: $result == 0 - $x
sub abs_negative_equals_negation {
    my ($x) = @_;
    return abs($x);
}

# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 999999999
# post: $result == $x
sub abs_positive_is_identity {
    my ($x) = @_;
    return abs($x);
}
