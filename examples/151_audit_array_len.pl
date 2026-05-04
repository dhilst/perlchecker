# =============================================================
# Round 151: Soundness audit — push/pop/scalar length tracking
# =============================================================
# Test that push, pop, and scalar(@arr) maintain consistent length
# across multiple operations.

# --- push then pop cancels out on length ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 5
# post: $result == scalar(@arr)
sub push_pop_length_identity {
    my ($arr, $v) = @_;
    push(@arr, $v);
    my $x = pop(@arr);
    return scalar(@arr);
}

# --- pop returns the value that was pushed ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 5
# post: $result == $v
sub push_pop_value {
    my ($arr, $v) = @_;
    push(@arr, $v);
    my $x = pop(@arr);
    return $x;
}

# --- two pushes then scalar is original + 2 ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 5
# post: $result == scalar(@arr) + 2
sub two_pushes_length {
    my ($arr, $a, $b) = @_;
    push(@arr, $a);
    push(@arr, $b);
    return scalar(@arr);
}

# --- two pushes then two pops: second pop returns first pushed ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 5
# post: $result == $a
sub two_push_two_pop_first {
    my ($arr, $a, $b) = @_;
    push(@arr, $a);
    push(@arr, $b);
    my $x = pop(@arr);
    my $y = pop(@arr);
    return $y;
}

# --- two pushes then two pops: first pop returns second pushed ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 5
# post: $result == $b
sub two_push_two_pop_second {
    my ($arr, $a, $b) = @_;
    push(@arr, $a);
    push(@arr, $b);
    my $x = pop(@arr);
    return $x;
}

# --- pop from array then check length ---
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 5
# post: $result == scalar(@arr) - 1
sub pop_decrements_length {
    my ($arr) = @_;
    my $x = pop(@arr);
    return scalar(@arr);
}

# --- Soundness fix: reading at the popped index no longer sees old value ---
# After pop, $arr[scalar(@arr)] is undef in Perl (0 in numeric context).
# The fix invalidates the popped slot with an unconstrained variable,
# so the checker correctly rejects this claim.
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 5 && $x != 0
# post: $result == $x
sub pop_ghost_read_counterexample {
    my ($arr, $x) = @_;
    push(@arr, $x);
    my $popped = pop(@arr);
    my $idx = scalar(@arr);
    return $arr[$idx];
}
