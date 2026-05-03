# sig: (Str, Str) -> Int
# pre: length($a) >= 1 && length($b) >= 1
# post: ($result == -1 || $result == 0 || $result == 1)
sub str_compare {
    my ($a, $b) = @_;
    my $r = $a cmp $b;
    return $r;
}

# sig: (Str, Str) -> Int
# pre: length($a) >= 1 && length($b) >= 1
# post: $result >= 0
sub cmp_abs {
    my ($a, $b) = @_;
    my $c = $a cmp $b;
    my $r = ($c >= 0) ? $c : 0 - $c;
    return $r;
}
