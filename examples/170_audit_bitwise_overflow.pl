# Soundness audit: bitwise ops (& | ^) must guard against int2bv overflow.
#
# Z3's int2bv wraps mod 2^64, but Perl saturates floats at UINT64_MAX
# when converting to unsigned for bitwise operations.  Values >= 2^64
# in the tool's unbounded integer model would wrap to 0 via int2bv,
# while Perl would saturate at UINT64_MAX.
#
# Example: (~0 + 1) & 255
#   Tool (before fix): int2bv(2^64, 64) = 0, 0 & 255 = 0  (WRONG)
#   Perl:              ~0+1 overflows to float, saturates, & 255 = 255
#
# Fix: encode_int_safety now guards BitAnd/BitOr/BitXor operands
# to be in [-(2^63), 2^64), discarding paths with unrepresentable values.

# Test 1: Normal bitwise AND on in-range values still verifies
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result >= 0 && $result <= 255
sub and_in_range {
    my ($x) = @_;
    return ($x & 255);
}

# Test 2: Normal bitwise OR on in-range values still verifies
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 15 && $b >= 0 && $b <= 240
# post: $result >= 0 && $result <= 255
sub or_in_range {
    my ($a, $b) = @_;
    return ($a | $b);
}

# Test 3: Normal bitwise XOR on in-range values still verifies
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result == 0
sub xor_self {
    my ($x) = @_;
    return ($x ^ $x);
}

# Test 4: BitNot result (up to 2^64-1) used in AND still works
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result == 255 - $x
sub bitnot_then_and {
    my ($x) = @_;
    return ((~$x) & 255);
}

# Test 5: Negative values in bitwise AND still work correctly
# In Perl: (-1) & 255 == 255 because -1 as unsigned 64-bit is all 1s
# sig: (Int) -> Int
# pre: $x >= -100 && $x <= -1
# post: $result >= 0 && $result <= 255
sub neg_and_byte {
    my ($x) = @_;
    return ($x & 255);
}
