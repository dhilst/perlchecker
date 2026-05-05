# Round 160: Fix exists() unsoundness — normalize companion to 0/1

# Test 1: exists() on a parameter hash always returns 0 or 1.
# Before fix: the unconstrained companion hash could return any integer,
# so Z3 could find a "counterexample" with exists() returning e.g. 7.
# sig: (Hash<Str, I64>, Str) -> I64
# pre: length($k) >= 1 && length($k) <= 5
# post: $result >= 0 && $result <= 1
sub exists_returns_bool {
    my ($h, $k) = @_;
    my $r = exists($h{$k});
    return $r;
}

# Test 2: After assignment, exists() still returns exactly 1
# sig: (Hash<Str, I64>, Str) -> I64
# pre: length($k) >= 1 && length($k) <= 5
# post: $result == 1
sub exists_after_store {
    my ($h, $k) = @_;
    $h{$k} = 99;
    my $r = exists($h{$k});
    return $r;
}

# Test 3: exists() result used in arithmetic stays bounded
# sig: (Hash<Str, I64>, Str, Str) -> I64
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: $result >= 0 && $result <= 2
sub exists_sum_bounded {
    my ($h, $a, $b) = @_;
    my $r = exists($h{$a}) + exists($h{$b});
    return $r;
}
