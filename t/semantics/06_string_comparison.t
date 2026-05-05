use strict;
use warnings;
use Test::More;

# eq / ne
ok("abc" eq "abc",   "eq: identical strings are equal");
ok(!("abc" eq "ABC"), "eq: case-sensitive");
ok("" eq "",          "eq: empty strings are equal");
ok(!("" eq " "),      "eq: empty string not equal to space");
ok("abc" ne "def",    "ne: different strings");
ok(!("abc" ne "abc"), "ne: same strings are not ne");

# lt / gt / le / ge — lexicographic by code point
ok("a" lt "b",        "lt: lexicographic order (a < b)");
ok("A" lt "a",        "lt: uppercase before lowercase (ASCII order)");
ok("" lt "a",         "lt: empty string is less than any non-empty string");
ok("abc" lt "abd",    "lt: differs at last character");
ok("abc" lt "abcd",   "lt: prefix is less than longer string");
ok(!("abc" lt "abc"), "lt: not less than self");
is(("a" lt "b"), ("b" gt "a"), "lt/gt: antisymmetric pair");
ok("abc" le "abc",    "le: less than or equal (equal case)");
ok("abc" le "abd",    "le: less than or equal (less case)");
ok("abc" ge "abc",    "ge: greater or equal (equal case)");
ok("abd" ge "abc",    "ge: greater or equal (greater case)");

# cmp
is("abc" cmp "abc", 0,  "cmp: equal strings return 0");
is("a" cmp "b",     -1, "cmp: lesser string returns -1");
is("b" cmp "a",     1,  "cmp: greater string returns 1");
is("abc" cmp "def", -(("def" cmp "abc")), "cmp: antisymmetry");
is("" cmp "a",      -1, "cmp: empty string compares less than non-empty");

# Return value type
is(("abc" eq "abc"), 1,  "eq: true returns 1");
is(("abc" eq "def"), "", "eq: false returns '' (empty string)");

# Numeric strings — lexicographic vs numeric
ok("10" lt "9",        "lt: '10' lt '9' is true (lexicographic, not numeric)");
ok("2" gt "10",        "gt: '2' gt '10' is true (lexicographic)");
is(!("10" lt "9"), (10 < 9), "lt vs <: string and numeric comparison differ for numeric strings");

done_testing;
