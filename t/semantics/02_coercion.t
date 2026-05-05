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

# I64 → Str
is("" . 42,  "42",  "I64->Str: positive integer stringifies");
is("" . 0,   "0",   "I64->Str: zero stringifies to '0'");
is("" . -5,  "-5",  "I64->Str: negative integer stringifies with minus sign");
is("" . 9223372036854775807,  "9223372036854775807",  "I64->Str: I64_MAX stringifies correctly (19 chars)");
is("" . -9223372036854775808, "-9223372036854775808", "I64->Str: I64_MIN stringifies correctly (20 chars)");

prop_ok("I64->Str: single digit produces 1-char string", sub {
    my $n = int(rand(10));
    length("" . $n) == 1;
});

prop_ok("I64->Str: two-digit produces 2-char string", sub {
    my $n = int(rand(90)) + 10;
    length("" . $n) == 2;
});

prop_ok("I64->Str: negative single digit produces 2-char string", sub {
    my $n = -(int(rand(9)) + 1);
    length("" . $n) == 2;
});

prop_ok("I64->Str->I64: roundtrip preserves value for small positives", sub {
    my $n = int(rand(1000));
    ("" . $n) + 0 == $n;
});

# Str → I64
is(int("42"),     42,  "Str->I64: pure digit string converts exactly");
is(int("-7"),     -7,  "Str->I64: negative digit string converts exactly");
is(int("0"),      0,   "Str->I64: '0' converts to 0");
{ no warnings 'numeric'; is(int(""),      0,   "Str->I64: empty string converts to 0") }
{ no warnings 'numeric'; is(int("3abc"),  3,   "Str->I64: leading digits extracted, trailing non-digits ignored") }
is(int("  42"),   42,  "Str->I64: leading whitespace stripped");
{ no warnings 'numeric'; is(int("abc"),   0,   "Str->I64: non-numeric string converts to 0") }
{ no warnings 'numeric'; is(int("  -12xyz"), -12, "Str->I64: leading whitespace + sign + digits + junk") }
is(int("3.14"),   3,   "Str->I64: float string truncated toward zero");
is(int("-3.14"),  -3,  "Str->I64: negative float string truncated toward zero");

# Implicit numeric coercion
is("42" + 0,    42, "Str->I64 implicit: string in addition context");
{ no warnings 'numeric'; is("" + 0,      0,  "Str->I64 implicit: empty string is 0") }
{ no warnings 'numeric'; is("hello" + 0, 0,  "Str->I64 implicit: non-numeric string is 0") }

# Boolean context
ok(!0,         "Bool: 0 is false");
ok(!"",        "Bool: empty string is false");
ok(!"0",       'Bool: string "0" is false');
{ my $x; ok(!$x, "Bool: undef is false") }
ok(1 ? 1 : 0,           "Bool: 1 is true");
ok("1" ? 1 : 0,         "Bool: string '1' is true");
ok("00" ? 1 : 0,        "Bool: string '00' is true (only '0' is false)");
ok(" " ? 1 : 0,         "Bool: single space is true");
ok("0E0" ? 1 : 0,       "Bool: '0E0' is true (zero but true idiom)");

prop_ok("Bool: positive integers are true", sub {
    my $x = int(rand(1000)) + 1;
    $x ? 1 : 0;
});

prop_ok("Bool: negative integers are true", sub {
    my $x = -(int(rand(1000)) + 1);
    $x ? 1 : 0;
});

# ! return values
is(!0, 1,  "Bool: !false returns 1");
is(!1, "", "Bool: !true returns ''");

# Cross-coercion edge cases
{
    my $s = "0" . "";
    ok(!$s, "Coercion: '0' concatenated with '' is still false");
}
{
    my $n = "0" + 0;
    ok(!$n, "Coercion: '0' in numeric context is 0 (false)");
}

done_testing;
