# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 255
# post: $result == 5
sub return_binary {
    my ($x) = @_;
    my $r = 0b101;
    return $r;
}

# sig: (I64) -> I64
# pre: $mode >= 0
# post: $result == 493
sub octal_permission {
    my ($mode) = @_;
    my $r = 0o755;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 0xFF
# post: $result == ($x & 0b11110000)
sub mask_with_binary {
    my ($x) = @_;
    my $r = $x & 0b11110000;
    return $r;
}
