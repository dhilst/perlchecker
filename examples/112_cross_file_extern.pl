# Declare an external function's contract
# extern: external_abs (I64) -> I64 post: $result >= 0

# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result >= 0
sub use_external_abs {
    my ($x) = @_;
    my $result = external_abs($x);
    return $result;
}

# extern: clamp (I64, I64, I64) -> I64 pre: $b <= $c post: $result >= $b && $result <= $c

# sig: (I64) -> I64
# pre: $x >= -1000 && $x <= 1000
# post: $result >= 0 && $result <= 100
sub use_clamp {
    my ($x) = @_;
    return clamp($x, 0, 100);
}
