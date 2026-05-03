# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 5
# post: $result == $x * 2
sub double_until {
    my ($x) = @_;
    my $r = 0;
    my $i;
    $i = 0;
    until ($i >= $x) {
        $r += 2;
        $i += 1;
    }
    return $r;
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 5
# post: $result >= $n
sub count_up {
    my ($n) = @_;
    my $c = 0;
    until ($c >= $n) {
        $c += 1;
    }
    return $c;
}
