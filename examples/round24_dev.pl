# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result >= 0
sub abs_with_modifier {
    my ($x) = @_;
    return 0 - $x if ($x < 0);
    return $x;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result <= 50
sub cap_at_fifty {
    my ($x) = @_;
    return 50 unless ($x <= 50);
    return $x;
}
