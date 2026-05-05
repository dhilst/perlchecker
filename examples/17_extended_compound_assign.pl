# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 100 && $n >= 0 && $n <= 3
# post: $result >= 1
sub shift_assign {
    my ($x, $n) = @_;
    my $r = $x;
    $r <<= $n;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result >= 0 && $result <= 15
sub mask_low_nibble {
    my ($x) = @_;
    my $r = $x;
    $r &= 15;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 2 && $x <= 5
# post: $result >= 4
sub square_assign {
    my ($x) = @_;
    my $r = $x;
    $r **= 2;
    return $r;
}
