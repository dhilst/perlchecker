# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 2
sub add_two {
    my ($x) = @_;
    my $y = inc($x);
    return inc($y);
}
