# =============================================================
# Round 75: Shift + mask + conditional path stress
# =============================================================
# Functions implementing bit manipulation patterns (extract bit field,
# test bit, set/clear bit) using shifts and bitwise AND/OR with
# conditional branches based on bit test results. This stresses the
# symbolic execution engine with bitvector reasoning + path expansion.

# --- Function 1: Extract bit field ---
# Extracts a 4-bit field from an 8-bit value starting at position pos.
# pos is 0 or 4, so we either get the lower or upper nibble.
# sig: (I64, I64) -> I64
# pre: $val >= 0 && $val <= 255 && $pos >= 0 && $pos <= 4 && ($pos == 0 || $pos == 4)
# post: $result >= 0 && $result <= 15
sub extract_nibble {
    my ($val, $pos) = @_;
    my $shifted = $val >> $pos;
    my $masked = $shifted & 15;
    return $masked;
}

# --- Function 2: Test individual bits and branch ---
# Tests bits 0, 1, 2, 3 of a 4-bit value independently.
# Returns the count of set bits (popcount for 4 bits).
# The verifier must reason about each bit test through 16 path combinations.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 15
# post: $result >= 0 && $result <= 4
sub popcount4 {
    my ($x) = @_;
    my $count = 0;
    my $b0 = $x & 1;
    if ($b0 == 1) {
        $count = $count + 1;
    }
    my $b1 = ($x >> 1) & 1;
    if ($b1 == 1) {
        $count = $count + 1;
    }
    my $b2 = ($x >> 2) & 1;
    if ($b2 == 1) {
        $count = $count + 1;
    }
    my $b3 = ($x >> 3) & 1;
    if ($b3 == 1) {
        $count = $count + 1;
    }
    return $count;
}

# --- Function 3: Set and clear bits conditionally ---
# Given a byte value and a bit position (0-3), sets the bit if flag==1,
# clears it if flag==0. Returns the modified value.
# Uses the pattern: set = val | (1 << pos), clear = val & (~(1 << pos)) & 255
# sig: (I64, I64, I64) -> I64
# pre: $val >= 0 && $val <= 255 && $pos >= 0 && $pos <= 3 && $flag >= 0 && $flag <= 1
# post: $result >= 0 && $result <= 255
sub set_or_clear_bit {
    my ($val, $pos, $flag) = @_;
    my $mask = 1 << $pos;
    my $r;
    if ($flag == 1) {
        $r = $val | $mask;
    } else {
        my $inv = (~$mask) & 255;
        $r = $val & $inv;
    }
    return $r;
}

# --- Function 4: Bit field insert ---
# Inserts a 2-bit value into positions 2-3 of an 8-bit byte.
# Clears bits 2-3 first, then ORs in the new field shifted into position.
# Postcondition: result is still a valid byte.
# sig: (I64, I64) -> I64
# pre: $byte >= 0 && $byte <= 255 && $field >= 0 && $field <= 3
# post: $result >= 0 && $result <= 255
sub insert_field_2_3 {
    my ($byte, $field) = @_;
    my $clear_mask = (~12) & 255;
    my $cleared = $byte & $clear_mask;
    my $shifted_field = $field << 2;
    my $r = $cleared | $shifted_field;
    return $r;
}

# --- Function 5: Multi-path bit pattern classifier ---
# Classifies a 4-bit value based on which bits are set.
# Returns different values based on combinations of bit tests,
# creating many conditional paths for the verifier to explore.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 15
# post: $result >= 0 && $result <= 4
sub bit_pattern_class {
    my ($x) = @_;
    my $b0 = $x & 1;
    my $b1 = ($x >> 1) & 1;
    my $b2 = ($x >> 2) & 1;
    my $b3 = ($x >> 3) & 1;
    my $r;
    if ($b3 == 1 && $b0 == 1) {
        $r = 4;
    } elsif ($b2 == 1 && $b1 == 1) {
        $r = 3;
    } elsif ($b1 == 1 || $b2 == 1) {
        $r = 2;
    } elsif ($b0 == 1) {
        $r = 1;
    } else {
        $r = 0;
    }
    return $r;
}
