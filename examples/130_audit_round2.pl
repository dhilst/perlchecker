# Audit Round 2: right shift on negative numbers
#
# Perl's >> is a LOGICAL right shift — it shifts in zero bits from
# the left, so (-8 >> 1) produces a large positive number
# (9223372036854775804), not -4.
#
# The checker previously used Z3's bvashr (arithmetic shift right)
# which preserves the sign bit.  Fixed to use bvlshr (logical).

# The postcondition ($result >= 0) is always true for logical right
# shift of any non-negative shift amount.  Previously the checker
# would find a spurious counterexample because arithmetic shift of
# a negative number stays negative.
# sig: (Int) -> Int
# pre: $x >= -1000 && $x <= 1000
# post: $result >= 0
sub logical_shr_nonneg {
    my ($x) = @_;
    return $x >> 1;
}

# Positive inputs: logical and arithmetic right shift agree.
# sig: (Int) -> Int
# pre: $x >= 8 && $x <= 1000
# post: $result >= 4
sub positive_shr_verified {
    my ($x) = @_;
    return $x >> 1;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[-1000,1000], sized=>0) ]##
    my $result = $x >> 1;
    $result >= 0;
}, name => "logical_shr_nonneg: post holds";

Property {
    ##[ x <- Int(range=>[8,1000], sized=>0) ]##
    my $result = $x >> 1;
    $result >= 4;
}, name => "positive_shr_verified: post holds";
