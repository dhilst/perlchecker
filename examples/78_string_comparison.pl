# =============================================================
# Round 78: String comparison operator path stress
# =============================================================
# Functions using string comparison operators (eq, ne, lt, gt, le, ge)
# in complex conditional chains, creating paths where Z3 must reason
# about string ordering and equality across multiple branches.
# =============================================================

# --- Function 1: Classify string position relative to three boundaries ---
# Given a string and three boundary strings, returns which region it falls in.
# Creates 4 distinct paths based on lt/ge comparisons.
# sig: (Str, Str, Str, Str) -> Int
# pre: $lo lt $mid && $mid lt $hi
# post: $result >= 1 && $result <= 4
sub classify_region {
    my ($s, $lo, $mid, $hi) = @_;
    if ($s lt $lo) {
        return 1;
    } elsif ($s lt $mid) {
        return 2;
    } elsif ($s lt $hi) {
        return 3;
    } else {
        return 4;
    }
}

# --- Function 2: Nested equality and ordering checks ---
# Combines eq/ne with lt/gt in nested conditionals. Classifies a pair
# of strings by their equality and relative ordering.
# sig: (Str, Str) -> Int
# post: $result >= 1 && $result <= 3
sub compare_pair {
    my ($a, $b) = @_;
    if ($a eq $b) {
        return 1;
    } elsif ($a lt $b) {
        return 2;
    } else {
        return 3;
    }
}

# --- Function 3: Multi-string sorting classifier ---
# Given three strings, determines which of the 6 possible orderings
# they could have (or equal cases), returning a category code.
# Uses multiple comparisons creating many branch paths.
# sig: (Str, Str, Str) -> Int
# post: $result >= 1 && $result <= 6
sub triple_order {
    my ($x, $y, $z) = @_;
    if ($x le $y) {
        if ($y le $z) {
            return 1;
        } elsif ($x le $z) {
            return 2;
        } else {
            return 3;
        }
    } else {
        if ($x le $z) {
            return 4;
        } elsif ($y le $z) {
            return 5;
        } else {
            return 6;
        }
    }
}

# --- Function 4: Range check with equality edge cases ---
# Checks if string is in range [lo, hi] using le/ge, and further
# classifies boundary vs interior positions. The combination of
# ge/le/eq creates multiple path splits.
# sig: (Str, Str, Str) -> Int
# pre: $lo lt $hi
# post: $result >= 0 && $result <= 3
sub range_classify {
    my ($s, $lo, $hi) = @_;
    if ($s lt $lo) {
        return 0;
    } elsif ($s eq $lo) {
        return 1;
    } elsif ($s lt $hi) {
        return 2;
    } elsif ($s eq $hi) {
        return 2;
    } else {
        return 3;
    }
}

# --- Function 5: Chained string comparisons with accumulator ---
# Uses ne checks against multiple reference strings to count
# how many references a string differs from. Creates exponential
# path explosion through independent ne branches.
# sig: (Str, Str, Str, Str) -> Int
# post: $result >= 0 && $result <= 3
sub count_differences {
    my ($s, $r1, $r2, $r3) = @_;
    my $count = 0;
    if ($s ne $r1) {
        $count = $count + 1;
    }
    if ($s ne $r2) {
        $count = $count + 1;
    }
    if ($s ne $r3) {
        $count = $count + 1;
    }
    return $count;
}
