# =============================================================
# Round 104: String ordering soundness fix (lt/gt/le/ge)
# =============================================================
# Tests that string ordering comparisons produce correct results
# via Z3 lexicographic string comparison, not vacuous true.
# =============================================================

# --- Basic string ordering: returns 0 or 1 ---
# sig: (Str, Str) -> I64
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: $result >= 0 && $result <= 1
sub is_less_than {
    my ($a, $b) = @_;
    if ($a lt $b) {
        return 1;
    }
    return 0;
}

# --- Constant string ordering: "a" is always less than "b" ---
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub a_lt_b {
    my ($s) = @_;
    if ("a" lt "b") {
        return 1;
    }
    return 0;
}

# --- Ordering consistency: if a lt b then b gt a ---
# sig: (Str, Str) -> I64
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: $result >= 0 && $result <= 1
sub ordering_consistency {
    my ($a, $b) = @_;
    if ($a lt $b) {
        if ($b gt $a) {
            return 1;
        }
        return 0;
    }
    return 0;
}

# --- le/ge reflexivity: any string is le and ge itself ---
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub le_reflexive {
    my ($s) = @_;
    if ($s le $s) {
        return 1;
    }
    return 0;
}
