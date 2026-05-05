# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 1
sub both_positive_or_default {
    my ($x, $y) = @_;
    my $r = 1;
    if ($x > 0 and $y > 0) {
        $r = $x + $y;
    }
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result >= 0
sub guard_or {
    my ($x) = @_;
    my $r = $x;
    if (not ($x >= 0)) {
        $r = 0 - $x;
    }
    return $r;
}

# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 5 && $y >= 0 && $y <= 5
# post: $result >= 0
sub low_or_example {
    my ($x, $y) = @_;
    my $r = 0;
    if ($x > 0 or $y > 0) {
        $r = $x + $y;
    }
    return $r;
}
