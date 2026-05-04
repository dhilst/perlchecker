# Soundness audit: BitNot must use signed BV64 interpretation.
# ~x == -x - 1 is the two's-complement algebraic identity.

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result == -$x - 1
sub bitnot_nonneg {
    my ($x) = @_;
    return ~$x;
}

# sig: (Int) -> Int
# pre: $x >= -100 && $x <= -1
# post: $result == -$x - 1
sub bitnot_neg {
    my ($x) = @_;
    return ~$x;
}

# sig: (Int) -> Int
# pre: $x == 0
# post: $result == -1
sub bitnot_zero {
    my ($x) = @_;
    return ~$x;
}

# sig: (Int) -> Int
# pre: $x == 5
# post: $result == -6
sub bitnot_five {
    my ($x) = @_;
    return ~$x;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[-100,-1], sized=>0) ]##
    use integer;
    bitnot_neg($x) == -$x - 1;
}, name => "bitnot_neg: ~x == -x-1 for x<0";

Property {
    ##[ x <- Int(range=>[0,100], sized=>0) ]##
    use integer;
    bitnot_nonneg($x) == -$x - 1;
}, name => "bitnot_nonneg: ~x == -x-1 for x>=0";
