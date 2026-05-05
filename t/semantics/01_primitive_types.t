use strict;
use warnings;
no warnings 'uninitialized';
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

# I64 range
is((1 << 63) - 1, 9223372036854775807, "I64: max value is 2^63 - 1");
is(-(1 << 63), -9223372036854775808,    "I64: min value is -(2^63)");

# I64 overflow promotes to float
{
    my $max = 9223372036854775807;
    is($max + 1, 9223372036854775808, "I64: overflow promotes to float (max + 1)");
}
{
    my $min = -9223372036854775808;
    is($min - 1, -9223372036854775809, "I64: underflow promotes to float (min - 1)");
}

# use integer wraps (two's complement)
{
    my $r = do { use integer; my $max = 9223372036854775807; $max + 1 };
    is($r, -9223372036854775808, "I64: use integer wraps on overflow");
}
{
    my $r = do { use integer; my $min = -9223372036854775808; $min - 1 };
    is($r, 9223372036854775807, "I64: use integer wraps on underflow");
}

# Small integers exact
prop_ok("I64: small integers are exact", sub {
    my $x = int(rand(2001)) - 1000;
    ($x + 0) == $x;
});

ok(0 == 0 && -0 == 0,     "I64: no negative zero");
ok(0xFF == 255,            "I64: hex literals");
ok(0b11111111 == 255,      "I64: binary literals");
ok(0377 == 255,            "I64: octal literals");

# Str
is(length(""), 0,          "Str: empty string has length 0");
ok("" eq "",               "Str: empty string equals itself");
is(length("abc"), 3,       "Str: length counts characters");
ok(ord("A") == 65 && ord("a") == 97 && ord("0") == 48, "Str: ASCII code points are standard");
ok(length("\n") == 1 && length("\t") == 1 && length("\\") == 1, "Str: escape sequences are single characters");
is(length("\0"), 1,        "Str: null byte is a valid single character");
is(ref(\("hello")), "SCALAR", "Str: strings are scalars");
{
    my $s = "test";
    ok(("" . $s) eq $s && ($s . "") eq $s, "Str: empty string is concat identity");
}
{
    my $a = "hello";
    my $b = "hello";
    ok($a eq $b, "Str: equality is by value not reference");
}
is(length("\x{263A}"), 1,  "Str: length counts code points not bytes (Unicode)");

# undef
{
    my $x;
    is($x + 0, 0, "I64: undef numifies to 0");
}
{
    my $x;
    is("" . $x, "", "Str: undef stringifies to empty string");
}

done_testing;
