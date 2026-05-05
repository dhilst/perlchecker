# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 100
# post: $result >= 2
sub shift_left_one {
    my ($x) = @_;
    my $r = $x << 1;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 4 && $x <= 100
# post: $result >= 1
sub shift_right_two {
    my ($x) = @_;
    my $r = $x >> 2;
    return $r;
}

# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 4
# post: $result >= 1
sub power_of_two {
    my ($n) = @_;
    my $r = 1 << $n;
    return $r;
}
