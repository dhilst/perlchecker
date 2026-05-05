# Round 32: Implicit int-to-string coercion in concatenation

# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 99
# post: $result >= 1 && $result <= 2
sub digit_count {
    my ($n) = @_;
    my $s = "" . $n;
    my $len = length($s);
    return $len;
}

# sig: (I64) -> I64
# pre: $n >= 10 && $n <= 99
# post: $result == 2
sub two_digit_str_len {
    my ($n) = @_;
    my $s = "" . $n;
    my $len = length($s);
    return $len;
}

# sig: (Str, I64) -> Str
# pre: $count >= 1 && $count <= 5
# post: length($result) >= length($prefix) + 1
sub label_with_number {
    my ($prefix, $count) = @_;
    my $result = $prefix . $count;
    return $result;
}
