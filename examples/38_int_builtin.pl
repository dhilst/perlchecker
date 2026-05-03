# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 99
# post: $result == $n
sub int_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    my $r = int($s);
    return $r;
}
