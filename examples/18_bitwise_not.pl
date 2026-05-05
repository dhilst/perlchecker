# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result >= 0 && $result <= 255
sub complement_byte {
    my ($x) = @_;
    my $r = (~$x) & 255;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result == 255 - $x
sub complement_byte_exact {
    my ($x) = @_;
    my $r = (~$x) & 255;
    return $r;
}
