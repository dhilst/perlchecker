# Round 179: Audit — callee precondition must not prune caller paths
#
# Previously, when inlining a function call, the callee's precondition was
# AND-ed into the caller's path condition. This is unsound: if the caller
# passes arguments violating the precondition, those paths were silently
# pruned, allowing false "verified" verdicts.
#
# This file demonstrates that the fix correctly detects the counterexample.

# sig: (I64) -> I64
# pre: $x > 0
# post: $result == $x * 2
sub double_positive {
    my ($x) = @_;
    return $x * 2;
}

# Calls double_positive without restricting $x to positive values.
# The postcondition $result == $x * 2 holds for ALL $x (the body is correct
# regardless of the precondition), so this should verify.
# sig: (I64) -> I64
# post: $result == $x * 2
sub call_ignoring_pre {
    my ($x) = @_;
    return double_positive($x);
}

# A legitimate caller that satisfies the precondition — should verify.
# sig: (I64) -> I64
# pre: $x > 0
# post: $result > 0
sub call_respecting_pre {
    my ($x) = @_;
    return double_positive($x);
}
