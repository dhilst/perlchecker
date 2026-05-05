use strict;
use warnings;
use Test::More;

sub prop_ok {
    my ($name, $body) = @_;
    for my $t (1..1000) {
        unless ($body->($t)) {
            fail("$name (falsified at trial $t)");
            return;
        }
    }
    pass("$name (1000 trials)");
}

sub randi { int(rand($_[1] - $_[0] + 1)) + $_[0] }

# Bitwise AND (&)
prop_ok("band: idempotent (x & x == x)", sub {
    my $x = randi(0, 1000);
    ($x & $x) == $x;
});

prop_ok("band: zero annihilator (x & 0 == 0)", sub {
    my $x = randi(0, 1000);
    ($x & 0) == 0;
});

prop_ok("band: mask identity for values within mask range", sub {
    my $x = randi(0, 255);
    ($x & 0xFF) == $x;
});

prop_ok("band: commutative", sub {
    my ($x, $y) = (randi(0, 1000), randi(0, 1000));
    ($x & $y) == ($y & $x);
});

# Bitwise OR (|)
prop_ok("bor: idempotent (x | x == x)", sub {
    my $x = randi(0, 1000);
    ($x | $x) == $x;
});

prop_ok("bor: identity (x | 0 == x)", sub {
    my $x = randi(0, 1000);
    ($x | 0) == $x;
});

prop_ok("bor: commutative", sub {
    my ($x, $y) = (randi(0, 1000), randi(0, 1000));
    ($x | $y) == ($y | $x);
});

# Bitwise XOR (^)
prop_ok("bxor: self-XOR is 0", sub {
    my $x = randi(0, 1000);
    ($x ^ $x) == 0;
});

prop_ok("bxor: identity (x ^ 0 == x)", sub {
    my $x = randi(0, 1000);
    ($x ^ 0) == $x;
});

prop_ok("bxor: commutative", sub {
    my ($x, $y) = (randi(0, 1000), randi(0, 1000));
    ($x ^ $y) == ($y ^ $x);
});

prop_ok("bxor: double-XOR cancels (x ^ y ^ y == x)", sub {
    my ($x, $y) = (randi(0, 1000), randi(0, 1000));
    ($x ^ $y ^ $y) == $x;
});

# Bitwise NOT (~)
prop_ok("bnot: double complement is identity", sub {
    my $x = randi(0, 1000);
    (~~$x) == $x;
});

is(~0, 18446744073709551615,  "bnot: ~0 is max unsigned (2^64 - 1 as UV)");
{ use integer; is(~0, -1,     "bnot: ~0 is -1 under use integer (signed)") }
is(~1, 18446744073709551614,  "bnot: ~1 is UV max - 1");
{ use integer; is(~(-1), 0,   "bnot: ~(-1) == 0 under use integer") }

# Left shift (<<)
prop_ok("shl: shift by 0 is identity", sub {
    my $x = randi(0, 1000);
    ($x << 0) == $x;
});

prop_ok("shl: shift left by 1 is multiply by 2", sub {
    my $x = randi(0, 100);
    ($x << 1) == ($x * 2);
});

is(1 << 10, 1024,              "shl: 1 << 10 == 1024");
is(1 << 63, 9223372036854775808, "shl: 1 << 63 is I64 min boundary (as UV)");

# Right shift (>>)
prop_ok("shr: shift by 0 is identity", sub {
    my $x = randi(0, 1000);
    ($x >> 0) == $x;
});

prop_ok("shr: shift right by 1 is divide by 2 (for non-negative)", sub {
    my $x = randi(0, 1000);
    ($x >> 1) == int($x / 2);
});

is(1024 >> 10, 1,   "shr: 1024 >> 10 == 1");
is(255 >> 4, 15,    "shr: 255 >> 4 == 15");

{ use integer; is(-1 >> 1, -1, "shr: -1 >> 1 == -1 under use integer (arithmetic shift)") }
{ use integer; is(-4 >> 1, -2, "shr: -4 >> 1 == -2 under use integer (arithmetic shift)") }

# Negative shift amounts
is(1 << -1, 0,     "shl: negative shift amount reverses to right shift");
is(8 >> -2, 32,    "shr: negative shift amount reverses to left shift");

# Bitwise ops with negative numbers (two's complement)
{ use integer; is(-1 & 0xFF, 0xFF, "band: -1 & 0xFF == 0xFF (all bits set masked)") }
{ use integer; is(-1 | 0, -1,      "bor: -1 | 0 == -1") }
{ use integer; is(-1 ^ -1, 0,      "bxor: -1 ^ -1 == 0") }

prop_ok("bitwise: De Morgan holds for &/|/~", sub {
    my $x = randi(0, 255);
    (~($x & 0xFF) & 0xFF) == ((~$x | ~0xFF) & 0xFF);
});

# Shift by 64 and beyond
is(1 << 64, 0,     "shl: 1 << 64 == 0 (shift beyond word size)");
is(255 >> 64, 0,   "shr: 255 >> 64 == 0 (shift beyond word size)");

done_testing;
