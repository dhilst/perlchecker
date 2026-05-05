# Test 1: Multiple returns - both satisfy postcondition (VERIFIED)
# sig: (I64) -> I64
# pre: $x > -100 && $x < 100
# post: $result >= 0
sub abs_value {
    my ($x) = @_;
    if ($x >= 0) {
        return $x;
    }
    return 0 - $x;
}

# Test 2: Early return violates postcondition (COUNTEREXAMPLE x=0)
# sig: (I64) -> I64
# post: $result > 0
sub early_return_bug {
    my ($x) = @_;
    if ($x == 0) {
        return 0;
    }
    return 1;
}

# Test 3: Late return violates postcondition (COUNTEREXAMPLE)
# sig: (I64) -> I64
# post: $result > 0
sub late_return_bug {
    my ($x) = @_;
    if ($x == 0) {
        return 1;
    }
    return 0;
}

# Test 4: Three-way return all satisfying post (VERIFIED)
# sig: (I64) -> I64
# post: $result >= 0 && $result <= 2
sub three_way {
    my ($x) = @_;
    if ($x > 0) {
        return 2;
    }
    if ($x == 0) {
        return 1;
    }
    return 0;
}

# Test 5: Variable modified before early return (VERIFIED)
# sig: (I64) -> I64
# post: $result >= $x
sub modified_var_return {
    my ($x) = @_;
    if ($x > 0) {
        my $y = $x + 1;
        return $y;
    }
    return $x;
}

# Test 6: elsif with multiple early returns (VERIFIED)
# sig: (I64) -> I64
# post: $result >= 0 && $result <= 3
sub elsif_returns {
    my ($x) = @_;
    if ($x > 10) {
        return 3;
    } elsif ($x > 0) {
        return 2;
    } elsif ($x == 0) {
        return 1;
    }
    return 0;
}

# Test 7: Nested if-else with returns in different branches.
# The WRONG postcondition: $result == $x. This should fail because
# when $x <= 0, we return 0-$x which != $x (unless $x==0).
# sig: (I64) -> I64
# post: $result == $x
sub wrong_post_nested {
    my ($x) = @_;
    if ($x > 0) {
        if ($x > 100) {
            return 100;
        }
        return $x;
    }
    return 0 - $x;
}

# Test 8: Return from inside nested if — postcondition checked there too
# sig: (I64, I64) -> I64
# post: $result > 0
sub nested_early_return_bug {
    my ($x, $y) = @_;
    if ($x > 0) {
        if ($y == 0) {
            return 0;
        }
        return $x;
    }
    return 1;
}
