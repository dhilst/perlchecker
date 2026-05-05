# sig: (I64, I64) -> I64
# post: $result == 1
sub division_zero_path_discarded {
    my ($x, $y) = @_;
    if ($y == 0) {
        return $x / $y;
    }
    return 1;
}
