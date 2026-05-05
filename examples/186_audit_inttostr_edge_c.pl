# Round 186 audit part C: IntToStr string comparison soundness
# ============================================================
# Probing whether Z3 int.to.str produces correct string equality/ordering

# --- PROBE: "" . $n ne "" . $m when $n != $m ---
# TRUE in Perl: different integers stringify differently (in range)
# sig: (I64, I64) -> I64
# pre: $n >= 0 && $n <= 99 && $m >= 0 && $m <= 99 && $n != $m
# post: $result == 1
sub distinct_ints_distinct_strs {
    my ($n, $m) = @_;
    my $sn = "" . $n;
    my $sm = "" . $m;
    if ($sn eq $sm) { return 0; }
    return 1;
}

# --- PROBE: "" . $n eq "" . $n (same integer) ---
# sig: (I64) -> I64
# pre: $n >= -99 && $n <= 99
# post: $result == 1
sub same_int_same_str {
    my ($n) = @_;
    my $s1 = "" . $n;
    my $s2 = "" . $n;
    if ($s1 eq $s2) { return 1; }
    return 0;
}

# --- PROBE: int("" . $n) == $n for small range ---
# This roundtrip should hold for small integers.
# sig: (I64) -> I64
# pre: $n >= -99 && $n <= 99
# post: $result == $n
sub roundtrip_small {
    my ($n) = @_;
    my $s = "" . $n;
    return int($s);
}

# --- PROBE: "" . 0 ne "" (zero is not empty string) ---
# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 1
sub zero_not_empty {
    my ($n) = @_;
    my $s = "" . $n;
    if ($s eq "") { return 0; }
    return 1;
}

# --- PROBE: length("" . $n) for wide range: always >= 1 ---
# sig: (I64) -> I64
# pre: $n >= -9999 && $n <= 9999
# post: $result >= 1
sub wide_range_len_ge1 {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE (FALSE): can the tool prove length("" . $n) <= 4 for all $n in [-9999,9999]? ---
# This is FALSE: $n = -1000 gives "-1000" which has length 5.
# If verified, that's UNSOUND.
# sig: (I64) -> I64
# pre: $n >= -9999 && $n <= 9999
# post: $result <= 4
sub wide_range_len_le4_false {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE: length("" . $n) <= 5 for $n in [-9999,9999] ---
# TRUE: max is "-9999" = 5 chars.
# sig: (I64) -> I64
# pre: $n >= -9999 && $n <= 9999
# post: $result <= 5
sub wide_range_len_le5 {
    my ($n) = @_;
    return length("" . $n);
}
