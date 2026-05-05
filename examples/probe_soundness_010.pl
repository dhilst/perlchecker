# Probe 10: chr() clamping above 196607 — ord(chr(200000)) == 65533
# BV/SMT: chr() is encoded to clamp values > 196607 to replacement char U+FFFD (65533)
#          so ord(chr(200000)) encodes as ord(chr(65533)) = 65533 -> post TRUE
# Perl:  chr(200000) is actually U+30D40; ord(chr(200000)) = 200000 != 65533
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 200000
# post: $result == 65533
sub probe_chr_clamping {
    my ($x) = @_;
    my $c = chr($x);
    my $result = ord($c);
    return $result;
}
