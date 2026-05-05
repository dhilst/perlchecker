# sig: (I64) -> I64
# post: $result == $x + 3
sub for_verified {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i < 3; $i = $i + 1) {
        $x = $x + 1;
    }
    return $x;
}
