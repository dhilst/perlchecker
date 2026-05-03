# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 100
# post: $result >= 1
sub guarded_with_die_if {
    my ($x) = @_;
    die "zero" if ($x == 0);
    print "processing";
    return $x;
}

# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result >= 0
sub positive_guard {
    my ($x) = @_;
    die "negative" unless ($x >= 0);
    say "ok";
    return $x;
}
