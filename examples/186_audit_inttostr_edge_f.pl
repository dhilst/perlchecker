# Round 186 audit part F: IntToStr length with wider symbolic ranges
# ===================================================================

# --- PROBE: length("" . $n) <= 12 for $n in [-99999999999, 99999999999] ---
# Max positive: "99999999999" = 11 chars. Max negative: "-99999999999" = 12 chars.
# sig: (I64) -> I64
# pre: $n >= -99999999999 && $n <= 99999999999
# post: $result <= 12
sub wide_max_strlen {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE: length("" . $n) >= 1 for $n in [-99999999999, 99999999999] ---
# sig: (I64) -> I64
# pre: $n >= -99999999999 && $n <= 99999999999
# post: $result >= 1
sub wide_min_strlen {
    my ($n) = @_;
    return length("" . $n);
}

# --- PROBE (FALSE): length("" . $n) <= 11 for $n in [-99999999999, 99999999999] ---
# FALSE: "-99999999999" has 12 chars. Should be counterexample.
# sig: (I64) -> I64
# pre: $n >= -99999999999 && $n <= 99999999999
# post: $result <= 11
sub wide_max_strlen_wrong {
    my ($n) = @_;
    return length("" . $n);
}
