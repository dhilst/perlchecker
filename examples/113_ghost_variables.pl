# Ghost variables: verification-only state for richer annotations

# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result == $x + $y
sub sum_with_ghost {
    my ($x, $y) = @_;
    # ghost: $expected = $x + $y
    my $sum = $x + $y;
    # assert: $sum == $expected
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= $n
sub double_ghost {
    my ($n) = @_;
    # ghost: $original = $n
    my $result = $n * 2;
    # assert: $result >= $original
    return $result;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 0
sub ghost_tracks_intermediate {
    my ($a, $b) = @_;
    my $x = $a + 1;
    # ghost: $snapshot = $x
    my $y = $x + $b;
    # assert: $y >= $snapshot
    return $y;
}
