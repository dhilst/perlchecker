# Probe 14: CONTROL — right shift of negative number (should be SOUND)
# Both BV64 bvlshr and Perl use LOGICAL (unsigned) right shift
# -8 >> 1 = 9223372036854775804 in BOTH Perl and BV64
# Expected: SOUND (both agree: result > 0)

# sig: (I64) -> I64
# pre: $x == -8
# post: $result > 0
sub probe_shr_negative_sound {
    my ($x) = @_;
    my $result = $x >> 1;
    return $result;
}
