# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 100 && $b >= 0 && $b <= 100
# post: ($result == -1 || $result == 0 || $result == 1)
sub compare {
    my ($a, $b) = @_;
    my $r = $a <=> $b;
    return $r;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $b >= 0
# post: $result >= 0
sub abs_diff_via_spaceship {
    my ($a, $b) = @_;
    my $cmp = $a <=> $b;
    my $r = ($cmp >= 0) ? $a - $b : $b - $a;
    return $r;
}
