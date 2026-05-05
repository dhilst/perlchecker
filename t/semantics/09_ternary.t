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

# Basic branch selection
is((1 ? "yes" : "no"), "yes",  "ternary: true condition selects left branch");
is((0 ? "yes" : "no"), "no",   "ternary: false condition selects right branch");
is(("" ? "yes" : "no"), "no",  "ternary: empty string is falsy");
is(("0" ? "yes" : "no"), "no", "ternary: string '0' is falsy");

prop_ok("ternary: positive integer is truthy", sub {
    my $x = randi(1, 1000);
    ($x ? "yes" : "no") eq "yes";
});

# Type preservation
prop_ok("ternary: returns integer when true branch is integer", sub {
    my $x = randi(0, 100);
    my $r = (1 ? $x : "string");
    $r == $x;
});

is((0 ? 42 : "hello"), "hello", "ternary: returns string when false branch is string");

# Only one branch evaluated
{
    my $side = 0;
    my $r = (1 ? 42 : ($side = 1));
    is($side, 0, "ternary: false branch not evaluated when condition is true");
}
{
    my $side = 0;
    my $r = (0 ? ($side = 1) : 42);
    is($side, 0, "ternary: true branch not evaluated when condition is false");
}

# Nested ternaries — right-associative
is((1 ? 1 : 0 ? 2 : 3), 1,  "ternary: nested — first true wins");
is((0 ? 1 : 1 ? 2 : 3), 2,  "ternary: nested — falls through to second condition");
is((0 ? 1 : 0 ? 2 : 3), 3,  "ternary: nested — all false gives last branch");

# Ternary as expression
prop_ok("ternary: can compute absolute value", sub {
    my $x = randi(-100, 100);
    my $abs = ($x >= 0 ? $x : -$x);
    $abs >= 0;
});

done_testing;
