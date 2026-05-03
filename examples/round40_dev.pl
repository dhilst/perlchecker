# sig: (Str, Str) -> Int
# pre: length($s) >= 4 && length($sub) == 1
# post: $result >= -1
sub index_from_start {
    my ($s, $sub) = @_;
    my $r = index($s, $sub, 2);
    return $r;
}

# sig: (Str, Str) -> Int
# pre: length($s) >= 3 && length($sub) == 1
# post: $result >= -1 && $result <= length($s) - 1
sub bounded_index {
    my ($s, $sub) = @_;
    my $first = index($s, $sub);
    my $second = index($s, $sub, 1);
    return $second;
}
