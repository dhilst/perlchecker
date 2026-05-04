# Soundness audit: bitwise AND/OR/XOR with negative inputs
# Perl bitwise ops produce unsigned results when MSB is set.
# (-1) & 5 == 5 (MSB=0, treated as signed IV)
# (-1) | 0 == 18446744073709551615 (MSB=1, treated as unsigned UV)
# (-1) ^ (-1) == 0 (MSB=0, treated as signed IV)

# sig: (Int, Int) -> Int
# pre: $x == -1 && $y == 5
# post: $result == 5
sub bitand_neg1_five {
    my ($x, $y) = @_;
    my $r = $x & $y;
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $x == -1 && $y == -1
# post: $result == 0
sub bitxor_neg1_neg1 {
    my ($x, $y) = @_;
    my $r = $x ^ $y;
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $x == -3 && $y == 7
# post: $result == 5
sub bitand_neg3_seven {
    my ($x, $y) = @_;
    my $r = $x & $y;
    return $r;
}

# Perl produces unsigned (UV) for bitwise results with MSB=1.
# (-1) | 0 gives UV 18446744073709551615, which is >= 0.
# sig: (Int, Int) -> Int
# pre: $x == -1 && $y == 0
# post: $result >= 0
sub bitor_neg1_zero_unsigned {
    my ($x, $y) = @_;
    my $r = $x | $y;
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $x == -3 && $y == 4
# post: $result >= 0
sub bitor_neg3_four_unsigned {
    my ($x, $y) = @_;
    my $r = $x | $y;
    return $r;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[0,255], sized=>0), y <- Int(range=>[0,255], sized=>0) ]##
    ((-1) & $y) == $y;
}, name => "bitand -1 with non-negative == identity";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0) ]##
    ($x ^ $x) == 0;
}, name => "bitxor self == 0";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[0,100], sized=>0) ]##
    my $r = $x | $y;
    $r >= 0;
}, name => "bitor any with non-neg produces non-neg (UV)";
