# Probe 8: Left shift of negative number — (-8) << 1 < 0
# BV64: bvshl(-8, 1) = -16 (signed BV arithmetic, stays negative)
# Perl:  -8 << 1 = 18446744073709551600 (UV, Perl promotes to UV on left shift of negative)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == -8
# post: $result < 0
sub probe_shl_negative {
    my ($x) = @_;
    my $result = $x << 1;
    return $result;
}
