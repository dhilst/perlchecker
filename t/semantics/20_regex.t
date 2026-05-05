use strict;
use warnings;
use Test::More;

# =~ match operator
ok("hello" =~ /hello/,         "=~: exact match returns true");
ok("hello world" =~ /world/,   "=~: substring match returns true");
ok(!("hello" =~ /xyz/),        "=~: non-match returns false");
is(("hello" =~ /hello/), 1,    "=~: match returns 1 in numeric context");
is(("hello" =~ /xyz/), "",     "=~: non-match returns '' in string context");

# !~ negated match
ok("hello" !~ /xyz/,           "!~: non-match returns true");
ok(!("hello" !~ /hello/),      "!~: match returns false");
{
    my $match    = ("hello" =~ /ell/);
    my $no_match = ("hello" !~ /ell/);
    ok($match && !$no_match, "!~: exact negation of =~");
}

# Character classes
ok("abc123" =~ /\d/,           'regex: \d matches digits');
ok("abc" =~ /\w/,              'regex: \w matches word characters');
ok("hello world" =~ /\s/,      'regex: \s matches whitespace');
ok("abc" =~ /[a-c]/,           "regex: character range [a-c]");

# Anchors
ok("hello" =~ /^hello/,        'regex: ^ anchors to start');
ok(!("xhello" =~ /^hello/),    'regex: ^ fails when not at start');
ok("hello" =~ /hello$/,        'regex: $ anchors to end');
ok(!("hellox" =~ /hello$/),    'regex: $ fails when not at end');
ok("hello\n" =~ /hello$/,      'regex: $ matches before trailing newline');

# Quantifiers
ok("aaa" =~ /a+/,              "regex: + matches one or more");
ok("" =~ /a*/,                 "regex: * matches zero or more");
ok("ab" =~ /a?b/,              "regex: ? matches zero or one");
ok("aaa" =~ /a{3}/,            "regex: {n} matches exactly n");

# Alternation
ok("cat" =~ /cat|dog/,         "regex: alternation matches first option");
ok("dog" =~ /cat|dog/,         "regex: alternation matches second option");
ok(!("fish" =~ /cat|dog/),     "regex: alternation fails when no option matches");
ok("a.b" =~ /a\.b/,            "regex: escaped dot matches literal dot");
ok("axb" =~ /a.b/,             "regex: unescaped dot matches any char");

done_testing;
