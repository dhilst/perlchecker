# Round 178: Hash read of non-existent key returns undef (0 / "")
# In Perl, reading a hash key that was never stored returns undef.
# In numeric context undef == 0, in string context undef eq "".
# The checker must enforce this invariant via the exists companion.

# Test 1: If exists() returns 0 for a key, reading that key must be 0.
# sig: (Hash<Str, Int>, Str) -> Int
# post: $result == 0
sub nonexist_implies_zero {
    my ($h, $k) = @_;
    if (exists($h{$k}) == 0) {
        return $h{$k};
    }
    return 0;
}

# Test 2: On the non-existence branch, value must be non-negative (i.e. 0).
# sig: (Hash<Str, Int>, Str) -> Int
# post: $result >= 0
sub nonexist_positive_or_zero {
    my ($h, $k) = @_;
    if (exists($h{$k}) == 0) {
        return $h{$k};
    }
    return 1;
}

# Test 3: After storing to key "x", reading "x" should give back 42 (key exists).
# sig: (Hash<Str, Int>, Str) -> Int
# post: $result == 42
sub stored_key_returns_value {
    my ($h, $k) = @_;
    $h{"x"} = 42;
    return $h{"x"};
}
