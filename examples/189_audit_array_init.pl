# Soundness audit: array literal initialization vs Z3 encoding.
# Tests that element values, length, and OOB reads are Perl-consistent.

# --- Length of a 3-element literal should be 3 ---
# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 3
sub literal_length {
    my ($x) = @_;
    my @arr = (10, 20, 30);
    return scalar(@arr);
}

# --- Element 0 of (10, 20, 30) should be 10 ---
# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 10
sub first_element {
    my ($x) = @_;
    my @arr = (10, 20, 30);
    return $arr[0];
}

# --- Element 2 of (10, 20, 30) should be 30 ---
# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 30
sub last_element {
    my ($x) = @_;
    my @arr = (10, 20, 30);
    return $arr[2];
}

# --- OOB read at index 5 on a 3-element literal should be 0 (undef) ---
# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 0
sub oob_returns_zero {
    my ($x) = @_;
    my @arr = (10, 20, 30);
    return $arr[5];
}

# --- Sum of all three elements ---
# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 60
sub sum_all_elements {
    my ($x) = @_;
    my @arr = (10, 20, 30);
    return $arr[0] + $arr[1] + $arr[2];
}
