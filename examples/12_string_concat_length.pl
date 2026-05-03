# sig: (Str, Str) -> Str
# post: length($result) == length($x) + length($y)
sub string_concat_length {
    my ($x, $y) = @_;
    return $x . $y;
}
