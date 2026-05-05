# =============================================================
# Round 96: Complex postcondition implication path stress
# =============================================================
# Functions where postconditions use logical implication patterns
# expressed as (!cond || result), meaning "if cond then result
# must hold". The solver must reason about conditional guarantees
# across multiple execution paths.

# --- Function 1: Simple implication — if x > 0 then result > 0 ---
# When x is positive, result is x*2 (positive). When x <= 0,
# result is 0. The implication (!($x > 0) || $result > 0) holds
# because on the x > 0 path, result = x*2 > 0.
# sig: (I64) -> I64
# pre: $x >= -5 && $x <= 5
# post: (!($x > 0) || $result > 0) && $result >= 0
sub implication_positive {
    my ($x) = @_;
    if ($x > 0) {
        return $x * 2;
    }
    return 0;
}

# --- Function 2: Conjunction of two implications ---
# Two implications combined: if x > 0 then result >= x, AND
# if x < 0 then result >= -x. This means result >= abs(x)
# regardless of sign.
# sig: (I64) -> I64
# pre: $x >= -5 && $x <= 5
# post: (!($x > 0) || $result >= $x) && (!($x < 0) || $result >= 0 - $x) && $result >= 0
sub implication_abs_bound {
    my ($x) = @_;
    if ($x > 0) {
        return $x + 1;
    }
    if ($x < 0) {
        return 0 - $x + 1;
    }
    return 0;
}

# --- Function 3: Implication with loop — if n > 0 then result >= n ---
# Accumulates a sum in a loop. If n > 0, the loop runs at least
# once adding at least 1 each iteration, so result >= n.
# If n <= 0, no constraint on result (trivially satisfied).
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 5
# post: (!($n > 0) || $result >= $n) && $result >= 0
sub implication_loop_sum {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    while ($i < $n) {
        $sum = $sum + $i + 1;
        $i = $i + 1;
    }
    return $sum;
}

# --- Function 4: Biconditional-style — condition determines result exactly ---
# Uses (condition || result == A) && (!condition || result == B) pattern.
# When flag > 0: result must be 10. When flag <= 0: result must be 20.
# sig: (I64) -> I64
# pre: $flag >= -3 && $flag <= 3
# post: ($flag > 0 || $result == 20) && (!($flag > 0) || $result == 10)
sub biconditional_branch {
    my ($flag) = @_;
    if ($flag > 0) {
        return 10;
    }
    return 20;
}

# --- Function 5: Triple implication with nested conditions ---
# Three implications about a piecewise function:
# if x > 3 then result == 3 (capped at 3)
# if x >= 1 && x <= 3 then result == x (identity in range)
# if x < 1 then result == 1 (floored at 1)
# This is a clamp(x, 1, 3) function.
# sig: (I64) -> I64
# pre: $x >= -2 && $x <= 6
# post: (!($x > 3) || $result == 3) && (!($x < 1) || $result == 1) && $result >= 1 && $result <= 3
sub implication_clamp {
    my ($x) = @_;
    if ($x > 3) {
        return 3;
    }
    if ($x < 1) {
        return 1;
    }
    return $x;
}
