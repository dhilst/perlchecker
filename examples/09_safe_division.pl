# sig: (I64, I64) -> I64
# pre: $y != 0
# post: $result == $x / $y
sub safe_division {
    my ($x, $y) = @_;
    return $x / $y;
}
