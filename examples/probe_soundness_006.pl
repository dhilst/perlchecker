# Probe 6: abs() overflow at i64::MIN — abs(i64::MIN) < 0
# BV64: abs encodes as bvneg when x < 0; bvneg(i64::MIN) = i64::MIN (overflow, stays negative)
# Perl:  abs(-9223372036854775808) = 9223372036854775808 (UV, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 0 - 9223372036854775807 - 1
# post: $result < 0
sub probe_abs_overflow {
    my ($x) = @_;
    my $result = abs($x);
    return $result;
}
