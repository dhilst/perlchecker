# sig: (Str, Str) -> Int
# pre: length($s) >= 3 && length($sub) == 1
# post: $result >= 0 && $result <= 1
sub check_contains {
    my ($s, $sub) = @_;
    my $r = contains($s, $sub);
    return $r;
}

# sig: (Str) -> Int
# pre: length($s) >= 5
# post: $result == 1
sub self_contains {
    my ($s) = @_;
    my $r = contains($s, $s);
    return $r;
}
