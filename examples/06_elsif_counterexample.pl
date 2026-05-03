# sig: (Int) -> Int
# post: $result > $x
sub elsif_counterexample {
    my ($x) = @_;
    if ($x < 0) {
        return 0 - $x;
    } elsif ($x == 0) {
        return 0;
    } else {
        return $x + 1;
    }
}
