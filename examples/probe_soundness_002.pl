# Probe 2: Exponentiation overflow — 2**63 < 0
# BV64: pow encodes via Z3 Real then back to BV; 2^63 = i64::MIN in BV (sign bit)
# Perl:  2**63 = 9.22337203685478e+18 (NV float, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 2
# post: $result < 0
sub probe_pow_overflow {
    my ($x) = @_;
    my $result = $x ** 63;
    return $result;
}
