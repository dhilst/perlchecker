# sig: (I64) -> I64
# post: $result >= $x
sub recursion_rejected {
    my ($x) = @_;
    return recursion_rejected($x);
}
