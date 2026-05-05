# ---- Test 1: Long elsif chain with accumulated negations ----
# The postcondition $result >= 0 && $result <= 4 should be verified
# because each branch returns a distinct value in [0,4].
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result >= 0 && $result <= 4
sub long_elsif_verified {
    my ($x) = @_;
    if ($x > 10) {
        return 0;
    } elsif ($x > 5) {
        return 1;
    } elsif ($x > 0) {
        return 2;
    } elsif ($x > -5) {
        return 3;
    } else {
        return 4;
    }
}

# ---- Test 2: elsif chain where result depends on accumulated negations ----
# When we reach "elsif ($x > 0)", we know $x <= 10 AND $x <= 5, so $x <= 5.
# The postcondition claims $result <= $x + 5, which should hold for each arm.
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 20
# post: $result <= $x + 5
sub elsif_negation_matters {
    my ($x) = @_;
    if ($x > 10) {
        return $x - 10;
    } elsif ($x > 5) {
        return $x - 5;
    } elsif ($x > 0) {
        return $x;
    } else {
        return 0;
    }
}

# ---- Test 3: unless with else — verify semantics match Perl ----
# unless ($x > 0) means "if NOT ($x > 0)", i.e., if ($x <= 0).
# So: $x <= 0 => return 1, $x > 0 => return $x.
# Postcondition: $result >= 1 should be COUNTEREXAMPLE because
# when $x > 0, we return $x, and $x could be 0... wait, $x > 0 means $x >= 1.
# Actually when $x <= 0 we return 1, when $x > 0 we return $x >= 1.
# So $result >= 1 should HOLD.
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result >= 1
sub unless_with_else_verified {
    my ($x) = @_;
    unless ($x > 0) {
        return 1;
    } else {
        return $x;
    }
}

# ---- Test 4: unless without else — counterexample ----
# unless ($x > 5) { $y = 100; }
# When $x > 5, $y stays 0, so $result can be 0.
# Postcondition $result > 0 should be COUNTEREXAMPLE.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 20
# post: $result > 0
sub unless_no_else_counterexample {
    my ($x) = @_;
    my $y = 0;
    $y = 100 unless ($x > 5);
    return $y;
}

# ---- Test 5: Tricky elsif — the checker must know prior conditions are negated ----
# We rely on the fact that reaching "elsif ($x < 20)" means $x <= 10.
# Combined with $x < 20 (always true in that arm), we know $x <= 10.
# So $x + 10 <= 20. The postcondition $result <= 20 should hold.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 30
# post: $result <= 20
sub elsif_prior_negation_critical {
    my ($x) = @_;
    if ($x > 10) {
        return 20;
    } elsif ($x < 20) {
        return $x + 10;
    } else {
        return 0;
    }
}
