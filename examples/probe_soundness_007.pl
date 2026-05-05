# Probe 7: Bitwise NOT UV promotion — ~0 < 0
# BV64: bvnot(0) = -1 (all-ones pattern, interpreted as signed = -1, negative)
# Perl:  ~0 = 18446744073709551615 (UV::MAX, all-ones interpreted as unsigned, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 0
# post: $result < 0
sub probe_bitnot_uv {
    my ($x) = @_;
    my $result = ~$x;
    return $result;
}
