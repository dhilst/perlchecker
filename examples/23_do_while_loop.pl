# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 5
# post: $result == $n
sub count_do_while {
    my ($n) = @_;
    my $i = 0;
    do {
        $i += 1;
    } while ($i < $n);
    return $i;
}

# sig: (I64) -> I64
# pre: $x >= 10 && $x <= 20
# post: $result >= 0 && $result < 10
sub reduce_below_ten {
    my ($x) = @_;
    my $r = $x;
    do {
        $r -= 5;
    } while ($r >= 10);
    return $r;
}
