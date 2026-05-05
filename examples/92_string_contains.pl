# =============================================================
# Round 92: String contains + conditional path stress
# =============================================================
# Functions using contains(), starts_with(), ends_with() results
# (0/1) as branch conditions, creating paths where string property
# checks drive control flow and the verifier must reason about
# string containment implications.

# --- Function 1: Branch on contains result, different string ops per branch ---
# If the string contains "ab", concatenate a suffix and return length;
# otherwise extract a prefix via substr and return its length. The
# verifier must track string length through both paths.
# sig: (Str) -> I64
# pre: length($s) >= 5 && length($s) <= 12
# post: $result >= 3 && $result <= 15
sub contains_branch_ops {
    my ($s) = @_;
    my $has_ab = contains($s, "ab");
    my $r;
    if ($has_ab == 1) {
        my $ext = $s . "end";
        $r = length($ext);
    } else {
        my $pre = substr($s, 0, 3);
        $r = length($pre);
    }
    return $r;
}

# --- Function 2: Chain starts_with + ends_with creating 4 paths ---
# Tests both prefix and suffix conditions independently, creating
# 4 distinct paths (both/start-only/end-only/neither). Each path
# computes a different bounded result from string length.
# sig: (Str) -> I64
# pre: length($s) >= 4 && length($s) <= 10
# post: $result >= 1 && $result <= 12
sub four_path_prefix_suffix {
    my ($s) = @_;
    my $sp = starts_with($s, "he");
    my $ep = ends_with($s, "lo");
    my $len = length($s);
    my $r;
    if ($sp == 1 && $ep == 1) {
        $r = $len + 2;
    } elsif ($sp == 1) {
        $r = $len + 1;
    } elsif ($ep == 1) {
        $r = $len - 1;
    } else {
        $r = $len - 3;
    }
    return $r;
}

# --- Function 3: contains result used in arithmetic (counting patterns) ---
# Checks three different substrings for containment, sums the
# results (each 0 or 1) to get a count of matching patterns.
# The verifier must reason that each contains returns 0 or 1.
# sig: (Str) -> I64
# pre: length($s) >= 6 && length($s) <= 12
# post: $result >= 0 && $result <= 3
sub count_patterns {
    my ($s) = @_;
    my $c1 = contains($s, "ab");
    my $c2 = contains($s, "cd");
    my $c3 = contains($s, "ef");
    my $r = $c1 + $c2 + $c3;
    return $r;
}

# --- Function 4: String property checks combined with length constraints ---
# Uses contains + starts_with + length in nested conditions. The
# outer branch splits on length comparison, the inner branches
# split on string property checks, creating 6 total paths.
# sig: (Str) -> I64
# pre: length($s) >= 5 && length($s) <= 10
# post: $result >= 2 && $result <= 13
sub property_length_combo {
    my ($s) = @_;
    my $len = length($s);
    my $has_x = contains($s, "x");
    my $sw_a = starts_with($s, "a");
    my $r;
    if ($len > 7) {
        if ($has_x == 1) {
            $r = $len + $has_x + $sw_a;
        } else {
            $r = $len - $has_x;
        }
    } else {
        if ($sw_a == 1) {
            $r = $len + $sw_a;
        } elsif ($has_x == 1) {
            $r = $len - 2 + $has_x;
        } else {
            $r = $len - 3;
        }
    }
    return $r;
}

# --- Function 5: Cascading containment with substr extraction ---
# Extracts a prefix, checks contains on the prefix, then uses
# that result to decide further string operations. Chains
# multiple containment checks on different substrings of the input.
# sig: (Str) -> I64
# pre: length($s) >= 8 && length($s) <= 14
# post: $result >= 2 && $result <= 14
sub cascading_contains_substr {
    my ($s) = @_;
    my $front = substr($s, 0, 4);
    my $has_ab = contains($front, "ab");
    my $ew = ends_with($s, "ed");
    my $len = length($s);
    my $r;
    if ($has_ab == 1 && $ew == 1) {
        my $back = substr($s, 0, 6);
        $r = length($back) + length($front);
    } elsif ($has_ab == 1) {
        $r = length($front) + $ew + 1;
    } elsif ($ew == 1) {
        $r = $len - length($front) + 2;
    } else {
        $r = $len - length($front);
    }
    return $r;
}
