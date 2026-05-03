# sig: (Str) -> Str
# post: $result eq $x
sub string_identity {
    my ($x) = @_;
    return $x;
}
