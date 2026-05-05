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

# abs()
is(abs(0), 0, "abs: abs(0) == 0");

prop_ok("abs: abs(positive) == positive", sub {
    my $x = randi(1, 1000);
    abs($x) == $x;
});

prop_ok("abs: abs(negative) == -negative", sub {
    my $x = randi(-1000, -1);
    abs($x) == -$x;
});

prop_ok("abs: abs(x) == abs(-x)", sub {
    my $x = randi(-1000, 1000);
    abs($x) == abs(-$x);
});

prop_ok("abs: result is always non-negative", sub {
    my $x = randi(-1000, 1000);
    abs($x) >= 0;
});

{
    my $min = -9223372036854775808;
    is(abs($min), 9223372036854775808, "abs: abs(I64_MIN) promotes to float (overflow)");
}

# min/max (checker builtins — testing mathematical semantics via ternary)
prop_ok("min: min(x,x) == x (idempotent)", sub {
    my $x = randi(-1000, 1000);
    my $m = ($x < $x) ? $x : $x;
    $m == $x;
});

prop_ok("min/max: min(x,y) <= max(x,y)", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    my $mn = ($x < $y) ? $x : $y;
    my $mx = ($x > $y) ? $x : $y;
    $mn <= $mx;
});

prop_ok("min/max: result is one of the two inputs", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    my $mn = ($x < $y) ? $x : $y;
    my $mx = ($x > $y) ? $x : $y;
    ($mn == $x || $mn == $y) && ($mx == $x || $mx == $y);
});

# int()
prop_ok("int: identity on integers", sub {
    my $x = randi(-1000, 1000);
    int($x) == $x;
});

is(int(3.7), 3,    "int: truncates positive float toward zero");
is(int(-3.7), -3,  "int: truncates negative float toward zero (not floor)");
is(int(0.9), 0,    "int: int(0.9) == 0");
is(int(-0.9), 0,   "int: int(-0.9) == 0 (toward zero, not floor)");
is(int(3.0), 3,    "int: int(3.0) == 3 (no-op on integer-valued float)");

done_testing;
