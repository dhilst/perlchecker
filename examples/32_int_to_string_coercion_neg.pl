# Round 32: Test negative int-to-string coercion

# sig: (I64) -> I64
# pre: $n >= -99 && $n <= -10
# post: $result == 3
sub negative_two_digit_len {
    my ($n) = @_;
    my $s = "" . $n;
    my $len = length($s);
    return $len;
}

# sig: (I64) -> Str
# pre: $n >= 0 && $n <= 9
# post: length($result) == 5
sub int_prefix_concat {
    my ($n) = @_;
    my $result = $n . "test";
    return $result;
}
