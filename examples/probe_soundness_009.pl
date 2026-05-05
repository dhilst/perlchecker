# Probe 9: Left shift by zero of negative — (-1) << 0 < 0
# BV64: bvshl(-1, 0) = -1 (shifting by 0 is identity, stays -1, negative)
# Perl:  -1 << 0 = 18446744073709551615 (UV, even shift-by-0 promotes negative to UV in Perl)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == -1
# post: $result < 0
sub probe_shl_zero_negative {
    my ($x) = @_;
    my $result = $x << 0;
    return $result;
}
