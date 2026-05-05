# Round 133 audit: ord() on multi-character strings
# Z3's str.to_code returns -1 for strings of length != 1,
# but Perl's ord() always returns the code of the first character (>= 0).
# Fix: encode ord($s) as str.to_code(str.at($s, 0)) to extract first char.

# REGRESSION TEST: this must NOT be verified (it's false in Perl).
# Before the fix, the checker wrongly verified it because Z3 believed
# ord($x) >= 0 implies length($x) == 1.
# sig: (Str) -> I64
# pre: length($x) >= 1 && ord($x) >= 0
# post: length($x) == 1
sub ord_implies_len1 {
    my ($x) = @_;
    return length($x);
}

# POSITIVE TEST: ord of a known single-char string is non-negative.
# sig: (Str) -> I64
# pre: length($x) == 1
# post: $result >= 0
sub ord_single_char_nonneg {
    my ($x) = @_;
    return ord($x);
}

# POSITIVE TEST: ord returns same value regardless of string length
# (operates on first char). For single-char strings, ord(substr($x,0,1))
# equals ord($x).
# sig: (Str) -> I64
# pre: length($x) == 1
# post: $result == ord($x)
sub ord_of_first_char {
    my ($x) = @_;
    return ord(substr($x, 0, 1));
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- String(charset=>"a-z", length=>[1,10]) ]##
    # In Perl, ord($x) >= 0 for ANY non-empty string.
    # So the precondition is always satisfied.
    # But the postcondition length($x) == 1 is NOT always true.
    my $pre = (length($x) >= 1) && (ord($x) >= 0);
    my $result = ord_implies_len1($x);
    !$pre || ($result == 1);  # pre => post
}, name => "ord_implies_len1: post holds (EXPECT FAIL)";

Property {
    ##[ x <- String(charset=>"a-z", length=>[1,1]) ]##
    my $result = ord_single_char_nonneg($x);
    $result >= 0;
}, name => "ord_single_char_nonneg: post holds";

Property {
    ##[ x <- String(charset=>"a-z", length=>[1,1]) ]##
    my $result = ord_of_first_char($x);
    $result == ord($x);
}, name => "ord_of_first_char: post holds";
