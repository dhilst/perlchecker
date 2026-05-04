# Audit: left-shift with large shift amounts / overflow into BV64
#
# Bug found: When arithmetic produces a value >= 2^64 (e.g. (1<<63)+(1<<63)),
# Z3's int2bv wraps mod 2^64 (giving 0), but Perl's NV-to-UV conversion
# saturates at UINT64_MAX (0xFFFFFFFFFFFFFFFF). This caused the verifier to
# falsely accept postconditions like "$result == 0" when Perl gives 1.
#
# Fix applied:
# 1. Shift-amount guard: if abs(shift_count) >= 64, result is forced to 0
#    (prevents int2bv wrap for huge shift amounts >= 2^64).
# 2. Safety constraint: left operand of shifts must be < 2^64, otherwise
#    the path is discarded as invalid (sound/conservative: we reject rather
#    than accept a potentially wrong proof).

# --- Case 1: shift by 64 always yields 0 (sound before and after fix) ---
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 10
# post: $result == 0
sub shl_by_64 {
    my ($x) = @_;
    my $r = $x << 64;
    return $r;
}

# --- Case 2: large variable shift amount yields 0 ---
# sig: (Int) -> Int
# pre: $n >= 64 && $n <= 200
# post: $result == 0
sub shl_large_var_amount {
    my ($n) = @_;
    my $r = 42 << $n;
    return $r;
}

# --- Case 3: basic shift correctness (regression guard) ---
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 10
# post: $result == $x * 2
sub shl_one_is_times_two {
    my ($x) = @_;
    my $r = $x << 1;
    return $r;
}

# --- Case 4: shift by 63 for small values ---
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 1
# post: $result > 0
sub shl_63_positive {
    my ($x) = @_;
    my $r = $x << 63;
    return $r;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[1,10], sized=>0) ]##
    ($x << 64) == 0;
}, name => "shl_by_64: post holds";

Property {
    ##[ ]##
    my $x = 1;
    my $a = $x << 63;
    my $b = $a + $a;
    my $r = $b >> 63;
    $r == 1;
}, name => "shl_overflow_then_shr: Perl gives 1 (overflow saturates to UINT64_MAX)";

Property {
    ##[ x <- Int(range=>[1,10], sized=>0) ]##
    ($x << 1) == $x * 2;
}, name => "shl_one_is_times_two: post holds";

Property {
    ##[ x <- Int(range=>[1,100], sized=>0) ]##
    ($x << 100) == 0;
}, name => "shl_by_100: post holds";
