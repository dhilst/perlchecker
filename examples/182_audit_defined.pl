# Round 182: fix defined($hash{$key}) unsoundness
#
# Before this fix, defined($hash{$key}) always returned 1 regardless of
# whether the key existed.  In Perl, accessing a non-existent hash key
# returns undef, so defined() should return 0 for missing keys.
# The fix links defined($hash{$key}) to the exists-companion, making it
# equivalent to exists($hash{$key}) (the checker does not model undef
# values, so if a key exists its value is always defined).

# Test 1: defined() on a parameter hash is NOT always true
# (counterexample: key may not exist)
# sig: (Hash<Str, I64>, Str) -> I64
# pre: length($k) >= 1
# post: $result >= 0
sub defined_guards_hash_read {
    my ($h, $k) = @_;
    my $r = 0;
    if (defined($h{$k}) == 1) {
        $r = $h{$k};
        if ($r < 0) {
            $r = 0;
        }
    }
    return $r;
}

# Test 2: After assignment, defined() returns 1
# sig: (Hash<Str, I64>, Str) -> I64
# pre: length($k) >= 1
# post: $result == 1
sub defined_after_assign {
    my ($h, $k) = @_;
    $h{$k} = 42;
    return defined($h{$k});
}

# Test 3: defined() and exists() agree on a freshly assigned key
# sig: (Hash<Str, I64>, Str) -> I64
# pre: length($k) >= 1
# post: $result == 1
sub defined_matches_exists {
    my ($h, $k) = @_;
    $h{$k} = 99;
    my $d = defined($h{$k});
    my $e = exists($h{$k});
    if ($d == $e) {
        return 1;
    }
    return 0;
}

# Test 4: Unassigned key — defined() must not be provably 1
# (This would be UNSOUND before the fix)
# sig: (Hash<Str, I64>, Str, Str) -> I64
# pre: length($k1) >= 1 && length($k2) >= 1
# post: $result >= 0
sub defined_unassigned_key {
    my ($h, $k1, $k2) = @_;
    $h{$k1} = 10;
    my $r = 0;
    if (defined($h{$k2}) == 1) {
        $r = $h{$k2};
        if ($r < 0) {
            $r = 0;
        }
    }
    return $r;
}
