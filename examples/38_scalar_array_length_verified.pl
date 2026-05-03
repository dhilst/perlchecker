# sig: (Array<Int>) -> Int
# pre: scalar(@arr) > 0
# post: $result == scalar(@arr)
sub scalar_array_length_verified {
    my ($arr) = @_;
    return scalar(@arr);
}
