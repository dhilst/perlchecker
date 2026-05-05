# sig: (I64) -> I64
# post: $result == 0
sub modulo_by_zero_invalid {
    my ($x) = @_;
    return $x % 0;
}
