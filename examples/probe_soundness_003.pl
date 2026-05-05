# Probe 3: Division overflow — int(i64::MIN / -1) < 0
# BV64: bvsdiv(-2^63, -1) = -2^63 (overflow, stays i64::MIN)
# Perl:  int(-9223372036854775808 / -1) = 9223372036854775808 (UV, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64, I64) -> I64
# pre: $x == 0 - 9223372036854775807 - 1 && $y == -1
# post: $result < 0
sub probe_div_overflow {
    my ($x, $y) = @_;
    my $result = int($x / $y);
    return $result;
}
