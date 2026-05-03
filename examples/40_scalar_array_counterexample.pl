# sig: (Array<Int>) -> Int
# post: $result > 0
sub scalar_array_counterexample {
    my ($arr) = @_;
    return scalar(@arr);
}
