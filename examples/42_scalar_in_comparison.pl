# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n
# post: $result == $n
sub scalar_in_comparison {
    my ($arr, $n) = @_;
    if (scalar(@arr) == $n) {
        return $n;
    }
    return 0;
}
