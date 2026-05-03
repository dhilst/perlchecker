# sig: (Str, Str) -> Int
# pre: length($s) >= 3 && length($old) == 1
# post: $result >= 3
sub replace_preserves_min_length {
    my ($s, $old) = @_;
    my $new = "XX";
    my $r = replace($s, $old, $new);
    my $len = length($r);
    return $len;
}

# sig: (Str) -> Int
# pre: length($s) >= 5 && contains($s, "ab") == 1
# post: $result >= 5
sub replace_ab_with_xy {
    my ($s) = @_;
    my $r = replace($s, "ab", "xy");
    my $len = length($r);
    return $len;
}
