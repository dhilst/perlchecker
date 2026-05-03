# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result >= 0
sub conditional_assign {
    my ($x) = @_;
    my $r = $x;
    $r = 0 - $x if ($x < 0);
    return $r;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result <= 50
sub cap_unless {
    my ($x) = @_;
    my $r = $x;
    $r = 50 unless ($x <= 50);
    return $r;
}
