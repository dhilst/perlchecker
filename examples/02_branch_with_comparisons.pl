# sig: (Int) -> Int
# post: $result >= 0
sub branch_with_comparisons {
    my ($x) = @_;
    if ($x < 0) {
        return 0 - $x;
    } else {
        return $x;
    }
}
