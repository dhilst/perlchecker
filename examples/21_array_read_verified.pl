# sig: (Array<Int>, Int) -> Int
# post: $result == $arr[$i]
sub array_read_verified {
    my ($arr, $i) = @_;
    return $arr[$i];
}
