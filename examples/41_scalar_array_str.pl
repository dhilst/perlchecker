# sig: (Array<Str>) -> I64
# pre: scalar(@arr) > 0
# post: $result == scalar(@arr)
sub scalar_array_str {
    my ($arr) = @_;
    return scalar(@arr);
}
