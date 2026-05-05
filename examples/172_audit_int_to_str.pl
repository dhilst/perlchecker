# Round 172: Audit int-to-string coercion in string concatenation.
#
# Perl's . operator coerces integers to decimal strings:
#   "x" . 42   => "x42"
#   "x" . -5   => "x-5"
#   "" . (~0)  => "18446744073709551615" (unsigned BitNot)
#
# The unsoundness was: BitNot used the signed algebraic identity ~x == -x-1,
# producing negative values. When coerced to string via FromInt, this gave
# e.g. "-6" instead of "18446744073709551610" for ~5. Fixed by switching
# BitNot to unsigned arithmetic (add 2^64 when signed result is negative).

# sig: (Int) -> Str
# pre: $n == -5
# post: $result eq "x-5"
sub neg_concat {
    my ($n) = @_;
    return "x" . $n;
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 9
# post: $result == 1
sub single_digit_str_len {
    my ($n) = @_;
    my $s = "" . $n;
    return length($s);
}

# sig: (Int) -> Int
# pre: $n >= -99 && $n <= -10
# post: $result == 3
sub neg_two_digit_str_len {
    my ($n) = @_;
    my $s = "" . $n;
    return length($s);
}

# The key fix: ~$n for non-negative $n produces an unsigned result.
# "" . (~5) has length 20 (the string "18446744073709551610"), NOT 2 (not "-6").
# sig: (Int) -> Int
# pre: $n == 5
# post: $result == 20
sub bitnot_str_len_unsigned {
    my ($n) = @_;
    my $s = "" . (~$n);
    return length($s);
}

# Roundtrip: int("" . $n) == $n for small positive integers
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 9
# post: $result == $n
sub int_str_roundtrip_pos {
    my ($n) = @_;
    my $s = "" . $n;
    return int($s);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ n <- Int(range=>[-5,-5], sized=>0) ]##
    neg_concat($n) eq "x-5";
}, name => "neg_concat: post holds";

Property {
    ##[ n <- Int(range=>[0,9], sized=>0) ]##
    single_digit_str_len($n) == 1;
}, name => "single_digit_str_len: post holds";

Property {
    ##[ n <- Int(range=>[-99,-10], sized=>0) ]##
    neg_two_digit_str_len($n) == 3;
}, name => "neg_two_digit_str_len: post holds";

Property {
    ##[ n <- Int(range=>[0,9], sized=>0) ]##
    my $s = "" . (~$n);
    length($s) == 20;
}, name => "bitnot_str_len_unsigned: post holds";

Property {
    ##[ n <- Int(range=>[0,9], sized=>0) ]##
    int_str_roundtrip_pos($n) == $n;
}, name => "int_str_roundtrip_pos: post holds";
