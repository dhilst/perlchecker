# Round 186 audit: IntToStr edge cases — zero, negatives, length
# =================================================================
# Verify Z3's int.to.str encoding matches Perl for:
# - zero: "" . 0 => "0" (length 1)
# - small negatives: "" . (-1) => "-1", "" . (-5) => "-5"
# - length of stringified ints across sign boundary
# - roundtrip: int("" . $n) == $n

# --- Zero stringify ---
# sig: (I64) -> Str
# pre: $n == 0
# post: $result eq "0"
sub zero_stringify {
    my ($n) = @_;
    return "" . $n;
}

# --- Zero length ---
# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 1
sub zero_length {
    my ($n) = @_;
    return length("" . $n);
}

# --- Negative one stringify ---
# sig: (I64) -> Str
# pre: $n == -1
# post: $result eq "-1"
sub neg1_stringify {
    my ($n) = @_;
    return "" . $n;
}

# --- Negative five stringify ---
# sig: (I64) -> Str
# pre: $n == -5
# post: $result eq "-5"
sub neg5_stringify {
    my ($n) = @_;
    return "" . $n;
}

# --- Length of single-digit negatives: [-9..-1] should be 2 ---
# sig: (I64) -> I64
# pre: $n >= -9 && $n <= -1
# post: $result == 2
sub neg_single_digit_len {
    my ($n) = @_;
    return length("" . $n);
}

# --- Length of two-digit negatives: [-99..-10] should be 3 ---
# sig: (I64) -> I64
# pre: $n >= -99 && $n <= -10
# post: $result == 3
sub neg_two_digit_len {
    my ($n) = @_;
    return length("" . $n);
}

# --- Length of single-digit positives: [0..9] should be 1 ---
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 9
# post: $result == 1
sub pos_single_digit_len {
    my ($n) = @_;
    return length("" . $n);
}

# --- Roundtrip: int("" . $n) == $n for small negatives ---
# sig: (I64) -> I64
# pre: $n >= -99 && $n <= -1
# post: $result == $n
sub neg_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    return int($s);
}

# --- Roundtrip: int("" . 0) == 0 ---
# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 0
sub zero_roundtrip {
    my ($n) = @_;
    my $s = "" . $n;
    return int($s);
}

# --- Concat with string prefix: "val:" . (-1) should be "val:-1" ---
# sig: (I64) -> Str
# pre: $n == -1
# post: $result eq "val:-1"
sub prefix_neg_stringify {
    my ($n) = @_;
    return "val:" . $n;
}

use strict;
use warnings;

sub run_tests {
    die "FAIL zero_stringify" unless ("" . 0) eq "0";
    die "FAIL zero_length" unless length("" . 0) == 1;
    die "FAIL neg1_stringify" unless ("" . -1) eq "-1";
    die "FAIL neg5_stringify" unless ("" . -5) eq "-5";
    for my $n (-9..-1) {
        die "FAIL neg_single_digit_len $n" unless length("" . $n) == 2;
    }
    for my $n (-99..-10) {
        die "FAIL neg_two_digit_len $n" unless length("" . $n) == 3;
    }
    for my $n (0..9) {
        die "FAIL pos_single_digit_len $n" unless length("" . $n) == 1;
    }
    for my $n (-99..-1) {
        die "FAIL neg_roundtrip $n" unless int("" . $n) == $n;
    }
    die "FAIL zero_roundtrip" unless int("" . 0) == 0;
    die "FAIL prefix_neg" unless ("val:" . -1) eq "val:-1";
    print "ALL PASS\n";
}

run_tests();
