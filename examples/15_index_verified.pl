# sig: (Str) -> I64
# post: $result == 0
sub index_verified {
    my ($x) = @_;
    return index($x, $x);
}
