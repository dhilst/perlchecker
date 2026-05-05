# =============================================================
# Round 86: Compound assign + branch interaction path stress
# =============================================================
# Functions using compound assignment operators (+=, -=, *=, .=)
# inside different branches of if/elsif/else, creating paths where
# the final value of a variable depends on which compound operations
# were applied along the execution path.

# --- Function 1: += in one branch, -= in another ---
# An accumulator starts at a known value, then passes through two
# sequential if/else blocks. Each block either adds or subtracts
# depending on conditions, creating 4 distinct final values.
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 10 && $y >= 1 && $y <= 10
# post: $result >= -8 && $result <= 30
sub compound_add_sub_branches {
    my ($x, $y) = @_;
    my $acc = 10;
    if ($x > 5) {
        $acc += $x;
    } else {
        $acc -= $x;
    }
    if ($y > 5) {
        $acc += $y;
    } else {
        $acc -= $y;
    }
    return $acc;
}

# --- Function 2: Chained compound assigns with elsif ---
# Uses += -= *= in elsif chains. The final value depends on which
# single branch was taken. Tests that the verifier correctly merges
# the disjoint paths at the join point.
# sig: (I64, I64) -> I64
# pre: $n >= 0 && $n <= 20 && $base >= 1 && $base <= 5
# post: $result >= 0 && $result <= 105
sub compound_elsif_chain {
    my ($n, $base) = @_;
    my $r = $base;
    if ($n > 15) {
        $r *= $n;
    } elsif ($n > 10) {
        $r += $n;
    } elsif ($n > 5) {
        $r += $n - 3;
    } else {
        $r -= 1;
    }
    return $r;
}

# --- Function 3: Sequential compound assigns with intervening conditionals ---
# Multiple compound assign statements separated by conditionals that
# may modify the variable. The verifier must track how each += or -=
# accumulates along the path.
# sig: (I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 5 && $b >= 0 && $b <= 5 && $c >= 0 && $c <= 5
# post: $result >= 1 && $result <= 26
sub sequential_compound_conditional {
    my ($a, $b, $c) = @_;
    my $val = 1;
    $val += $a;
    if ($val > 3) {
        $val += $b;
    } else {
        $val += 1;
    }
    $val += $c;
    if ($val > 10) {
        $val -= 2;
    }
    return $val;
}

# --- Function 4: Compound assigns inside a bounded loop with branching ---
# Each loop iteration uses += or -= depending on a condition that
# changes with the loop variable. The compound ops accumulate across
# 3 unrolled iterations with 2 paths each = 8 total paths.
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= -6 && $result <= 30
sub compound_in_loop_branch {
    my ($x) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        if ($x > $i * 3) {
            $acc += $x;
        } else {
            $acc -= 2;
        }
    }
    return $acc;
}

# --- Function 5: Compound assign with nested branches and multiple variables ---
# Two accumulators are modified with different compound ops depending
# on nested conditions. The final result combines both, requiring the
# verifier to track two parallel accumulation chains across 4 paths.
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 8 && $y >= 1 && $y <= 8
# post: $result >= 2 && $result <= 32
sub dual_compound_nested {
    my ($x, $y) = @_;
    my $left = 0;
    my $right = 0;
    if ($x > 4) {
        $left += $x;
        if ($y > 4) {
            $right += $x + $y;
        } else {
            $right += $y;
        }
    } else {
        $left += 1;
        if ($y > 4) {
            $right += $y;
        } else {
            $right += 1;
        }
    }
    my $result = $left + $right;
    return $result;
}
