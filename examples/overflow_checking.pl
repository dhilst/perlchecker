# Overflow checking with $overflow special variable
#
# Without "# assert: !$overflow", arithmetic wraps as BV64 two's complement.
# With the assertion, the checker proves no overflow occurs or provides
# a counterexample.

# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 100 && $y >= 0 && $y <= 100
# post: $result >= 0 && $result <= 200
sub safe_bounded_add {
    my ($x, $y) = @_;
    my $sum = $x + $y;
    # assert: !$overflow
    return $sum;
}

# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 10 && $y >= 1 && $y <= 10
# post: $result >= 1 && $result <= 100
sub safe_bounded_mul {
    my ($x, $y) = @_;
    my $product = $x * $y;
    # assert: !$overflow
    return $product;
}

# sig: (I64) -> I64
# pre: $x >= -1000000 && $x <= 1000000
# post: $result >= 0
sub safe_abs {
    my ($x) = @_;
    my $r = abs($x);
    # assert: !$overflow
    return $r;
}
