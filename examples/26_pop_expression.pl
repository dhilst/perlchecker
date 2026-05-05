# =============================================================
# Round 26: pop(@arr) expression for arrays
# =============================================================
# pop returns the last element and decrements the array length.

# --- push then pop returns the pushed value ---
# sig: (Array<I64>, I64, I64) -> I64
# pre: scalar(@arr) == $n && $n >= 0 && $n <= 10 && $val >= 0
# post: $result == $val
sub push_pop {
    my ($arr, $n, $val) = @_;
    push(@arr, $val);
    my $r = pop(@arr);
    return $r;
}

# --- two pops return elements in reverse order ---
# sig: (Array<I64>, I64, I64, I64) -> I64
# pre: scalar(@arr) == $n && $n >= 0 && $n <= 10
# post: $result == $b
sub pop_middle {
    my ($arr, $n, $b, $c) = @_;
    push(@arr, $b);
    push(@arr, $c);
    my $top = pop(@arr);
    my $mid = pop(@arr);
    return $mid;
}
