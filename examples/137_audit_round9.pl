# =============================================================
# Audit Round 9: int() / StrToInt on non-numeric strings
# =============================================================
# Perl's int() returns 0 for non-numeric strings:
#   int("") == 0, int("abc") == 0, int("-abc") == 0, int("-") == 0
# Z3's str.to_int returns -1 for non-digit strings, which leaked
# through the encoding. Fixed by clamping the -1 sentinel to 0.

# --- int("") should be 0, not -1 ---
# sig: (Str) -> I64
# pre: length($x) == 0
# post: $result == 0
sub int_empty_string {
    my ($x) = @_;
    return int($x);
}

# --- int of a non-numeric string should be 0 ---
# sig: (Str) -> I64
# pre: $x eq "abc"
# post: $result == 0
sub int_nonnumeric {
    my ($x) = @_;
    return int($x);
}

# --- int("-abc") should be 0, not 1 ---
# sig: (Str) -> I64
# pre: $x eq "-abc"
# post: $result == 0
sub int_neg_nonnumeric {
    my ($x) = @_;
    return int($x);
}

# --- int("-") should be 0 ---
# sig: (Str) -> I64
# pre: $x eq "-"
# post: $result == 0
sub int_bare_minus {
    my ($x) = @_;
    return int($x);
}

# --- valid negative int conversion still works ---
# sig: (Str) -> I64
# pre: $x eq "-42"
# post: $result == -42
sub int_neg_valid {
    my ($x) = @_;
    return int($x);
}

# --- valid positive int conversion still works ---
# sig: (Str) -> I64
# pre: $x eq "99"
# post: $result == 99
sub int_pos_valid {
    my ($x) = @_;
    return int($x);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 6;

sub check_prop {
    my ($name, $gen_sub, $check_sub, $n) = @_;
    $n //= 1000;
    for my $trial (1..$n) {
        my @args = $gen_sub->($trial);
        unless ($check_sub->(@args)) { diag("FAIL $name: args=(@args)"); return 0; }
    }
    return 1;
}

ok(check_prop("int_empty_string",
    sub { ("") },
    sub { int_empty_string($_[0]) == 0 }
), "int_empty_string: post holds");

ok(check_prop("int_nonnumeric",
    sub { ("abc") },
    sub { int_nonnumeric($_[0]) == 0 }
), "int_nonnumeric: post holds");

ok(check_prop("int_neg_nonnumeric",
    sub { ("-abc") },
    sub { int_neg_nonnumeric($_[0]) == 0 }
), "int_neg_nonnumeric: post holds");

ok(check_prop("int_bare_minus",
    sub { ("-") },
    sub { int_bare_minus($_[0]) == 0 }
), "int_bare_minus: post holds");

ok(check_prop("int_neg_valid",
    sub { ("-42") },
    sub { int_neg_valid($_[0]) == -42 }
), "int_neg_valid: post holds");

ok(check_prop("int_pos_valid",
    sub { ("99") },
    sub { int_pos_valid($_[0]) == 99 }
), "int_pos_valid: post holds");
