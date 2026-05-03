# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result == $x + 1
sub inc_with_warn {
    my ($x) = @_;
    warn "incrementing";
    my $r = $x + 1;
    return $r;
}

# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result >= 0
sub abs_with_debug {
    my ($x) = @_;
    my $r = $x;
    if ($x < 0) {
        warn "negating";
        $r = 0 - $x;
    }
    return $r;
}
