# Soundness audit: Array out-of-bounds read returns 0 (undef in numeric context).
#
# In Perl, $arr[N] where N >= scalar(@arr) returns undef, which is 0 in numeric
# context. Before this fix, the checker used Z3 Arrays (functional maps) where
# reading an unwritten index returned an arbitrary value -- producing spurious
# counterexamples for correct programs.
#
# This example demonstrates the fix: reading $arr[10] from a 3-element parameter
# array returns 0 (the Perl-correct undef value). The branch ($oob > 0) is never
# taken, so $result is always 0.

# sig: (Array<I64>) -> I64
# pre: scalar(@arr) == 3
# post: $result == 0
sub oob_read_returns_zero {
    my ($arr) = @_;
    my $oob = $arr[10];
    if ($oob > 0) {
        return 1;
    }
    return 0;
}
