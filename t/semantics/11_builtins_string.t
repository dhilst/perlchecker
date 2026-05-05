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

# length()
is(length(""), 0,           "length: empty string is 0");
is(length("abc"), 3,        "length: counts characters");
is(length("\n\t\0"), 3,     "length: escape chars are single characters");
is(length("\x{263A}"), 1,   "length: Unicode code point is 1 character");

# substr()
is(substr("hello", 0), "hello",    "substr: from 0 returns whole string");
is(substr("hello", 1), "ello",     "substr: from 1 skips first char");
is(substr("hello", 0, 3), "hel",   "substr: 3-arg extracts substring");
is(substr("hello", -2), "lo",      "substr: negative start wraps from end");
is(substr("hello", -3, 2), "ll",   "substr: negative start with length");
is(substr("hello", 1, 0), "",      "substr: length 0 returns empty string");
is(substr("hello", 5), "",         "substr: start at end returns empty string");
is(substr("hello", 0, 100), "hello", "substr: length beyond end clamps to available");
is(substr("hello", 1, -1), "ell",  "substr: negative length means leave off N from end");

# index()
is(index("hello world", "world"), 6,  "index: finds substring position");
is(index("hello", "xyz"), -1,         "index: returns -1 when not found");
is(index("hello", ""), 0,             "index: empty needle returns 0");
is(index("hello", "l"), 2,            "index: finds first occurrence");
is(index("hello", "l", 3), 3,         "index: 3-arg starts search from position");
is(index("hello", "l", 4), -1,        "index: 3-arg returns -1 when not found from position");
is(index("", ""), 0,                   "index: empty in empty returns 0");
is(index("", "a"), -1,                 "index: non-empty in empty returns -1");
is(index("hello", "", 3), 3,           "index: empty needle at position returns that position");
is(index("hello", "", 10), 5,          "index: empty needle beyond end returns length");

# ord()
is(ord("A"), 65,       "ord: ord('A') == 65");
is(ord("a"), 97,       "ord: ord('a') == 97");
is(ord("\0"), 0,       "ord: ord(null byte) == 0");
is(ord("ABC"), 65,     "ord: multi-char string returns first char's code point");
{ no warnings 'uninitialized'; is(ord(""), 0, "ord: ord('') returns 0 (undef numified)") }
is(ord("\x{263A}"), 9786, "ord: Unicode smiley code point is 9786");

# chr()
is(chr(65), "A",       "chr: chr(65) eq 'A'");
is(chr(0), "\0",       "chr: chr(0) is null byte");
is(length(chr(65)), 1, "chr: produces single character");
is(chr(9786), "\x{263A}", "chr: chr(9786) produces Unicode smiley");

prop_ok("chr/ord: roundtrip for printable ASCII", sub {
    my $n = randi(32, 126);
    ord(chr($n)) == $n;
});

# chomp()
{
    my $s = "hello\n";
    chomp($s);
    is($s, "hello", "chomp: removes trailing newline");
}
{
    my $s = "hello";
    my $r = chomp($s);
    is($s, "hello", "chomp: no newline — string unchanged");
    is($r, 0,       "chomp: no newline — returns 0");
}
{
    my $s = "hello\n";
    my $r = chomp($s);
    is($r, 1, "chomp: returns 1 when newline removed");
}
{
    my $s = "hello\n\n";
    chomp($s);
    is($s, "hello\n", "chomp: removes only one trailing newline");
}
{
    my $s = "";
    my $r = chomp($s);
    is($s, "", "chomp: empty string — no change");
    is($r, 0,  "chomp: empty string — returns 0");
}
{
    my $s = "hello\r\n";
    chomp($s);
    is($s, "hello\r", "chomp: removes only \\n, leaves \\r from \\r\\n");
}
{
    my $s = "hello\r";
    my $r = chomp($s);
    is($s, "hello\r", "chomp: does not remove standalone \\r");
    is($r, 0,         "chomp: standalone \\r returns 0");
}

# reverse()
is(reverse("hello"), "olleh", "reverse: reverses character order");
is(reverse(""), "",           "reverse: empty string stays empty");
is(reverse("a"), "a",         "reverse: single char is identity");
is(scalar reverse(scalar reverse("hello")), "hello", "reverse: double reverse is identity");
is(length(reverse("hello")), length("hello"), "reverse: preserves length");

# contains/starts_with/ends_with semantics (via index/substr)
ok(index("hello world", "world") != -1, "contains: found via index != -1");
ok(index("hello", "xyz") == -1,         "contains: not found via index == -1");
ok(index("hello", "") != -1,            "contains: every string contains empty string");
is(substr("hello", 0, 3), "hel",        "starts_with: equivalent to substr(s, 0, len(t)) eq t");
is(substr("hello", -2), "lo",           "ends_with: equivalent to substr(s, -len(t)) eq t");

# replace semantics (via s///)
{
    my $s = "hello world";
    (my $r = $s) =~ s/world/perl/;
    is($r, "hello perl", "replace: first occurrence replacement");
}
{
    my $s = "aaa";
    (my $r = $s) =~ s/a/b/g;
    is($r, "bbb", "replace: global replacement (all occurrences)");
}
{
    my $s = "hello";
    (my $r = $s) =~ s/xyz/abc/;
    is($r, "hello", "replace: no match — string unchanged");
}

# char_at semantics (via substr)
is(substr("hello", 0, 1), "h",  "char_at: position 0 is first char");
is(substr("hello", 4, 1), "o",  "char_at: last position");
is(substr("hello", -1, 1), "o", "char_at: negative index wraps from end");
is(substr("hello", -2, 1), "l", "char_at: negative index -2 is second from end");

done_testing;
