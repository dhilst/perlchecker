# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result == length($s)
sub reverse_preserves_length {
    my ($s) = @_;
    my $r = reverse($s);
    my $len = length($r);
    return $len;
}

# sig: (Str) -> Int
# pre: length($s) >= 2 && length($s) <= 5
# post: $result >= 2
sub reverse_at_least_two {
    my ($s) = @_;
    my $r = reverse($s);
    my $len = length($r);
    return $len;
}
