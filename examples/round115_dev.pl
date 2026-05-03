# Round 115: exists() builtin for hash key existence checking

# Test 1: After assigning to a hash key, exists returns 1
# sig: (Hash<Str, Int>, Str) -> Int
# pre: length($k) >= 1
# post: $result == 1
sub check_exists_after_assign {
    my ($h, $k) = @_;
    $h{$k} = 42;
    my $r = exists($h{$k});
    return $r;
}

# Test 2: exists on a parameter hash is unconstrained, use in conditional
# sig: (Hash<Str, Int>, Str) -> Int
# pre: length($k) >= 1
# post: $result >= 0
sub check_exists_param_hash {
    my ($h, $k) = @_;
    my $r = 0;
    if (exists($h{$k}) == 1) {
        $r = $h{$k};
        if ($r < 0) {
            $r = 0;
        }
    }
    return $r;
}

# Test 3: Multiple key assignments, exists checks specific key
# sig: (Hash<Str, Int>, Str, Str) -> Int
# pre: length($k1) >= 1 && length($k2) >= 1
# post: $result == 1
sub check_exists_multiple_keys {
    my ($h, $k1, $k2) = @_;
    $h{$k1} = 10;
    $h{$k2} = 20;
    my $r = exists($h{$k1});
    return $r;
}
