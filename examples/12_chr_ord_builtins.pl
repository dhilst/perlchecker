# sig: (Int) -> Int
# pre: $n >= 65 && $n <= 90
# post: $result >= 97 && $result <= 122
sub to_lower_code {
    my ($n) = @_;
    my $r = $n + 32;
    return $r;
}

# sig: (Int) -> Int
# pre: $n >= 65 && $n <= 90
# post: $result == $n + 32
sub ord_chr_roundtrip {
    my ($n) = @_;
    my $c = chr($n);
    my $r = ord($c);
    my $lower = $r + 32;
    return $lower;
}
