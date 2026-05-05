# Round 186 audit part E: IntToStr with computed integer expressions
# =================================================================
# Test that FromInt works correctly for expressions, not just variables.

# --- PROBE: "" . ($a + $b) should match Perl semantics ---
# sig: (I64, I64) -> Str
# pre: $a == 3 && $b == 7
# post: $result eq "10"
sub sum_to_str {
    my ($a, $b) = @_;
    return "" . ($a + $b);
}

# --- PROBE: "" . ($a - $b) for negative result ---
# sig: (I64, I64) -> Str
# pre: $a == 3 && $b == 7
# post: $result eq "-4"
sub diff_neg_to_str {
    my ($a, $b) = @_;
    return "" . ($a - $b);
}

# --- PROBE: "" . ($a * $b) ---
# sig: (I64, I64) -> Str
# pre: $a == 6 && $b == 7
# post: $result eq "42"
sub product_to_str {
    my ($a, $b) = @_;
    return "" . ($a * $b);
}

# --- KEY PROBE: "" . (0 - 0) should be "0", not "-0" ---
# Perl doesn't have negative zero for integers.
# sig: (I64, I64) -> Str
# pre: $a == 0 && $b == 0
# post: $result eq "0"
sub zero_minus_zero_to_str {
    my ($a, $b) = @_;
    return "" . ($a - $b);
}

# --- KEY PROBE: length("" . ($n * 0)) should be 1 ---
# $n * 0 = 0, stringify to "0", length 1
# sig: (I64) -> I64
# pre: $n >= -99 && $n <= 99
# post: $result == 1
sub multiply_by_zero_len {
    my ($n) = @_;
    my $s = "" . ($n * 0);
    return length($s);
}

# --- KEY PROBE: can int.to.str produce leading zeros? ---
# It should NOT. "" . 5 should be "5", not "05".
# If int.to.str(5) == "05", then length would be 2 (WRONG).
# This is implicitly tested by single_digit_len but let's be explicit.
# sig: (I64) -> I64
# pre: $n == 5
# post: $result == 0
sub no_leading_zero {
    my ($n) = @_;
    my $s = "" . $n;
    # substr($s, 0, 1) should NOT be "0" for $n > 0
    if (substr($s, 0, 1) eq "0") { return 1; }
    return 0;
}

# --- PROBE: Leading zeros for any positive integer? ---
# For any n in [1..99], the first char should never be "0".
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 99
# post: $result == 0
sub no_leading_zero_range {
    my ($n) = @_;
    my $s = "" . $n;
    if (substr($s, 0, 1) eq "0") { return 1; }
    return 0;
}
