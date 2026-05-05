# sig: (Array<I64>) -> I64
# pre: scalar(@arr) > 0
# post: $result == scalar(@arr)
sub scalar_array_length_verified {
    my ($arr) = @_;
    return scalar(@arr);
}
