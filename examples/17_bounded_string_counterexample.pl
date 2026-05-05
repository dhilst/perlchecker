# sig: (Str) -> I64
# post: $result >= 0
sub bounded_string_counterexample {
    my ($x) = @_;
    return index($x, "z");
}
