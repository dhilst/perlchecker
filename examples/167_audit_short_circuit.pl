# Round 167: Short-circuit evaluation soundness for && and ||
#
# Bug: function calls inside the RHS of && or || were hoisted before the
# condition, executing unconditionally. When the callee's precondition
# conflicted with the short-circuited path, that path was incorrectly pruned,
# allowing false postconditions to verify.
#
# Fix: desugar && / || into if-then-else when the RHS contains hoisted calls,
# so calls only execute on the non-short-circuited path.

# A helper that requires $x > 5
# sig: (I64) -> I64
# pre: $x > 5
# post: $result == 1
sub only_above_five {
    my ($x) = @_;
    return 1;
}

# Previously unsound: checker verified $result == 99 even though
# Perl returns $x (which is 0..5) on the else branch for $x <= 5.
# After fix: correctly finds counterexample at x = 0.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= 0
sub and_short_circuit_fixed {
    my ($x) = @_;
    # When $x <= 5: && short-circuits (never calls only_above_five), goes to else
    # When $x > 5: evaluates only_above_five($x) which returns 1 > 0, goes to if
    if ($x > 5 && only_above_five($x) > 0) {
        return 99;
    }
    return $x;
}

# A helper that requires $x > 5
# sig: (I64) -> I64
# pre: $x > 5
# post: $result == 0
sub big_returns_zero {
    my ($x) = @_;
    return 0;
}

# Test || short-circuit: ensures right side not evaluated when left is true
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= 0
sub or_short_circuit_fixed {
    my ($x) = @_;
    # When $x <= 5: || sees truthy LHS, short-circuits (never calls big_returns_zero)
    # When $x > 5: evaluates big_returns_zero($x) returns 0, 0 > 0 = false, goes to else
    if ($x <= 5 || big_returns_zero($x) > 0) {
        return $x;
    }
    return 99;
}

# Verify that plain && / || (no function calls) still work correctly
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 100 && $y >= 0 && $y <= 100
# post: $result >= 0
sub plain_and_or {
    my ($x, $y) = @_;
    if ($x > 50 && $y > 50) {
        return $x + $y;
    }
    if ($x > 50 || $y > 50) {
        return $x + $y;
    }
    return 0;
}
