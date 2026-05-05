# Probe 11: Large exponentiation overflow — 3**40 < 0
# BV64: 3^40 = 12157665459056928801 which as i64 (signed) is negative (> 2^63)
#        12157665459056928801 - 2^64 = -6289078614652622815 (negative)
# Perl:  3**40 = 1.21576654590569e+19 (NV float, positive)
# Expected: UNSOUND (checker verifies, Perl disagrees)

# sig: (I64) -> I64
# pre: $x == 3
# post: $result < 0
sub probe_pow_large {
    my ($x) = @_;
    my $result = $x ** 40;
    return $result;
}
