# sig: (Str, Str) -> Str
# post: $result eq substr($x, 0, length($y))
sub substr_prefix_verified {
    my ($x, $y) = @_;
    return substr($x, 0, length($y));
}
