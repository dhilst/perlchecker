# Soundness audit: char_at($s, $i) negative-index behavior
#
# Z3's str.at(s, i) returns "" for i < 0, but Perl's substr($s, i, 1)
# wraps negative indices from the end of the string. Before the fix,
# char_at($s, -1) was unsoundly encoded as "" instead of the last char.

# Negative index -1 returns the last character (Perl wrap-around).
# sig: (Str) -> Str
# pre: $s eq "abc"
# post: $result eq "c"
sub char_at_neg_returns_last {
    my ($s) = @_;
    return char_at($s, -1);
}

# Negative index -2 returns the second-to-last character.
# sig: (Str) -> Str
# pre: $s eq "hello"
# post: $result eq "l"
sub char_at_neg2_returns_penultimate {
    my ($s) = @_;
    return char_at($s, -2);
}

# Positive OOB index returns "" (both Perl and Z3 agree).
# sig: (Str) -> Str
# pre: length($s) == 3
# post: $result eq ""
sub char_at_oob_is_empty {
    my ($s) = @_;
    return char_at($s, 100);
}

# Normal in-bounds access still works.
# sig: (Str) -> Str
# pre: $s eq "xyz"
# post: $result eq "y"
sub char_at_inbounds {
    my ($s) = @_;
    return char_at($s, 1);
}

# Length of char_at result is 1 for valid in-bounds access.
# sig: (Str, Int) -> Int
# pre: length($s) >= 5 && $i >= 0 && $i < length($s)
# post: $result == 1
sub char_at_length_one {
    my ($s, $i) = @_;
    my $c = char_at($s, $i);
    my $len = length($c);
    return $len;
}
