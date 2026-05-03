# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# pre: $x >= 0
# post: $result >= 1
sub call_in_condition {
    my ($x) = @_;
    if (inc($x) > 5) {
        return inc($x);
    }
    return 1;
}
