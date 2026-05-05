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

# Concatenation
is("a" . "b", "ab",       "concat: basic concatenation");
is("" . "x", "x",         "concat: empty string is left identity");
is("x" . "", "x",         "concat: empty string is right identity");
is("a" . "b" . "c", "abc","concat: left-associative chaining");
is(length("abc" . "de"), 5, "concat: length is additive");
is("x" . 42, "x42",       "concat: I64 right operand coerced to string");
is(42 . "x", "42x",       "concat: I64 left operand coerced to string");
is(1 . 2 . 3, "123",      "concat: chaining integers via concat");
isnt("a" . "b", "b" . "a","concat: not commutative");

# Repetition
is("ab" x 3, "ababab",    "repeat: basic repetition");
is("x" x 1, "x",          "repeat: x 1 is identity");
is("x" x 0, "",           "repeat: x 0 is empty string");
is("abc" x -1, "",        "repeat: negative count produces empty string");
is("" x 5, "",            "repeat: empty string repeated is still empty");
is("a" x 5, "aaaaa",      "repeat: single char repeated");

prop_ok("repeat: length is original_length * count", sub {
    my $n = randi(1, 10);
    length("ab" x $n) == 2 * $n;
});

done_testing;
