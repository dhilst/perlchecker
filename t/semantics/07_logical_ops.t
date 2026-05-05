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

# && — returns deciding value, NOT boolean
is(5 && 3, 3,           "&&: returns right operand when left is true");
is(0 && 3, 0,           "&&: returns left operand (0) when left is false");
is("" && 3, "",         "&&: returns left operand ('') when left is false");
is("hello" && "world", "world", "&&: returns right when both true strings");
{
    no warnings 'uninitialized';
    is(undef && 3, undef, "&&: returns undef when left is undef");
}

# || — returns deciding value, NOT boolean
is(5 || 3, 5,           "||: returns left operand when left is true");
is(0 || 3, 3,           "||: returns right operand when left is false");
is("" || "world", "world", "||: returns right when left is empty string");
is(0 || 0, 0,           "||: returns right operand when both are false");
is(0 || "", "",         "||: returns right operand (empty string) when both false");

# ! — returns '' or 1
is(!0, 1,               "!: !0 returns 1");
is(!1, "",              "!: !1 returns ''");
is(!"", 1,             "!: !'' returns 1");
is(!"hello", "",        "!: !'hello' returns ''");

prop_ok("!: double negation normalizes to 0/1", sub {
    my $x = randi(-1000, 1000);
    !!$x == ($x ? 1 : 0);
});

# Short-circuit evaluation
{
    my $side_effect = 0;
    0 && ($side_effect = 1);
    is($side_effect, 0, "&&: short-circuits — right not evaluated when left is false");
}
{
    my $side_effect = 0;
    1 || ($side_effect = 1);
    is($side_effect, 0, "||: short-circuits — right not evaluated when left is true");
}
{
    my $side_effect = 0;
    1 && ($side_effect = 1);
    is($side_effect, 1, "&&: evaluates right when left is true");
}
{
    my $side_effect = 0;
    0 || ($side_effect = 1);
    is($side_effect, 1, "||: evaluates right when left is false");
}

# and / or / not — same semantics, lower precedence
is((5 and 3), 3,        "and: same semantics as && (returns right when left true)");
is((0 or 3), 3,         "or: same semantics as || (returns right when left false)");
is((not 0), 1,          "not: same semantics as ! (returns 1 for false)");

# De Morgan's laws
prop_ok("De Morgan: !(&&) == (|| !)", sub {
    my ($x, $y) = (randi(0, 1), randi(0, 1));
    !($x && $y) == (!$x || !$y);
});

prop_ok("De Morgan: !(||) == (&& !)", sub {
    my ($x, $y) = (randi(0, 1), randi(0, 1));
    !($x || $y) == (!$x && !$y);
});

# Chained logical operators
is(0 || 0 || 3, 3,     "||: chained — returns first truthy value");
is(0 || "" || 0 || 5 || 6, 5, "||: chained — skips falsy, returns first truthy");
is("0" && "hello", "0", '&&: string "0" is false, short-circuits');

# Precedence: and/or vs &&/||
{
    my $x = 0 || 2;
    is($x, 2, "||: higher precedence than = (0 || 2 gives 2)");
}
{
    my $x = 1 || 2;
    is($x, 1, "||: short-circuits in assignment context");
}
{
    my $x;
    eval '$x = 0 or 2';
    is($x, 0, "or: lower precedence than = (assignment happens first)");
}
{
    my $x;
    eval '$x = 0 || 2';
    is($x, 2, "||: higher precedence than = via eval comparison");
}

done_testing;
