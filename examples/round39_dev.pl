# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 1000
# post: $result == $x + 1_000_000
sub add_million {
    my ($x) = @_;
    my $r = $x + 1_000_000;
    return $r;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 0xFF_FF
# post: $result >= 0 && $result <= 255
sub high_byte {
    my ($x) = @_;
    my $r = ($x >> 8) & 0xFF;
    return $r;
}
