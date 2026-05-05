# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0 && $result <= 255
sub mask_byte {
    my ($a, $b) = @_;
    my $r = $a & $b;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 15
# post: $result == 0
sub xor_self {
    my ($x) = @_;
    my $r = $x ^ $x;
    return $r;
}

# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0 && $result <= 255
sub or_bytes {
    my ($a, $b) = @_;
    my $r = $a | $b;
    return $r;
}
