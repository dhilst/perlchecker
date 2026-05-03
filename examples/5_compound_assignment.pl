# sig: (Int, Int) -> Int
# pre: $n >= 0 && $n <= 5 && $step > 0 && $step <= 10
# post: $result == $n * $step
sub mul_by_add {
    my ($n, $step) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $n; $i += 1) {
        $acc += $step;
    }
    return $acc;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 100 && $b >= 0 && $b <= 100
# post: $result == $a + $b
sub sum_compound {
    my ($a, $b) = @_;
    my $r = $a;
    $r += $b;
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 50 && $y >= 0 && $y <= 50
# post: $result == $x - $y
sub diff_compound {
    my ($x, $y) = @_;
    my $r = $x;
    $r -= $y;
    return $r;
}
