# sig: (Array<I64>) -> I64
# post: $result > 0
sub scalar_array_counterexample {
    my ($arr) = @_;
    return scalar(@arr);
}
