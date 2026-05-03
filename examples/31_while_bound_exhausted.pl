# sig: (Int) -> Int
# post: $result == 0
sub while_bound_exhausted {
    my ($x) = @_;
    while ($x >= 0) {
        $x = $x + 1;
    }
    return 0;
}
