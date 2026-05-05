# Round 116: Scalar references with static alias resolution

# sig: (I64) -> I64
# pre: $x > 0
# post: $result == 10
sub ref_write {
    my ($x) = @_;
    my $ref = \$x;
    $$ref = 10;
    return $x;
}

# sig: (I64) -> I64
# pre: $x > 0
# post: $result == $x * 2
sub ref_read {
    my ($x) = @_;
    my $ref = \$x;
    my $y = $$ref + $x;
    return $y;
}

# sig: (I64) -> I64
# pre: $n > 0
# post: $result == $n + 5
sub ref_chain {
    my ($n) = @_;
    my $ref = \$n;
    $$ref = $n + 5;
    return $$ref;
}
