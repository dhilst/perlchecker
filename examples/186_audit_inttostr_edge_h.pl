# Round 186 audit part H: IntToStr - can Z3 wrongly prove digit properties?
# ===========================================================================

# --- PROBE: For $n in [0,9], "" . $n should equal chr(48 + $n) ---
# Because "0" = chr(48), "1" = chr(49), ..., "9" = chr(57).
# This tests whether Z3's int.to.str matches Perl's chr for digits.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 9
# post: $result == 1
sub digit_matches_chr {
    my ($n) = @_;
    my $s = "" . $n;
    my $c = chr(48 + $n);
    if ($s eq $c) { return 1; }
    return 0;
}

# --- PROBE: "" . 10 has exactly 2 characters ---
# sig: (I64) -> I64
# pre: $n == 10
# post: $result == 2
sub ten_len {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE: "" . $n for symbolic $n in [0,9] is a single char ---
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 9
# post: $result == 1
sub single_char_for_digit {
    my ($n) = @_;
    my $s = "" . $n;
    return length($s);
}

# --- PROBE: Can the tool prove "" . 0 equals the result of chr(48)? ---
# chr(48) = "0" in ASCII. This should be verifiable.
# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 1
sub zero_equals_chr48 {
    my ($n) = @_;
    my $s = "" . $n;
    if ($s eq chr(48)) { return 1; }
    return 0;
}
