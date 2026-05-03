# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result >= 0 && $result <= 240
sub mask_high_nibble {
    my ($x) = @_;
    my $r = $x & 0xF0;
    return $r;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 0xFF
# post: $result >= 0 && $result <= 15
sub low_nibble_hex {
    my ($x) = @_;
    my $r = $x & 0x0F;
    return $r;
}

# sig: (Int) -> Int
# pre: $x == 0xAB
# post: $result == 0xAB
sub hex_identity {
    my ($x) = @_;
    my $r = $x;
    return $r;
}
