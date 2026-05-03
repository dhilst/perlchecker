# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 100
# post: $result == $n * 5
sub multiply_by_five {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum == $i * 5 && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + 5;
        $i = $i + 1;
    }
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 50
# post: $result >= $n
sub multiply_with_invariant {
    my ($n) = @_;
    my $result = 0;
    my $i = 0;
    # inv: $result == $i * 3 && $i >= 0 && $i <= $n
    while ($i < $n) {
        $result = $result + 3;
        $i = $i + 1;
    }
    return $result;
}
