# Audit Round 6: chr() with out-of-range (negative) code point
#
# Fixed bug: Z3's str.from_code returns "" for negative inputs,
# but Perl's chr() always returns a 1-character string even for
# negative code points (returning U+FFFD replacement character).
# The fix clamps out-of-range code points to 65533 (U+FFFD) before
# calling str.from_code, so chr() always returns a 1-char string.

# After fix: chr($x) for negative $x has length 1, matching Perl.
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= -1
# post: $result == 1
sub chr_neg_len {
    my ($x) = @_;
    return length(chr($x));
}

# Control test: For valid code points, chr() returns length 1 in both Perl and Z3.
# sig: (I64) -> I64
# pre: $x >= 32 && $x <= 126
# post: $result == 1
sub chr_valid_len {
    my ($x) = @_;
    return length(chr($x));
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 2;

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

ok(check_prop("chr_neg_len",
    sub { my $t = shift; (Int(range=>[-100,-1], sized=>0)->generate($t)) },
    sub { my ($x) = @_; length(chr($x)) == 1 },
), "chr_neg_len: post holds");

ok(check_prop("chr_valid_len",
    sub { my $t = shift; (Int(range=>[32,126], sized=>0)->generate($t)) },
    sub { my ($x) = @_; length(chr($x)) == 1 },
), "chr_valid_len: post holds");
