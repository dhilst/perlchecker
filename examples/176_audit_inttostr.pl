# Round 176 audit: IntToStr (int-to-string) for negative numbers
# ================================================================
# Z3's int.to.str is undefined for negative integers (returns "").
# The tool's FromInt encoding handles this via ITE: for n<0 it
# produces "-" ++ int_to_str(-n). This is correct for values in
# Perl's exact integer range [-2^63, 2^64-1].
#
# BUG (fixed): When the integer overflows this range (e.g.,
# -(~5) = -18446744073709551610), Perl converts to a float and
# stringifies with scientific notation:
#   Perl: "" . -(~5) = "-1.84467440737096e+19"
#   Tool: "" . -(~5) = "-18446744073709551610" (WRONG before fix)
#
# Fix: model out-of-range FromInt as a fresh unconstrained string,
# so the solver cannot prove properties about overflowed values.

# --- Previously unsound: now correctly rejected as counterexample ---
# In Perl: -(~5) overflows i64, becomes float, stringifies as
# "-1.84467440737096e+19" (NOT "-18446744073709551610").
# sig: (Int) -> Int
# pre: $n == 5
# post: $result == 1
sub inttostr_overflow_unsound {
    my ($n) = @_;
    my $val = -(~$n);
    my $s = "" . $val;
    if ($s eq "-18446744073709551610") {
        return 1;
    }
    return 0;
}

# --- SOUND: within i64 range, negative stringify is correct ---
# sig: (Int) -> Int
# pre: $n >= -999 && $n <= -1
# post: $result == $n
sub inttostr_neg_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    my $back = int($s);
    return $back;
}

# --- SOUND: stringified negative always starts with "-" ---
# sig: (Int) -> Int
# pre: $n >= -99 && $n <= -1
# post: $result == 0
sub inttostr_neg_starts_minus {
    my ($n) = @_;
    my $s = "" . $n;
    return index($s, "-");
}

use strict;
use warnings;

sub run_tests {
    # inttostr_overflow_unsound: in Perl, -(~5) overflows to float
    my $n = 5;
    my $val = -(~$n);
    my $s = "" . $val;
    # Perl produces "-1.84467440737096e+19", NOT "-18446744073709551610"
    die "FAIL overflow" if $s eq "-18446744073709551610";
    print "PASS overflow: got [$s]\n";

    # inttostr_neg_roundtrip
    for my $i (-999..-1) {
        my $back = int("" . $i);
        die "FAIL roundtrip $i" unless $back == $i;
    }
    print "PASS neg_roundtrip\n";

    # inttostr_neg_starts_minus
    for my $i (-99..-1) {
        die "FAIL starts_minus $i" unless index("" . $i, "-") == 0;
    }
    print "PASS neg_starts_minus\n";
}

run_tests();
