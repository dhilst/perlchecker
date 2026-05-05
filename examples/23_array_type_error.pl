# sig: (Array<I64>, Str) -> I64
# post: $result >= 0
sub array_type_error {
    my ($arr, $k) = @_;
    return $arr[$k];
}
