# sig: (I64) -> I64
# pre: $x >= 10 && $x <= 50
# post: $result >= 0 && $result < 10
sub reduce_until {
    my ($x) = @_;
    my $r = $x;
    do {
        $r -= 10;
    } until ($r < 10);
    return $r;
}

# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 5
# post: $result == $n * 2
sub double_until {
    my ($n) = @_;
    my $r = 0;
    my $i = 0;
    do {
        $r += 2;
        $i++;
    } until ($i >= $n);
    return $r;
}
