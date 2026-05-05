# Round 105: 64-bit bitvectors for bitwise operations

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result >= 0
sub shift_left_large {
    my ($x) = @_;
    my $result = $x << 33;
    return $result;
}

# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0 && $result <= 255
sub bitwise_and {
    my ($a, $b) = @_;
    return $a & $b;
}

# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0
sub bitwise_or {
    my ($a, $b) = @_;
    return $a | $b;
}
