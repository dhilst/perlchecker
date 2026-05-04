# sig: (Array<Int>) -> Int
# pre: scalar(@arr) == 3 && $arr[0] == 10 && $arr[1] == 20 && $arr[2] == 30
# post: $result == 30
sub last_elem {
    my ($arr) = @_;
    return $arr[-1];
}

# sig: (Array<Int>) -> Int
# pre: scalar(@arr) == 3 && $arr[0] == 10 && $arr[1] == 20 && $arr[2] == 30
# post: $result == 20
sub second_to_last {
    my ($arr) = @_;
    return $arr[-2];
}

# sig: (Array<Int>) -> Int
# pre: scalar(@arr) == 4 && $arr[0] == 1 && $arr[1] == 2 && $arr[2] == 3 && $arr[3] == 4
# post: $result == 5
sub neg_write_then_read {
    my ($arr) = @_;
    $arr[-1] = 5;
    return $arr[3];
}
