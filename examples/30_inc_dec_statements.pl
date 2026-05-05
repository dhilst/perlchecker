# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 5
# post: $result == $n
sub count_with_inc {
    my ($n) = @_;
    my $i = 0;
    my $count = 0;
    while ($i < $n) {
        $count++;
        $i++;
    }
    return $count;
}

# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 5
# post: $result == 0
sub dec_to_zero {
    my ($n) = @_;
    my $r = $n;
    while ($r > 0) {
        $r--;
    }
    return $r;
}

# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 5
# post: $result == $n
sub for_with_inc {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        $sum++;
    }
    return $sum;
}
