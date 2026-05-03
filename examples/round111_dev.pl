# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 0
sub sum_with_assert {
    my ($x, $y) = @_;
    my $sum = $x + $y;
    # assert: $sum >= 0
    # assert: $sum <= 20
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 100
# post: $result >= 0
sub double_with_assert {
    my ($n) = @_;
    my $result = $n * 2;
    # assert: $result >= 0
    return $result;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 1 && $b <= 10
# post: $result >= 0
sub guarded_division {
    my ($a, $b) = @_;
    # assert: $b >= 1
    my $result = $a / $b;
    return $result;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 5
# post: $result >= 0
sub loop_with_assert {
    my ($x) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum >= 0 && $i >= 0
    while ($i < $x) {
        $sum = $sum + $i;
        # assert: $sum >= 0
        $i = $i + 1;
    }
    return $sum;
}
