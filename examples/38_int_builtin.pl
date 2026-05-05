# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 99
# post: $result == $n
sub int_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    my $r = int($s);
    return $r;
}
