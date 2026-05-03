# sig: (Int) -> Int
# post: $result == $x + 3
sub for_verified {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i < 3; $i = $i + 1) {
        $x = $x + 1;
    }
    return $x;
}
