# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 100
# post: $result >= 1
sub croak_on_zero {
    my ($x) = @_;
    croak "invalid" if ($x <= 0);
    return $x;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 50
# post: $result >= 0
sub confess_negative {
    my ($x) = @_;
    confess "negative" unless ($x >= 0);
    return $x;
}

# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 10
# post: $result == $x * 2
sub die_guard {
    my ($x) = @_;
    die "bad" if ($x <= 0);
    my $r = $x * 2;
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 10
# post: $result == $x
sub bare_die_in_branch {
    my ($x) = @_;
    if ($x <= 0) {
        die;
    }
    return $x;
}
