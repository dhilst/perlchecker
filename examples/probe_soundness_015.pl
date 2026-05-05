# Probe 15: CONTROL — Perl modulo with negative dividend (should be SOUND)
# Both Perl % and Z3 bvsmod use floor-mod (result sign follows divisor)
# -7 % 3 = 2 in BOTH Perl and BV64
# Expected: SOUND (both agree)

# sig: (I64, I64) -> I64
# pre: $x == -7 && $y == 3
# post: $result == 2
sub probe_modulo_negative_sound {
    my ($x, $y) = @_;
    my $result = $x % $y;
    return $result;
}
