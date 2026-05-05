# =============================================================
# Domain: String Comparison Operators (lt, gt, le, ge)
# =============================================================
# Perl uses lt/gt/le/ge for lexicographic string comparison.
# These are extremely common in real Perl code for sorting,
# validation, and range checks on string data.
#
# BOUNDARY PUSH: String comparison operators lt/gt/le/ge
# returning boolean values for use in conditions.
# =============================================================

# --- Check if a string comes before another lexicographically ---
# sig: (Str, Str) -> I64
# post: $result == 0 || $result == 1
sub is_before {
    my ($a, $b) = @_;
    if ($a lt $b) {
        return 1;
    }
    return 0;
}

# --- Check if two strings are in sorted order (a <= b <= c) ---
# sig: (Str, Str, Str) -> I64
# post: $result == 0 || $result == 1
sub is_sorted_triple {
    my ($a, $b, $c) = @_;
    if ($a le $b && $b le $c) {
        return 1;
    }
    return 0;
}

# --- Return 1 if string is in [lo, hi] range lexicographically ---
# sig: (Str, Str, Str) -> I64
# pre: $lo le $hi
# post: $result == 0 || $result == 1
sub in_str_range {
    my ($s, $lo, $hi) = @_;
    if ($s ge $lo && $s le $hi) {
        return 1;
    }
    return 0;
}
