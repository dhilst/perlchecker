# sig: (Str) -> Int
# pre: length($s) == 2
# post: $result == 6
sub repeat_three_len {
    my ($s) = @_;
    my $r = $s x 3;
    my $len = length($r);
    return $len;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 3
# post: $result == length($s) * 2
sub repeat_two_len {
    my ($s) = @_;
    my $r = $s x 2;
    my $len = length($r);
    return $len;
}
