# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 5
# post: $result >= 1
sub square {
    my ($x) = @_;
    my $r = $x ** 2;
    return $r;
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 4
# post: $result >= 1
sub two_to_n {
    my ($n) = @_;
    my $r = 2 ** $n;
    return $r;
}
