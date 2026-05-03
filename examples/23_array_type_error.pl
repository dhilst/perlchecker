# sig: (Array<Int>, Str) -> Int
# post: $result >= 0
sub array_type_error {
    my ($arr, $k) = @_;
    return $arr[$k];
}
