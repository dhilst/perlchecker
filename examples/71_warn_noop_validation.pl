# =============================================================
# Round 71: Warn no-op validation in complex paths
# =============================================================
# Verifies that `warn` statements are correctly treated as no-ops
# during symbolic execution. Functions scatter warn calls through
# branches, loops, and conditional exits to stress-test that the
# presence of warn never affects postcondition provability.

# --- Function 1: Warn in every branch of nested if/elsif/else ---
# Despite warn in all branches, the arithmetic result is unaffected.
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 20
# post: $result >= 3 && $result <= 42
sub warn_in_nested_branches {
    my ($x) = @_;
    my $r = 0;
    if ($x > 15) {
        warn "high path";
        $r = $x * 2;
        if ($r > 30) {
            warn "very high sub-path";
            $r += 2;
        } else {
            warn "moderate high sub-path";
            $r += 1;
        }
    } elsif ($x > 8) {
        warn "mid path";
        $r = $x + 5;
        if ($x > 12) {
            warn "upper mid";
            $r += 3;
        } else {
            warn "lower mid";
            $r += 1;
        }
    } else {
        warn "low path";
        $r = $x + 2;
    }
    return $r;
}

# --- Function 2: Warn before and after loop operations ---
# A for-loop with warn before each accumulation step.
# The warn should not change the computed sum.
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 5
# post: $result >= 1 && $result <= 15
sub warn_in_loop_body {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 1; $i <= $n; $i++) {
        warn "iteration start";
        $sum += $i;
        warn "iteration end";
    }
    return $sum;
}

# --- Function 3: Warn before last/next in while-loop ---
# Complex control with warn right before early exit (last) and
# skip (next). Verifies warn doesn't affect loop termination logic.
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 5 && $threshold >= 2 && $threshold <= 4
# post: $result >= 0 && $result <= 12
sub warn_before_exits {
    my ($x, $threshold) = @_;
    my $acc = 0;
    my $i = 0;
    while ($i < $x) {
        $i += 1;
        if ($i > $threshold) {
            warn "breaking out of loop";
            last;
        }
        if ($i % 2 == 0) {
            warn "skipping even iteration";
            $acc += 1;
            next;
        }
        warn "normal path";
        $acc += $i;
    }
    return $acc;
}

# --- Function 4: Warn mixed with ternary and do-while ---
# Uses warn before a ternary expression inside a do-while loop.
# Demonstrates no interference with conditional expressions.
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 4
# post: $result >= 2 && $result <= 8
sub warn_with_ternary_loop {
    my ($x) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        warn "before ternary decision";
        my $inc = ($i < 2) ? 2 : 1;
        warn "after ternary decision";
        $acc += $inc;
        $i++;
    } while ($i < $x);
    return $acc;
}

# --- Function 5: Dense warn saturation in multi-path function ---
# Every possible statement position has a warn call.
# Postcondition depends purely on arithmetic, not on warn.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 5 && $b >= 0 && $b <= 5
# post: $result >= 0 && $result <= 35
sub warn_saturation {
    my ($a, $b) = @_;
    warn "entry";
    my $r = $a + $b;
    warn "after initial sum";
    if ($a > $b) {
        warn "a dominates";
        $r += $a;
        if ($a > 3) {
            warn "a is large";
            $r += $a;
        } else {
            warn "a is small-ish";
            $r += 1;
        }
    } else {
        warn "b dominates or equal";
        $r += $b;
        if ($b > 3) {
            warn "b is large";
            $r += $b;
        } else {
            warn "b is small-ish";
            $r += 1;
        }
    }
    warn "exit";
    return $r;
}
