# =============================================================
# Round 82: Reverse + length + substr combo path stress
# =============================================================
# Functions that combine reverse(), length(), and substr() in
# sequence with conditional branches, testing cross-operation
# string reasoning where the verifier must track how reverse
# affects length and substring properties.

# --- Function 1: Reverse preserves length, branch on original ---
# Takes a string, reverses it, asserts length is preserved,
# then branches on original string length to return different values.
# sig: (Str) -> Int
# pre: length($s) >= 3 && length($s) <= 10
# post: $result >= 3 && $result <= 20
sub reverse_length_branch {
    my ($s) = @_;
    my $rev = reverse($s);
    my $rev_len = length($rev);
    my $orig_len = length($s);
    if ($orig_len > 6) {
        return $rev_len + $orig_len;
    } else {
        return $rev_len;
    }
}

# --- Function 2: Reverse + substr length reasoning ---
# Reverses a string, takes a prefix of the reversal, then
# branches on prefix length vs original length relationship.
# The verifier must know length(reverse(s)) == length(s) to
# prove that substr on the reversal is valid.
# sig: (Str) -> Int
# pre: length($s) >= 4 && length($s) <= 8
# post: $result >= 2 && $result <= 8
sub reverse_substr_branch {
    my ($s) = @_;
    my $rev = reverse($s);
    my $prefix = substr($rev, 0, 2);
    my $prefix_len = length($prefix);
    my $rev_len = length($rev);
    if ($rev_len > 5) {
        return $rev_len;
    } else {
        return $prefix_len + $rev_len;
    }
}

# --- Function 3: Chained reverse + length arithmetic ---
# Uses length(reverse(s)) in arithmetic with another integer,
# then branches multiple times on computed values.
# sig: (Str, Int) -> Int
# pre: length($s) >= 3 && length($s) <= 7 && $k >= 1 && $k <= 5
# post: $result >= 1 && $result <= 12
sub reverse_length_arithmetic {
    my ($s, $k) = @_;
    my $rev = reverse($s);
    my $rlen = length($rev);
    my $sum = $rlen + $k;
    my $diff = $rlen - $k;
    if ($sum > 8) {
        if ($diff > 3) {
            return $diff;
        } else {
            return $sum - $rlen;
        }
    } else {
        if ($diff > 0) {
            return $diff;
        } else {
            return $k;
        }
    }
}

# --- Function 4: Multiple reverses with substr comparison ---
# Reverses a string, takes a substr of both original and reversed,
# uses their lengths (which are fixed by the substr args) in
# branching logic. Verifier must reason about substr producing
# a string of known length and reverse preserving length.
# sig: (Str) -> Int
# pre: length($s) >= 5 && length($s) <= 9
# post: $result >= 3 && $result <= 9
sub double_reverse_substr {
    my ($s) = @_;
    my $rev = reverse($s);
    my $orig_sub = substr($s, 0, 3);
    my $rev_sub = substr($rev, 0, 3);
    my $orig_sub_len = length($orig_sub);
    my $rev_sub_len = length($rev_sub);
    my $full_len = length($rev);
    if ($full_len > 7) {
        return $full_len;
    } elsif ($full_len > 5) {
        return $orig_sub_len + $rev_sub_len;
    } else {
        return $full_len;
    }
}

# --- Function 5: Nested conditions with reverse + length + die ---
# Complex branching using reverse length preservation with die
# to prune impossible paths. The verifier must use the axiom
# that length(reverse(s)) == length(s) to prove die is unreachable.
# sig: (Str, Int) -> Int
# pre: length($s) >= 4 && length($s) <= 6 && $m >= 1 && $m <= 3
# post: $result >= 1 && $result <= 12
sub reverse_die_prune {
    my ($s, $m) = @_;
    my $rev = reverse($s);
    my $rlen = length($rev);
    my $slen = length($s);
    die "impossible" if ($rlen != $slen);
    my $val = $rlen * $m;
    if ($val > 12) {
        return 12;
    } elsif ($val > 6) {
        return $val;
    } else {
        return $val;
    }
}
