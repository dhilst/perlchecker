# Probe 12: i64::MAX squared wraps to 1 in BV64
# BV64: (2^63 - 1)^2 = 2^126 - 2*2^63 + 1 ≡ 0 - 0 + 1 = 1 (mod 2^64)
#        So i64::MAX * i64::MAX == 1 in BV64
# Perl:  9223372036854775807 * 9223372036854775807 = 8.50705917302346e+37 (NV float) != 1
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 9223372036854775807
# post: $result == 1
sub probe_max_squared {
    my ($x) = @_;
    my $result = $x * $x;
    return $result;
}
