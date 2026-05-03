# sig: (Int, Int) -> Int
# pre: $y != 0
# post: $result == $x % $y
sub modulo_verified {
    my ($x, $y) = @_;
    return $x % $y;
}
