# Probe 4: Subtraction underflow — i64::MIN - 1 > 0
# BV64: -2^63 - 1 = i64::MAX (wraps to positive)
# Perl:  -9223372036854775808 - 1 = -9.22e+19 (NV float, more negative)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 0 - 9223372036854775807 - 1
# post: $result > 0
sub probe_sub_underflow {
    my ($x) = @_;
    my $result = $x - 1;
    return $result;
}
