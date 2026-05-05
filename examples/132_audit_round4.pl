# Audit Round 4: Exponentiation ** with negative exponent — floor vs truncation
#
# Z3's Int.power returns Real, and the checker converts back via real2int (floor).
# But Perl returns a float, and when used as Int, it truncates toward zero.
# For negative fractional results: floor(-0.5) = -1, but trunc(-0.5) = 0.
# This causes unsoundness: checker verifies (-x)**-1 == -1, but Perl disagrees.

# UNSOUND CASE: checker says verified, but Perl says false.
# (-x)**-1 is a negative fraction (e.g., -0.5), floor = -1, but Perl float != -1.
# sig: (I64) -> I64
# pre: $x >= 2 && $x <= 10
# post: $result == -1
sub neg_pow_unsound {
    my ($x) = @_;
    return (-$x) ** -1;
}

# CORRECT CASE: positive exponents work fine (result is exact integer).
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result == $x * $x * $x
sub cube_ok {
    my ($x) = @_;
    return $x ** 3;
}

# CORRECT CASE: positive base, negative exp — floor(1/x) = 0 = trunc(1/x).
# sig: (I64) -> I64
# pre: $x >= 2 && $x <= 100
# post: $result == 0
sub pos_pow_neg_exp_ok {
    my ($x) = @_;
    return $x ** -1;
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

# This test FAILS in Perl — the postcondition is wrong (demonstrates unsoundness)
ok(check_prop("neg_pow_unsound",
    sub { my $t = shift; (Int(range=>[2,10], sized=>0)->generate($t)) },
    sub { my ($x) = @_; ((-$x) ** -1) == -1 },
), "neg_pow_unsound: (-x)**-1 == -1 (SHOULD FAIL)") or diag("Expected failure: Perl float != -1");

ok(check_prop("cube_ok",
    sub { my $t = shift; (Int(range=>[-10,10], sized=>0)->generate($t)) },
    sub { my ($x) = @_; $x ** 3 == $x * $x * $x },
), "cube_ok: x**3 == x*x*x");

ok(check_prop("pos_pow_neg_exp_ok",
    sub { my $t = shift; (Int(range=>[2,100], sized=>0)->generate($t)) },
    sub { my ($x) = @_; int($x ** -1) == 0 },
), "pos_pow_neg_exp_ok: int(x**-1) == 0 for x >= 2");
