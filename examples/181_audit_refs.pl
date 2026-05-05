# Round 181: Audit of reference aliasing soundness

# After $x = 10 and $$ref = 99, $x must reflect the write through the ref.
# sig: (I64) -> I64
# pre: $n > 0
# post: $result == 99
sub deref_write_alias {
    my ($n) = @_;
    my $x = 10;
    my $ref = \$x;
    $$ref = 99;
    return $x;
}

# After taking a reference to $x, reading $$ref must equal reading $x.
# sig: (I64) -> I64
# pre: $n > 0
# post: $result == $n
sub deref_read_alias {
    my ($n) = @_;
    my $x = $n;
    my $ref = \$x;
    return $$ref;
}

# Two references to the same variable should alias correctly.
# sig: (I64) -> I64
# pre: $n > 0
# post: $result == 42
sub two_refs_same_target {
    my ($n) = @_;
    my $x = $n;
    my $ref1 = \$x;
    my $ref2 = \$x;
    $$ref1 = 42;
    return $$ref2;
}

# Arrow array write through ref must be visible via original array.
# sig: (I64) -> I64
# pre: $n >= 0
# post: $result == 77
sub arrow_array_alias {
    my ($n) = @_;
    my @arr = (1, 2, 3);
    my $aref = \@arr;
    $aref->[1] = 77;
    return $arr[1];
}

# Arrow hash write through ref must be visible via original hash.
# sig: (Hash<Str, I64>, Str) -> I64
# pre: $key eq "a"
# post: $result == 55
sub arrow_hash_alias {
    my ($h, $key) = @_;
    my $href = \%h;
    $href->{"a"} = 55;
    return $h{"a"};
}

# BUG PROBE: False invariant that holds for ONE step from initial value
# but fails for arbitrary starting points.
# Loop: x starts at 0, each iteration does $$ref = $x + 3.
# Trace: 0 -> 3 -> 6 -> 9 -> 12 (exits).
# Invariant "$x <= 3" holds for step 0->3, but NOT for step 3->6.
# Without freshening $x, only the 0->3 step is checked (using
# constrained $n == 0), so the false invariant passes.
# sig: (I64) -> I64
# pre: $n == 0
# post: $result >= 0
sub deref_loop_invariant_bug {
    my ($n) = @_;
    my $x = $n;
    my $ref = \$x;
    # inv: $x <= 3
    while ($x < 10) {
        $$ref = $x + 3;
    }
    return $x;
}
