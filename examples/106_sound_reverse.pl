# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub reverse_preserves_length {
    my ($s) = @_;
    my $r = reverse($s);
    if (length($r) == length($s)) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub double_reverse_identity {
    my ($s) = @_;
    my $r = reverse(reverse($s));
    if ($r eq $s) {
        return 1;
    }
    return 0;
}
