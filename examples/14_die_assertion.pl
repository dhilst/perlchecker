# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 100
# post: $result >= 1
sub guarded_positive {
    my ($x) = @_;
    if ($x <= 0) {
        die "must be positive";
    }
    return $x;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 10
# post: $result >= 0
sub die_is_reachable {
    my ($x) = @_;
    if ($x == 0) {
        die "zero!";
    }
    return $x;
}
