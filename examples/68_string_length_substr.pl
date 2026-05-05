# =============================================================
# Round 68: String length + substr path verification
# =============================================================
# Functions that use length() to compute indices for substr(),
# combined with conditional branches that check string properties,
# creating paths where the verifier must reason about string
# length constraints and substring extraction.

# --- Function 1: Branch on length then extract different prefix sizes ---
# Short strings get their first 2 chars, longer strings get their
# first 5 chars. Each branch uses a different substr length based on
# the length constraint established by the condition.
# sig: (Str) -> I64
# pre: length($s) >= 4 && length($s) <= 20
# post: $result >= 2 && $result <= 5
sub branch_prefix_length {
    my ($s) = @_;
    my $len = length($s);
    my $r;
    if ($len < 8) {
        my $prefix = substr($s, 0, 2);
        $r = length($prefix);
    } else {
        my $prefix = substr($s, 0, 5);
        $r = length($prefix);
    }
    return $r;
}

# --- Function 2: Substr + contains on extracted prefix ---
# Extracts a prefix, checks if it contains a pattern, then
# returns a different value per branch. Exercises cross-theory
# reasoning: string extraction feeds containment predicate.
# sig: (Str) -> I64
# pre: length($s) >= 6 && length($s) <= 12
# post: $result >= 3 && $result <= 12
sub substr_contains_branch {
    my ($s) = @_;
    my $prefix = substr($s, 0, 3);
    my $has_ab = contains($prefix, "ab");
    my $len = length($s);
    my $r;
    if ($has_ab == 1) {
        $r = length($prefix) + length($prefix);
    } else {
        $r = $len;
    }
    return $r;
}

# --- Function 3: Nested conditions on length producing varied extractions ---
# Three paths based on length ranges, each extracting a different
# fixed-size prefix and returning its length. Verifier must prove
# extraction length matches the literal used in substr.
# sig: (Str) -> I64
# pre: length($s) >= 5 && length($s) <= 15
# post: $result >= 1 && $result <= 4
sub multi_path_prefix {
    my ($s) = @_;
    my $len = length($s);
    my $r;
    if ($len <= 7) {
        my $piece = substr($s, 0, 1);
        $r = length($piece);
    } elsif ($len <= 11) {
        my $piece = substr($s, 0, 3);
        $r = length($piece);
    } else {
        my $piece = substr($s, 0, 4);
        $r = length($piece);
    }
    return $r;
}

# --- Function 4: Length arithmetic with concat then substr ---
# Concatenates strings, computes length of result, uses substr
# on the concatenation. Verifier reasons about length(a.b) = length(a)+length(b).
# sig: (Str, Str) -> I64
# pre: length($a) >= 3 && length($a) <= 5 && length($b) >= 2 && length($b) <= 4
# post: $result >= 5 && $result <= 9
sub concat_then_extract {
    my ($a, $b) = @_;
    my $combined = $a . $b;
    my $total_len = length($combined);
    my $prefix = substr($combined, 0, 4);
    my $prefix_len = length($prefix);
    my $r = $total_len - $prefix_len + $prefix_len;
    return $r;
}

# --- Function 5: Cascading length checks with contains and substr ---
# Multiple conditions involving length and contains results,
# creating 4 paths. Each path extracts a different prefix and
# computes a different bounded integer result.
# sig: (Str) -> I64
# pre: length($s) >= 6 && length($s) <= 10
# post: $result >= 1 && $result <= 10
sub cascade_length_contains {
    my ($s) = @_;
    my $len = length($s);
    my $pre = substr($s, 0, 3);
    my $has_x = contains($pre, "x");
    my $r;
    if ($len > 8) {
        if ($has_x == 1) {
            my $ext = $s . "z";
            $r = length($ext) - $len;
        } else {
            $r = $len;
        }
    } else {
        if ($has_x == 1) {
            $r = length($pre) + length($pre);
        } else {
            $r = length($pre);
        }
    }
    return $r;
}
