use strict;
use warnings;
use Test::More;
use POSIX qw(floor);

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

# Addition
prop_ok("add: commutativity", sub {
    my ($x, $y) = (randi(-1000,1000), randi(-1000,1000));
    ($x + $y) == ($y + $x);
});

prop_ok("add: identity (x + 0 == x)", sub {
    my $x = randi(-1000,1000);
    ($x + 0) == $x;
});

prop_ok("add: associativity", sub {
    my ($x, $y, $z) = (randi(-1000,1000), randi(-1000,1000), randi(-1000,1000));
    (($x + $y) + $z) == ($x + ($y + $z));
});

prop_ok("add: additive inverse (x + (-x) == 0)", sub {
    my $x = randi(-1000,1000);
    ($x + (-$x)) == 0;
});

# Subtraction
prop_ok("sub: x - x == 0", sub {
    my $x = randi(-1000,1000);
    ($x - $x) == 0;
});

prop_ok("sub: x - 0 == x", sub {
    my $x = randi(-1000,1000);
    ($x - 0) == $x;
});

prop_ok("sub: antisymmetry", sub {
    my ($x, $y) = (randi(-1000,1000), randi(-1000,1000));
    ($x - $y) == -($y - $x);
});

# Multiplication
prop_ok("mul: commutativity", sub {
    my ($x, $y) = (randi(-1000,1000), randi(-1000,1000));
    ($x * $y) == ($y * $x);
});

prop_ok("mul: identity (x * 1 == x)", sub {
    my $x = randi(-1000,1000);
    ($x * 1) == $x;
});

prop_ok("mul: zero (x * 0 == 0)", sub {
    my $x = randi(-1000,1000);
    ($x * 0) == 0;
});

prop_ok("mul: x * -1 == -x", sub {
    my $x = randi(-1000,1000);
    ($x * -1) == -$x;
});

prop_ok("mul: distributive over addition", sub {
    my ($x, $y, $z) = (randi(-100,100), randi(-100,100), randi(-100,100));
    ($x * ($y + $z)) == ($x * $y + $x * $z);
});

# Division
prop_ok("div: x / 1 == x", sub {
    my $x = randi(1,1000);
    ($x / 1) == $x;
});

prop_ok("div: x / x == 1", sub {
    my $x = randi(1,1000);
    ($x / $x) == 1;
});

is(7 / 2,        3.5,  "div: 7 / 2 == 3.5 (not truncated)");
is(int(7 / 2),   3,    "div: int(7/2) == 3 (truncation toward zero)");
is(int(-7 / 2),  -3,   "div: int(-7/2) == -3 (toward zero, not floor)");
is(int(7 / -2),  -3,   "div: int(7/-2) == -3");
is(int(-7 / -2), 3,    "div: int(-7/-2) == 3");

{
    eval { my $x = 1 / 0 };
    like($@, qr/division by zero/i, "div: division by zero dies");
}

# Modulo
prop_ok("mod: result in [0, y) when y > 0", sub {
    my ($x, $y) = (randi(-1000,1000), randi(1,100));
    my $r = $x % $y;
    $r >= 0 && $r < $y;
});

prop_ok("mod: result in (y, 0] when y < 0", sub {
    my ($x, $y) = (randi(-1000,1000), randi(-100,-1));
    my $r = $x % $y;
    $r <= 0 && $r > $y;
});

prop_ok("mod: division identity for non-negative x", sub {
    my ($x, $y) = (randi(0,1000), randi(1,100));
    $x == int($x / $y) * $y + ($x % $y);
});

is(7 % 3,    1,  "mod: 7 % 3 == 1 (positive/positive)");
is(-7 % 3,   2,  "mod: -7 % 3 == 2 (negative/positive, floor-modulo)");
is(7 % -3,   -2, "mod: 7 % -3 == -2 (positive/negative)");
is(-7 % -3,  -1, "mod: -7 % -3 == -1 (negative/negative)");

# Division identity caveat
is(int(-921/14), -65, "mod: int() truncates toward zero");
is(-921 % 14,    3,   "mod: % uses floor-modulo");
is(floor(-921/14), -66, "mod: floor() is the correct divisor for identity");

{
    eval { my $x = 1 % 0 };
    like($@, qr/modulus zero|division by zero/i, "mod: modulo by zero dies");
}

# Exponentiation
is(0 ** 0,  1,    "exp: 0**0 == 1");
prop_ok("exp: x**0 == 1", sub { my $x = randi(-100,100); ($x ** 0) == 1 });
prop_ok("exp: x**1 == x", sub { my $x = randi(-100,100); ($x ** 1) == $x });
prop_ok("exp: x**2 == x*x", sub { my $x = randi(-10,10); ($x ** 2) == ($x * $x) });

{
    my $r = 2 ** -1;
    ok($r > 0 && $r < 1, "exp: positive**negative gives float in (0,1)");
}

is(2 ** 10,  1024, "exp: 2**10 == 1024");
ok((-1) ** 2 == 1 && (-1) ** 3 == -1, "exp: (-1)**even == 1, (-1)**odd == -1");

# Increment / Decrement
prop_ok("incr: x++ increments by 1", sub {
    my $x = randi(-1000,1000); my $y = $x; $y++; $y == ($x + 1);
});

prop_ok("decr: x-- decrements by 1", sub {
    my $x = randi(-1000,1000); my $y = $x; $y--; $y == ($x - 1);
});

# Unary negation
prop_ok("neg: double negation is identity", sub {
    my $x = randi(-1000,1000); -(-$x) == $x;
});
is(-(0), 0, "neg: -0 == 0");

# I64 boundary edge cases
{
    my $min = -9223372036854775808;
    is($min / -1, 9223372036854775808, "div: I64_MIN / -1 overflows to float");
}
{
    use integer;
    my $min = -9223372036854775808;
    is($min * -1, -9223372036854775808, "mul: I64_MIN * -1 wraps to I64_MIN under use integer");
}

is(-9223372036854775808 % -1, 0, "mod: I64_MIN % -1 == 0");
ok(2 ** 63 == 9223372036854775808, "exp: 2**63 overflows I64 (becomes float)");

done_testing;
