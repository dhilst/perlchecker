# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 5
# post: $result >= 0
sub sum_to_n {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    while ($i < $n) {
        $sum = $sum + $i;
        $i = $i + 1;
    }
    return $sum;
}

# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 5
# post: $result >= 0
sub for_loop_test {
    my ($x) = @_;
    my $result = 0;
    my $i = 0;
    for ($i = 0; $i < $x; $i = $i + 1) {
        $result = $result + $i;
    }
    return $result;
}

# sig: (Array<I64>) -> I64
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 4
# post: $result >= 0 && $result <= 4
sub foreach_still_works {
    my ($arr) = @_;
    my $count = 0;
    foreach my $x (@arr) {
        if ($x > 0) {
            $count = $count + 1;
        }
    }
    return $count;
}
