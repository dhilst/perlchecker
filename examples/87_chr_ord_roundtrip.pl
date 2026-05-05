# =============================================================
# Round 87: Chr/ord roundtrip path verification
# =============================================================
# Functions that convert between characters and their numeric codes
# using chr()/ord(), verify roundtrip properties, and branch on
# character code ranges to classify characters. Exercises path
# expansion through character-range conditionals.

# --- Function 1: ord(chr($n)) roundtrip in conditional context ---
# Verifies the fundamental roundtrip property: for a valid ASCII code,
# ord(chr(n)) == n. Uses conditional to branch on the range and still
# guarantee the roundtrip identity holds on both paths.
# sig: (I64) -> I64
# pre: $n >= 48 && $n <= 122
# post: $result == $n
sub ord_chr_roundtrip {
    my ($n) = @_;
    my $ch = chr($n);
    my $back = ord($ch);
    if ($n >= 65) {
        return $back;
    } else {
        return $back;
    }
}

# --- Function 2: Character classification by ord() range ---
# Classifies a character code into digit (48-57), uppercase (65-90),
# lowercase (97-122), or other. Returns a distinct category code for
# each range. Creates 4 paths through the elsif chain.
# sig: (I64) -> I64
# pre: $n >= 48 && $n <= 122
# post: $result >= 1 && $result <= 4
sub classify_char_code {
    my ($n) = @_;
    my $ch = chr($n);
    my $code = ord($ch);
    if ($code >= 97 && $code <= 122) {
        return 1;
    } elsif ($code >= 65 && $code <= 90) {
        return 2;
    } elsif ($code >= 48 && $code <= 57) {
        return 3;
    } else {
        return 4;
    }
}

# --- Function 3: Case conversion via ord arithmetic ---
# Converts an uppercase letter code to lowercase by adding 32.
# Verifies the result is in the lowercase ASCII range.
# Uses chr/ord roundtrip combined with arithmetic.
# sig: (I64) -> I64
# pre: $n >= 65 && $n <= 90
# post: $result >= 97 && $result <= 122
sub to_lower_via_ord {
    my ($n) = @_;
    my $ch = chr($n);
    my $code = ord($ch);
    my $lower_code = $code + 32;
    return $lower_code;
}

# --- Function 4: Branching case conversion with range check ---
# Takes a code that could be upper or lower case. If uppercase,
# converts to lowercase code; if already lowercase, returns as-is.
# Verifies the output is always in the lowercase range.
# sig: (I64) -> I64
# pre: $n >= 65 && $n <= 122 && ($n <= 90 || $n >= 97)
# post: $result >= 97 && $result <= 122
sub normalize_to_lower {
    my ($n) = @_;
    my $ch = chr($n);
    my $code = ord($ch);
    if ($code >= 65 && $code <= 90) {
        my $lower = $code + 32;
        return $lower;
    } else {
        return $code;
    }
}

# --- Function 5: Multi-branch ord arithmetic with accumulator ---
# Takes two character codes, classifies each (digit vs letter),
# and accumulates a score based on the classification of each.
# Creates 4 paths (2 conditions x 2 branches each).
# sig: (I64, I64) -> I64
# pre: $a >= 48 && $a <= 90 && $b >= 48 && $b <= 90
# post: $result >= 2 && $result <= 20
sub dual_char_score {
    my ($a, $b) = @_;
    my $ca = chr($a);
    my $cb = chr($b);
    my $oa = ord($ca);
    my $ob = ord($cb);
    my $score = 0;
    if ($oa >= 65) {
        $score += 10;
    } else {
        $score += 1;
    }
    if ($ob >= 65) {
        $score += 10;
    } else {
        $score += 1;
    }
    return $score;
}
