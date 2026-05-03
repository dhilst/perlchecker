# sig: (Str) -> Int
# post: $result == 0
sub index_verified {
    my ($x) = @_;
    return index($x, $x);
}
