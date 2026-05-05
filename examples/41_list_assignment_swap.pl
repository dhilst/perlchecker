# sig: (I64, I64) -> I64
# pre: $a >= 0 && $b >= 0
# post: $result == $a
sub swap_returns_first {
    my ($a, $b) = @_;
    my ($x, $y) = ($a, $b);
    ($x, $y) = ($y, $x);
    return $y;
}

# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result == $a + $b
sub list_assign_sum {
    my ($a, $b) = @_;
    my ($x, $y) = ($a, $b);
    my $r = $x + $y;
    return $r;
}
