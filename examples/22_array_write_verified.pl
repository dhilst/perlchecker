# sig: (Array<Int>, Int, Int) -> Int
# post: $result == $v
sub array_write_verified {
    my ($arr, $i, $v) = @_;
    $arr[$i] = $v;
    return $arr[$i];
}
