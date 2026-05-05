# Round 139: Soundness audit -- exponentiation with zero exponent (0**0)
#
# Perl defines $x ** 0 = 1 for ALL $x, including 0 ** 0 = 1.
# Z3 leaves 0^0 unspecified (the SMT-LIB power function does not
# define the result when both base and exponent are zero).
#
# Before the fix, the checker could not verify 0**0 == 1 and would
# produce a spurious counterexample.  The fix adds an explicit guard:
# when the exponent is 0, return 1 directly instead of relying on
# Z3's power function.

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 100
# post: $result == 1
sub any_pow_zero {
    my ($x) = @_;
    return $x ** 0;
}

# sig: (I64) -> I64
# pre: $x == 0
# post: $result == 1
sub zero_pow_zero {
    my ($x) = @_;
    return $x ** 0;
}

# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 10
# post: $result == $x
sub pow_one_identity {
    my ($x) = @_;
    return $x ** 1;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result == $x * $x
sub pow_two_is_square {
    my ($x) = @_;
    return $x ** 2;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 4;

sub check_prop {
    my ($name, $gen_sub, $check_sub, $n) = @_;
    $n //= 1000;
    for my $trial (1..$n) {
        my @args = $gen_sub->($trial);
        unless ($check_sub->(@args)) { diag("FAIL $name: args=(@args)"); return 0; }
    }
    return 1;
}

ok(check_prop(
    "any_pow_zero",
    sub { (Int(range=>[0,100], sized=>0)->generate($_[0])) },
    sub { my ($x) = @_; any_pow_zero($x) == 1 }
), "any_pow_zero: post holds");

ok(check_prop(
    "zero_pow_zero",
    sub { (0) },
    sub { my ($x) = @_; zero_pow_zero($x) == 1 }
), "zero_pow_zero: post holds");

ok(check_prop(
    "pow_one_identity",
    sub { (Int(range=>[1,10], sized=>0)->generate($_[0])) },
    sub { my ($x) = @_; pow_one_identity($x) == $x }
), "pow_one_identity: post holds");

ok(check_prop(
    "pow_two_is_square",
    sub { (Int(range=>[0,10], sized=>0)->generate($_[0])) },
    sub { my ($x) = @_; pow_two_is_square($x) == $x * $x }
), "pow_two_is_square: post holds");
