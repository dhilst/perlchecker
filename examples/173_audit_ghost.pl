# Ghost variable shadowing a real variable is now rejected as unsound.
# Previously, "# ghost: $x = 999" would overwrite $x in the verifier's
# model, making it verify false postconditions (Perl ignores the comment).

# This file tests that legitimate ghost usage still works after the fix.

# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result == $x + $y
sub sum_with_fresh_ghost {
    my ($x, $y) = @_;
    # ghost: $expected = $x + $y
    my $sum = $x + $y;
    # assert: $sum == $expected
    return $sum;
}
