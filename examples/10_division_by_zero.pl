# sig: (Int) -> Int
# post: $result == $x
sub division_by_zero {
    my ($x) = @_;
    return $x / 0;
}
