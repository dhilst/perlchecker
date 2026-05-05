# sig: (Array<I64>, I64) -> I64
# post: $result == $arr[$i]
sub array_read_verified {
    my ($arr, $i) = @_;
    return $arr[$i];
}
