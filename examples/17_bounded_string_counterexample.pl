# sig: (Str) -> Int
# post: $result >= 0
sub bounded_string_counterexample {
    my ($x) = @_;
    return index($x, "z");
}
