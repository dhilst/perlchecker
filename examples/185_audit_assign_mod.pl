# Test 1: compound assignment with modifier — $x += 1 if ($cond)
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $flag >= 0 && $flag <= 1
# post: $result >= 0 && $result <= 11
sub compound_if {
    my ($x, $flag) = @_;
    $x += 1 if ($flag == 1);
    return $x;
}

# Test 2: simple assign with modifier, variable read on both paths
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result >= 0 && $result <= 10
sub abs_mod {
    my ($x) = @_;
    my $r = $x;
    $r = 0 - $x if ($x < 0);
    return $r;
}

# Test 3: unless modifier with compound assignment
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 100 && $flag >= 0 && $flag <= 1
# post: $result >= 0
sub compound_unless {
    my ($x, $flag) = @_;
    $x -= 1 unless ($flag == 0);
    return $x;
}

# Test 4: chained conditional assignments — test SSA merging across multiple modifiers
# In Perl: $x starts at 0, gets +10 if a, gets +20 if b
# Range: [0, 30], so result <= 30 should hold
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1
# post: $result >= 0 && $result <= 30
sub chained_modifiers {
    my ($a, $b) = @_;
    my $x = 0;
    $x += 10 if ($a == 1);
    $x += 20 if ($b == 1);
    return $x;
}

# Test 5: conditional assign then unconditional read — verify the phi merge is correct
# if flag, x=100; then always return x+1
# result is either original x+1 or 101
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 50 && $flag >= 0 && $flag <= 1
# post: $result >= 1 && $result <= 101
sub modify_then_read {
    my ($x, $flag) = @_;
    $x = 100 if ($flag == 1);
    return $x + 1;
}

# Test 6: conditional assign SHOULD fail with tighter bound
# If flag=1, x becomes 100, so x+1=101, but we claim result<=51
# This should be a counterexample
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 50 && $flag >= 0 && $flag <= 1
# post: $result >= 1 && $result <= 51
sub modify_then_read_tight {
    my ($x, $flag) = @_;
    $x = 100 if ($flag == 1);
    return $x + 1;
}

# Test 7: verify false-condition path preserves value
# If cond is always false, x should remain 10
# sig: (I64) -> I64
# pre: $flag == 0
# post: $result == 10
sub false_cond_preserves {
    my ($flag) = @_;
    my $x = 10;
    $x = 999 if ($flag == 1);
    return $x;
}

# Test 8: verify compound assign on false path preserves
# sig: (I64) -> I64
# pre: $flag == 0
# post: $result == 10
sub false_cond_compound {
    my ($flag) = @_;
    my $x = 10;
    $x *= 2 if ($flag == 1);
    return $x;
}
