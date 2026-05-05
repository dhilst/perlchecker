# =============================================================
# Round 73: All-builtins-in-one path stress
# =============================================================
# Functions that use min, max, abs, length, substr, contains,
# starts_with, ord, chr, and reverse in a sequence of conditional
# operations, creating paths where the verifier must track multiple
# Z3 theories simultaneously (string + integer + ITE).

# --- Function 1: Mega builtin cascade ---
# Uses length, abs, min, max, contains, starts_with, substr, reverse
# in sequence. Each builtin result feeds into the next condition,
# interleaving string and integer theories.
# sig: (Str, I64) -> I64
# pre: length($s) >= 4 && length($s) <= 10 && $n >= -5 && $n <= 5
# post: $result >= 0 && $result <= 20
sub mega_builtin_cascade {
    my ($s, $n) = @_;
    my $len = length($s);
    my $an = abs($n);
    my $lo = min($an, $len);
    my $hi = max($an, $len);
    my $r = 0;
    if ($lo > 3) {
        my $pre = substr($s, 0, 3);
        my $has = contains($pre, "a");
        if ($has == 1) {
            $r = $hi - $lo + 2;
        } else {
            $r = $hi - $lo + 1;
        }
    } else {
        my $sw = starts_with($s, "x");
        if ($sw == 1) {
            $r = $len - $lo;
        } else {
            $r = $len - $lo + 1;
        }
    }
    return $r;
}

# --- Function 2: ord/chr bridge with string predicates ---
# Takes an integer code, converts to chr, checks properties of
# the resulting character via string builtins, uses abs/min/max
# on intermediate values.
# sig: (I64) -> I64
# pre: $code >= 97 && $code <= 122
# post: $result >= 1 && $result <= 26
sub ord_chr_bridge {
    my ($code) = @_;
    my $ch = chr($code);
    my $val = ord($ch);
    my $offset = $val - 96;
    my $bounded = min($offset, 26);
    my $final = max($bounded, 1);
    if ($final > 13) {
        my $diff = abs($final - 26);
        my $r = min($diff + 1, 26);
        return $r;
    } else {
        return $final;
    }
}

# --- Function 3: String theory cross-pollination ---
# Combines reverse, length, substr, contains, starts_with in
# branching logic. The verifier must reason that reverse preserves
# length and that substr on a bounded-length string yields bounded length.
# sig: (Str) -> I64
# pre: length($s) >= 5 && length($s) <= 8
# post: $result >= 1 && $result <= 8
sub string_cross_theory {
    my ($s) = @_;
    my $rev = reverse($s);
    my $len = length($rev);
    my $prefix = substr($s, 0, 3);
    my $has_ab = contains($prefix, "ab");
    my $sw = starts_with($rev, "z");
    my $r;
    if ($has_ab == 1) {
        if ($sw == 1) {
            $r = $len;
        } else {
            $r = $len - 1;
        }
    } else {
        if ($sw == 1) {
            $r = length($prefix) + 1;
        } else {
            $r = length($prefix);
        }
    }
    die "impossible" if ($r < 1);
    return $r;
}

# --- Function 4: Full interleave with die-based path pruning ---
# Uses all builtins in a complex chain with die statements pruning
# impossible paths. The verifier must track that certain conditions
# are impossible given the preconditions and earlier computations.
# sig: (Str, I64, I64) -> I64
# pre: length($s) >= 3 && length($s) <= 6 && $x >= 1 && $x <= 10 && $y >= 1 && $y <= 10
# post: $result >= 1 && $result <= 20
sub full_interleave_pruned {
    my ($s, $x, $y) = @_;
    my $len = length($s);
    my $lo = min($x, $y);
    my $hi = max($x, $y);
    my $d = abs($x - $y);
    my $prefix = substr($s, 0, 2);
    my $has_q = contains($prefix, "q");
    my $rev = reverse($s);
    my $rev_len = length($rev);
    die "impossible" if ($rev_len != $len);
    my $r;
    if ($d > 5) {
        if ($has_q == 1) {
            $r = $hi - $lo + $len;
        } else {
            $r = $hi - $lo;
        }
    } else {
        my $sw = starts_with($s, "a");
        if ($sw == 1) {
            $r = $lo + $len;
        } else {
            $r = $lo + length($prefix);
        }
    }
    die "impossible" if ($r < 1);
    return $r;
}
