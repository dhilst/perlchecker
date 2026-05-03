# sig: (Int) -> Int
# post: $result == $x
sub for_bound_exhausted {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i >= 0; $i = $i + 1) {
        $x = $x + 1;
    }
    return $x;
}
