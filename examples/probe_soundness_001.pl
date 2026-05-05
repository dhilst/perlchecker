# Probe 1: Multiplication overflow — x * 2 < 0 with x = 2^62
# BV64: 4611686018427387904 * 2 = i64::MIN (sign overflow, negative)
# Perl:  4611686018427387904 * 2 = 9223372036854775808 (UV, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 4611686018427387904
# post: $result < 0
sub probe_mul_overflow {
    my ($x) = @_;
    my $result = $x * 2;
    return $result;
}
