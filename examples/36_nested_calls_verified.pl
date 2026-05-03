# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 2
sub nested_calls {
    my ($x) = @_;
    return inc(inc($x));
}
