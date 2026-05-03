# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result == $x + 1
sub unless_basic {
    my ($x) = @_;
    my $r = $x;
    unless ($x == 0) {
        $r = $x + 1;
    } else {
        $r = 1;
    }
    return $r;
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 50
# post: $result >= 0
sub unless_no_else {
    my ($n) = @_;
    my $r = $n;
    unless ($n > 10) {
        $r = 0;
    }
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20
# post: $result == $a + $b
sub unless_desugar_equiv {
    my ($a, $b) = @_;
    my $r = 0;
    unless ($a == 0 && $b == 0) {
        $r = $a + $b;
    } else {
        $r = 0;
    }
    return $r;
}
