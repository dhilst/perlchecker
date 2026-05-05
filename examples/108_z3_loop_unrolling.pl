# Function 1: Sum integers 0..n-1 using while loop.
# With the increased default unroll limit (9), this works for n up to 8
# whereas the old limit of 5 would fail for n > 5.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 8
# post: $result == $n * ($n - 1) / 2
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

# Function 2: Bounded factorial using while loop.
# The increased unroll limit allows verifying up to n=7.
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 7
# post: $result >= 1
sub factorial_bounded {
    my ($n) = @_;
    my $result = 1;
    my $i = 1;
    while ($i <= $n) {
        $result = $result * $i;
        $i = $i + 1;
    }
    return $result;
}

# Function 3: Count down from n to 0, verifying the loop ran to completion.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 8
# post: $result == 0
sub count_down {
    my ($n) = @_;
    my $i = $n;
    while ($i > 0) {
        $i = $i - 1;
    }
    return $i;
}
