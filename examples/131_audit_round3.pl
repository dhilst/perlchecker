# Audit Round 3: Left/right-shift with negative shift amount
#
# Perl treats negative shift amounts by reversing the direction:
#   x << -n  is equivalent to  x >> n
#   x >> -n  is equivalent to  x << n
#
# Previously the checker's BV encoding converted -n to unsigned (2^64 - n),
# causing bvshl to shift by a huge amount and yield 0 instead of the
# correct reversed-direction result. Now fixed.

# --- left-shift with negative amount equals right-shift ---
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 1000 && $n >= 1 && $n <= 5
# post: $result == ($x >> $n)
sub shl_neg_equals_shr {
    my ($x, $n) = @_;
    my $r = $x << (-$n);
    return $r;
}

# --- right-shift with negative amount equals left-shift ---
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 100 && $n >= 1 && $n <= 3
# post: $result == ($x << $n)
sub shr_neg_equals_shl {
    my ($x, $n) = @_;
    my $r = $x >> (-$n);
    return $r;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[0,1000], sized=>0), n <- Int(range=>[1,5], sized=>0) ]##
    my $result = shl_neg_equals_shr($x, $n);
    $result == ($x >> $n);
}, name => "shl_neg_equals_shr: post holds";

Property {
    ##[ x <- Int(range=>[0,100], sized=>0), n <- Int(range=>[1,3], sized=>0) ]##
    my $result = shr_neg_equals_shl($x, $n);
    $result == ($x << $n);
}, name => "shr_neg_equals_shl: post holds";
