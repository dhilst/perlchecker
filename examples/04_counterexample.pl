# sig: (Int) -> Int
# post: $result > $x
sub counterexample {
    my ($x) = @_;
    if ($x >= 0) {
        return $x;
    } else {
        return $x + 1;
    }
}
