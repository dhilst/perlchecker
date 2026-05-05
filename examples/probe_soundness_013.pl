# Probe 13: Bitwise NOT for any non-negative input — ~x < 0 with x >= 0
# BV64: bvnot(x) = -x - 1; for x >= 0: -x - 1 <= -1 < 0 (always negative in signed BV)
# Perl:  ~x for x >= 0 gives UV::MAX - x (unsigned, always positive for x <= UV::MAX)
# Expected: UNSOUND (checker verifies universally, Perl disagrees for all x >= 0)

# sig: (I64) -> I64
# pre: $x >= 0
# post: $result < 0
sub probe_bitnot_all_nonneg {
    my ($x) = @_;
    my $result = ~$x;
    return $result;
}
