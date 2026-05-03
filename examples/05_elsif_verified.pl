# sig: (Int) -> Int
# post: $result >= 0
sub elsif_verified {
    my ($x) = @_;
    if ($x < 0) {
        return 0 - $x;
    } elsif ($x == 0) {
        return 1;
    } else {
        return $x;
    }
}
