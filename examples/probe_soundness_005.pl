# Probe 5: Negation overflow — -i64::MIN < 0
# BV64: bvneg(-2^63) = -2^63 (overflow, stays i64::MIN, negative)
# Perl:  -(-9223372036854775808) = 9223372036854775808 (UV, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 0 - 9223372036854775807 - 1
# post: $result < 0
sub probe_negate_overflow {
    my ($x) = @_;
    my $result = -$x;
    return $result;
}
