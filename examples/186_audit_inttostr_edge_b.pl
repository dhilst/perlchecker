# Round 186 audit part B: IntToStr soundness probes
# ==================================================
# These test for potential unsoundness -- properties that are FALSE
# in Perl but might be wrongly VERIFIED by the tool.

# --- PROBE 1: length("" . 100) should be 3, not 2 ---
# If the tool wrongly verified this as 2, that's unsound.
# sig: (I64) -> I64
# pre: $n == 100
# post: $result == 3
sub len_100 {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE 2: False claim about zero. Should be counterexample. ---
# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 0
sub zero_is_empty_string {
    my ($n) = @_;
    my $s = "" . $n;
    return length($s);
}

# --- PROBE 3: length("" . $n) >= 1 for all $n in range ---
# This is TRUE in Perl; should be verified.
# sig: (I64) -> I64
# pre: $n >= -100 && $n <= 100
# post: $result >= 1
sub strlen_at_least_1 {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE 4: For non-negative n, length("" . $n) >= 1 ---
# TRUE in Perl. Check if int.to.str(n) is never empty for n >= 0.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 999
# post: $result >= 1
sub strlen_nonneg_at_least_1 {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE 5: For negative n, length("" . $n) >= 2 ---
# TRUE in Perl (always "-" + at least one digit).
# sig: (I64) -> I64
# pre: $n >= -999 && $n <= -1
# post: $result >= 2
sub strlen_neg_at_least_2 {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE 6: FALSE claim. "" . 1 should NOT equal "01". ---
# If verified, that's unsound.
# sig: (I64) -> I64
# pre: $n == 1
# post: $result == 0
sub one_is_not_zero_one {
    my ($n) = @_;
    my $s = "" . $n;
    if ($s eq "01") { return 1; }
    return 0;
}

# --- PROBE 7: Three-digit length range check ---
# sig: (I64) -> I64
# pre: $n >= 100 && $n <= 999
# post: $result == 3
sub three_digit_len {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE 8: Negative three-digit length ---
# sig: (I64) -> I64
# pre: $n >= -999 && $n <= -100
# post: $result == 4
sub neg_three_digit_len {
    my ($n) = @_;
    return length("" . $n);
}
