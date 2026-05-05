# sig: (Array<I64>) -> I64
# pre: scalar(@arr) > 0
# post: $result > 0
sub scalar_array_in_condition {
    my ($arr) = @_;
    if (scalar(@arr) > 0) {
        return scalar(@arr);
    }
    return 0;
}
