# Audit Round 1: Modulo sign semantics — fixed
#
# Before fix: Z3 rem() was used for Perl's %, giving wrong results
# when the divisor is negative. Now encode_perl_modulo() correctly
# implements floor-modulo semantics matching Perl.

# Test: modulo with negative divisor, result follows divisor sign
# sig: (I64, I64) -> I64
# pre: $y == -3 && $x == 7
# post: $result == 5
sub mod_add_fixed {
    my ($x, $y) = @_;
    return $x + ($x % $y);
}

# Test: modulo with positive divisor, negative dividend
# sig: (I64, I64) -> I64
# pre: $y == 3 && $x == -7
# post: $result == -5
sub mod_add_neg_dividend {
    my ($x, $y) = @_;
    return $x + ($x % $y);
}

# Test: general property — when divisor > 0, result of % is non-negative
# sig: (I64, I64) -> I64
# pre: $y > 0 && $y < 100
# post: $result >= 0 && $result < $y
sub mod_nonneg_when_positive_divisor {
    my ($x, $y) = @_;
    return $x % $y;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ ]##
    my $result = mod_add_fixed(7, -3);
    $result == 5;
}, name => "mod_add_fixed: 7 + (7 % -3) == 7 + (-2) == 5";

Property {
    ##[ ]##
    my $result = mod_add_neg_dividend(-7, 3);
    $result == -5;
}, name => "mod_add_neg_dividend: -7 + (-7 % 3) == -7 + 2 == -5";

Property {
    ##[ x <- Int(range=>[-1000,1000], sized=>0), y <- Int(range=>[1,100], sized=>0) ]##
    my $result = mod_nonneg_when_positive_divisor($x, $y);
    $result >= 0 && $result < $y;
}, name => "mod_nonneg_when_positive_divisor: x % y in [0, y) when y > 0";
