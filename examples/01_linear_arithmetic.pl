# sig: (I64, I64) -> I64
# pre: $x >= 0 && $y >= 0
# post: $result == $x + $y * 2
sub linear_arithmetic {
    my ($x, $y) = @_;
    my $z = $y * 2;
    return $x + $z;
}
