# sig: (I64) -> I64
# post: $result == $x
sub division_by_zero {
    my ($x) = @_;
    return $x / 0;
}
