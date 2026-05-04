# =============================================================
# Round 169: Fix pop() on empty array — return 0 (undef) not unconstrained
# =============================================================
# In Perl, pop(@empty) returns undef which numifies to 0.
# Previously the verifier decremented length to -1 and read arr[-1],
# which was unconstrained — an unsoundness.

# --- pop on empty array returns 0 (undef numified) ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0
# post: $result == 0
sub pop_empty_is_zero {
    my ($arr, $n) = @_;
    my $x = pop(@arr);
    return $x;
}

# --- pop beyond array bounds returns 0 ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n == 3 && $arr[0] == 10 && $arr[1] == 20 && $arr[2] == 30
# post: $result == 0
sub pop_four_from_three {
    my ($arr, $n) = @_;
    my $a = pop(@arr);
    my $b = pop(@arr);
    my $c = pop(@arr);
    my $d = pop(@arr);
    return $d;
}

# --- normal pop still works correctly ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n == 3 && $arr[0] == 10 && $arr[1] == 20 && $arr[2] == 30
# post: $result == 30
sub pop_returns_last {
    my ($arr, $n) = @_;
    my $x = pop(@arr);
    return $x;
}

# --- push then scalar returns correct new length ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 2 && $val >= 0
# post: $result == 3
sub push_then_scalar {
    my ($arr, $n, $val) = @_;
    push(@arr, $val);
    my $len = scalar(@arr);
    return $len;
}

# --- length stays at 0 after pop on empty ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0
# post: $result == 0
sub pop_empty_length_stays_zero {
    my ($arr, $n) = @_;
    my $x = pop(@arr);
    my $len = scalar(@arr);
    return $len;
}
