# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 5
# post: $result == 0
sub while_verified {
    my ($x) = @_;
    while ($x > 0) {
        $x = $x - 1;
    }
    return $x;
}
