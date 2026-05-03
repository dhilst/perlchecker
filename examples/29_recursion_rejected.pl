# sig: (Int) -> Int
# post: $result >= $x
sub recursion_rejected {
    my ($x) = @_;
    return recursion_rejected($x);
}
