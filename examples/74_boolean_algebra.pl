# =============================================================
# Round 74: Boolean algebra path verification
# =============================================================
# Functions implementing boolean logic patterns (de Morgan's, XOR
# from AND/OR/NOT, majority vote) using Perl's &&/||/! operators
# with integer 0/1 values, creating paths where the verifier must
# reason about boolean satisfiability.

# --- Function 1: De Morgan's law verification ---
# Verifies !(a && b) == (!a || !b) for boolean inputs.
# Both branches compute the same result, proving the law holds.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1
# post: $result == 1
sub de_morgan_and {
    my ($a, $b) = @_;
    my $lhs;
    if ($a == 1 && $b == 1) {
        $lhs = 0;
    } else {
        $lhs = 1;
    }
    my $not_a;
    if ($a == 0) {
        $not_a = 1;
    } else {
        $not_a = 0;
    }
    my $not_b;
    if ($b == 0) {
        $not_b = 1;
    } else {
        $not_b = 0;
    }
    my $rhs;
    if ($not_a == 1 || $not_b == 1) {
        $rhs = 1;
    } else {
        $rhs = 0;
    }
    my $equal;
    if ($lhs == $rhs) {
        $equal = 1;
    } else {
        $equal = 0;
    }
    return $equal;
}

# --- Function 2: XOR from AND/OR/NOT ---
# Implements XOR(a, b) = (a || b) && !(a && b) and verifies it
# equals the direct computation (a != b) ? 1 : 0.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1
# post: $result == 1
sub xor_equivalence {
    my ($a, $b) = @_;
    my $or_ab;
    if ($a == 1 || $b == 1) {
        $or_ab = 1;
    } else {
        $or_ab = 0;
    }
    my $and_ab;
    if ($a == 1 && $b == 1) {
        $and_ab = 1;
    } else {
        $and_ab = 0;
    }
    my $not_and;
    if ($and_ab == 0) {
        $not_and = 1;
    } else {
        $not_and = 0;
    }
    my $xor_computed;
    if ($or_ab == 1 && $not_and == 1) {
        $xor_computed = 1;
    } else {
        $xor_computed = 0;
    }
    my $xor_direct;
    if ($a != $b) {
        $xor_direct = 1;
    } else {
        $xor_direct = 0;
    }
    my $match;
    if ($xor_computed == $xor_direct) {
        $match = 1;
    } else {
        $match = 0;
    }
    return $match;
}

# --- Function 3: Majority vote 2-of-3 ---
# Returns 1 if at least 2 of the 3 inputs are 1, else 0.
# The postcondition verifies tight bounds: result is always 0 or 1.
# sig: (I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1 && $c >= 0 && $c <= 1
# post: $result >= 0 && $result <= 1
sub majority_2of3 {
    my ($a, $b, $c) = @_;
    my $sum = $a + $b + $c;
    my $r;
    if ($sum >= 2) {
        $r = 1;
    } else {
        $r = 0;
    }
    return $r;
}

# --- Function 4: Boolean implication chain ---
# Computes a chain: a implies b, b implies c, c implies d.
# Implication: p => q is equivalent to !p || q.
# Returns the conjunction of all three implications.
# With all inputs constrained to 0/1, the verifier must reason
# through 16 input combinations.
# sig: (I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1 && $c >= 0 && $c <= 1 && $d >= 0 && $d <= 1
# post: $result >= 0 && $result <= 1
sub implication_chain {
    my ($a, $b, $c, $d) = @_;
    my $imp1;
    if ($a == 0 || $b == 1) {
        $imp1 = 1;
    } else {
        $imp1 = 0;
    }
    my $imp2;
    if ($b == 0 || $c == 1) {
        $imp2 = 1;
    } else {
        $imp2 = 0;
    }
    my $imp3;
    if ($c == 0 || $d == 1) {
        $imp3 = 1;
    } else {
        $imp3 = 0;
    }
    my $r;
    if ($imp1 == 1 && $imp2 == 1 && $imp3 == 1) {
        $r = 1;
    } else {
        $r = 0;
    }
    return $r;
}

# --- Function 5: Majority vote 3-of-5 with path explosion ---
# Returns 1 if at least 3 of 5 inputs are 1. The sum ranges
# from 0 to 5, and the verifier must handle 32 input combinations
# through the conditional paths.
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1 && $c >= 0 && $c <= 1 && $d >= 0 && $d <= 1 && $e >= 0 && $e <= 1
# post: $result >= 0 && $result <= 1
sub majority_3of5 {
    my ($a, $b, $c, $d, $e) = @_;
    my $sum = $a + $b + $c + $d + $e;
    my $r;
    if ($sum >= 3) {
        $r = 1;
    } else {
        $r = 0;
    }
    return $r;
}
