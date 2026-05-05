# Round 136 audit: int() on stringified negative numbers
# Perl: int("" . (-5)) == -5
# Z3: str.to_int("-5") == -1 (only non-negative digit strings are supported)
# This is a soundness divergence in the StrToInt encoding.

# sig: (I64) -> I64
# pre: $n >= -100 && $n <= -1
# post: $result == $n
sub neg_int_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    my $r = int($s);
    return $r;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 1;

sub check_prop {
    my ($name, $gen_sub, $check_sub, $n) = @_;
    $n //= 1000;
    for my $trial (1..$n) {
        my @args = $gen_sub->($trial);
        unless ($check_sub->(@args)) { diag("FAIL $name: args=(@args)"); return 0; }
    }
    return 1;
}

ok(check_prop("neg_int_roundtrip",
    sub { my $n = -(1 + int(rand(100))); ($n) },
    sub { my ($n) = @_; neg_int_roundtrip($n) == $n }
), "neg_int_roundtrip: post holds");
