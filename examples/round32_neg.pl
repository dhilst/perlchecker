# Round 32: Test negative int-to-string coercion

# sig: (Int) -> Int
# pre: $n >= -99 && $n <= -10
# post: $result == 3
sub negative_two_digit_len {
    my ($n) = @_;
    my $s = "" . $n;
    my $len = length($s);
    return $len;
}

# sig: (Int) -> Str
# pre: $n >= 0 && $n <= 9
# post: length($result) == 5
sub int_prefix_concat {
    my ($n) = @_;
    my $result = $n . "test";
    return $result;
}
