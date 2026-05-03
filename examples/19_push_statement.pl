# =============================================================
# Round 19: push(@arr, $val) statement
# =============================================================
# push stores a value at the current array length and increments
# the length. This enables building arrays element by element.

# --- push stores at the current length index ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n >= 0
# post: $result == $val
sub push_and_read {
    my ($arr, $n, $val) = @_;
    push(@arr, $val);
    return $arr[$n];
}

# --- two consecutive pushes ---
# sig: (Array<Int>, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n >= 0
# post: $result == $b
sub push_twice_read_second {
    my ($arr, $n, $b) = @_;
    push(@arr, 0);
    push(@arr, $b);
    my $idx = $n + 1;
    return $arr[$idx];
}

# --- push updates scalar length ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n >= 0 && $n <= 10
# post: $result == $n + 1
sub push_increments_length {
    my ($arr, $n) = @_;
    push(@arr, 42);
    return scalar(@arr);
}
