# =============================================================
# Round 54: String + integer mixed path verification
# =============================================================
# Exercises cross-theory (string + integer) path reasoning by
# combining string operations (contains, starts_with, length,
# substr, ord, chr) with integer arithmetic in branching
# conditions, creating paths where the verifier must reason
# about both Z3 string theory and integer arithmetic simultaneously.

# --- Length-gated branching with arithmetic ---
# Branch on string length, then compute integer results differently.
# The verifier must track length constraints through both paths.
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 20
# post: $result >= 1 && $result <= 20
sub length_category {
    my ($s) = @_;
    my $len = length($s);
    if ($len > 10) {
        my $r = $len - ($len - 10);
        return $r + ($len - 10);
    } else {
        return $len;
    }
}

# --- contains() result drives integer arithmetic paths ---
# Uses contains() as a boolean guard, computing different bounded
# integer results in each branch. Cross-theory: string containment
# determines which arithmetic path is taken.
# sig: (Str, Str) -> I64
# pre: length($s) >= 3 && length($sub) >= 1 && length($sub) <= 3
# post: $result >= 0 && $result <= 1
sub contains_to_flag {
    my ($s, $sub) = @_;
    my $c = contains($s, $sub);
    if ($c == 1) {
        my $r = 2 - 1;
        return $r;
    } else {
        my $r = 1 - 1;
        return $r;
    }
}

# --- ord/chr bridge with integer arithmetic ---
# Uses ord() to convert a character to integer, performs arithmetic,
# then verifies the result stays in bounds. Cross-theory: string
# character extraction feeds integer computation.
# sig: (I64) -> I64
# pre: $code >= 65 && $code <= 90
# post: $result >= 97 && $result <= 122
sub upper_to_lower_code {
    my ($code) = @_;
    my $ch = chr($code);
    my $val = ord($ch);
    my $lower = $val + 32;
    return $lower;
}

# --- starts_with guard combined with length arithmetic ---
# Uses starts_with() to branch, then uses length in arithmetic.
# The verifier must combine string prefix knowledge with length bounds.
# sig: (Str) -> I64
# pre: length($s) >= 5 && starts_with($s, "abc") == 1
# post: $result >= 5
sub prefix_aware_length {
    my ($s) = @_;
    my $len = length($s);
    my $sw = starts_with($s, "abc");
    if ($sw == 1) {
        return $len;
    } else {
        return $len + 1;
    }
}

# --- Multi-condition: length + contains + arithmetic ---
# Combines multiple string predicates with integer logic.
# Creates 4 paths based on (length > 8) x (contains pattern).
# All paths must produce a bounded result.
# sig: (Str, Str) -> I64
# pre: length($s) >= 1 && length($s) <= 15 && length($pat) >= 1 && length($pat) <= 3
# post: $result >= 0 && $result <= 30
sub multi_string_int_paths {
    my ($s, $pat) = @_;
    my $len = length($s);
    my $has = contains($s, $pat);
    if ($len > 8) {
        if ($has == 1) {
            return $len + $len;
        } else {
            return $len;
        }
    } else {
        if ($has == 1) {
            return $len + 1;
        } else {
            return $len;
        }
    }
}
