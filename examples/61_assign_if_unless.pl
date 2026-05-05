# =============================================================
# Round 61: Assign-if/unless statement modifier path stress
# =============================================================
# Exercises the `$x = EXPR if (COND)` and `$x = EXPR unless (COND)`
# statement modifier syntax. These create implicit branching paths
# where the assignment may or may not execute, stressing the
# symbolic execution engine's path expansion. Multiple sequential
# conditional assignments produce exponential path combinations.

# --- Basic assign-if modifier creates two paths ---
# When the condition is true, $r gets the new value; otherwise
# it keeps its initial value. Both paths must satisfy the post.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 100
# post: $result >= 0 && $result <= 100
sub assign_if_basic {
    my ($x) = @_;
    my $r = $x;
    $r = 50 if ($x > 80);
    return $r;
}

# --- Basic assign-unless modifier creates two paths ---
# Unless the flag is set, we override the value. If flag is set
# (i.e. condition is true), the assignment does NOT execute.
# sig: (I64, I64) -> I64
# pre: $val >= 0 && $val <= 50 && ($flag == 0 || $flag == 1)
# post: $result >= 0 && $result <= 50
sub assign_unless_basic {
    my ($val, $flag) = @_;
    my $r = $val;
    $r = 0 unless ($flag == 1);
    return $r;
}

# --- Sequential conditional assigns: 4 paths ---
# Two assign-if modifiers in sequence create 2x2 = 4 paths.
# The verifier must reason through all combinations to prove
# the postcondition holds universally.
# sig: (I64, I64) -> I64
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10
# post: $result >= 1 && $result <= 20
sub sequential_assign_if {
    my ($a, $b) = @_;
    my $r = $a + $b;
    $r = $a if ($b > 7);
    $r = $b if ($a > 7);
    return $r;
}

# --- Mixed if/unless with accumulation: 8 paths ---
# Three conditional assignments produce 2^3 = 8 execution paths.
# Each path either applies or skips each modifier. Postcondition
# must hold across all eight paths.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 20
# post: $result >= 0 && $result <= 30
sub triple_conditional_assign {
    my ($n) = @_;
    my $r = $n;
    $r = $r + 5 if ($n > 5);
    $r = $r + 3 unless ($n > 15);
    $r = 10 if ($n == 0);
    return $r;
}

# --- Conditional assigns inside a bounded loop with early exit ---
# Combines for-loop, last-if, and assign-if modifiers.
# The loop accumulates but conditionally resets, testing
# interaction of loop unrolling with modifier-generated branches.
# sig: (I64) -> I64
# pre: $start >= 0 && $start <= 5
# post: $result >= 0 && $result <= 15
sub loop_with_conditional_assign {
    my ($start) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        $sum = $sum + $start + $i;
        $sum = 0 if ($sum > 15);
        last if ($i == 2);
    }
    return $sum;
}
