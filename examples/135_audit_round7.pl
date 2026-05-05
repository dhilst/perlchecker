# Audit Round 7: ord("") returns 0 in Perl but -1 in the checker (FIXED)
#
# Z3's str.to_code returns -1 for strings of length != 1.
# The Round 5 fix extracts the first char via str.at($s, 0), but
# str.at("", 0) still produces "" which str.to_code maps to -1.
# Perl's ord("") returns 0. Fix: guard with length == 0 check.

# REGRESSION: checker must NOT verify this (Perl returns 0, not -1)
# sig: (Str) -> I64
# pre: length($s) == 0
# post: $result == -1
sub ord_empty_wrong {
    my ($s) = @_;
    return ord($s);
}

# After fix: checker should verify this (Perl returns 0 for empty)
# sig: (Str) -> I64
# pre: length($s) == 0
# post: $result == 0
sub ord_empty_correct {
    my ($s) = @_;
    return ord($s);
}

# After fix: ord() always returns >= 0 for any string (including empty)
# sig: (Str) -> I64
# post: $result >= 0
sub ord_always_nonneg {
    my ($s) = @_;
    return ord($s);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 3;

sub check_prop {
    my ($name, $gen_sub, $check_sub, $n) = @_;
    $n //= 1000;
    for my $trial (1..$n) {
        my @args = $gen_sub->($trial);
        unless ($check_sub->(@args)) {
            diag("FAIL $name: args=(@args)");
            return 0;
        }
    }
    return 1;
}

ok(!check_prop("ord_empty_wrong",
    sub { ("") },
    sub { my ($s) = @_; ord($s) == -1 },
), "ord_empty_wrong: Perl returns 0, not -1 (should FAIL)");

ok(check_prop("ord_empty_correct",
    sub { ("") },
    sub { my ($s) = @_; ord($s) == 0 },
), "ord_empty_correct: Perl returns 0 for empty string");

ok(check_prop("ord_always_nonneg",
    sub { my $t = shift; (String(charset=>"a-z\\n", length=>[0,10])->generate($t)) },
    sub { my ($s) = @_; ord($s) >= 0 },
), "ord_always_nonneg: ord is always >= 0");
