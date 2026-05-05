# Soundness audit: BitNot uses unsigned 64-bit interpretation.
# Perl's ~ operator does bitwise NOT on a 64-bit unsigned value.
# For x >= 0: ~x == 2^64 - 1 - x (a large positive number).
# For x < 0 (two's complement): ~x == -x - 1 (a small non-negative value).

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 100
# post: $result >= 0
sub bitnot_nonneg_unsigned {
    my ($x) = @_;
    return ~$x;
}

# sig: (I64) -> I64
# pre: $x >= -100 && $x <= -1
# post: $result == -$x - 1
sub bitnot_neg {
    my ($x) = @_;
    return ~$x;
}

# sig: (I64) -> I64
# pre: $x == 0
# post: $result > 0
sub bitnot_zero {
    my ($x) = @_;
    return ~$x;
}

# sig: (I64) -> I64
# pre: $x == 5
# post: $result > 0
sub bitnot_five {
    my ($x) = @_;
    return ~$x;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result == 255 - $x
sub bitnot_masked_byte {
    my ($x) = @_;
    return (~$x) & 255;
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
    bitnot_nonneg_unsigned($x) >= 0;
}, name => "bitnot_nonneg: ~x >= 0 (unsigned)";

Property {
    ##[ x <- Int(range=>[0,255], sized=>0) ]##
    bitnot_masked_byte($x) == 255 - $x;
}, name => "bitnot_masked_byte: (~x)&0xFF == 255-x";
