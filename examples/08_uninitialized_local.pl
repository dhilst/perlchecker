# sig: (Int) -> Int
# post: $result >= $x
sub uninitialized_local {
    my ($x) = @_;
    my $y;
    if ($x > 0) {
        $y = $x;
    }
    return $y;
}
