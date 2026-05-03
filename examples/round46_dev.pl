# sig: (Str, Str) -> Int
# pre: length($s) >= 3 && length($p) >= 1 && length($p) <= 3
# post: $result >= 0 && $result <= 1
sub check_prefix {
    my ($s, $p) = @_;
    my $r = starts_with($s, $p);
    return $r;
}

# sig: (Str) -> Int
# pre: length($s) >= 1
# post: $result == 1
sub string_starts_with_self {
    my ($s) = @_;
    my $r = starts_with($s, $s);
    return $r;
}

# sig: (Str) -> Int
# pre: length($s) >= 1
# post: $result == 1
sub string_ends_with_self {
    my ($s) = @_;
    my $r = ends_with($s, $s);
    return $r;
}
