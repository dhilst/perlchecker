# sig: (Str) -> Str
# post: $result eq $x
sub substr_out_of_bounds {
    my ($x) = @_;
    return substr($x, 1, 1);
}
