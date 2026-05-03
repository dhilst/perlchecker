# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == $x
sub identity_with_pre {
    my ($x) = @_;
    return $x;
}
