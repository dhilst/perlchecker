# =============================================================
# Round 63: Increment/decrement path stress validation
# =============================================================
# Exercises $x++ and $x-- as primary loop mechanisms in path-heavy
# scenarios: nested conditionals, do-while loops, early exits,
# and ternary combinations that stress symbolic execution.

# --- Countdown with branching: loop decrement with conditional paths ---
# A for-loop uses $i++ while the body branches on even/odd parity,
# accumulating into $acc. With $n in [1,4], the loop runs up to 4
# iterations creating 2^4 = 16 paths (bounded by unroll limit).
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 4
# post: $result >= 1 && $result <= 10
sub inc_branch_accumulate {
    my ($n) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        if ($i % 2 == 0) {
            $acc = $acc + 2;
        } else {
            $acc++;
        }
    }
    $acc++;
    return $acc;
}

# --- While-loop with decrement and early last ---
# Uses $x-- in a while loop with a conditional last that exits
# early when a threshold is hit. Precondition bounds ensure the
# loop terminates within unroll limits and postcondition is tight.
# sig: (I64) -> I64
# pre: $x >= 3 && $x <= 5
# post: $result >= 1 && $result <= 3
sub dec_while_early_exit {
    my ($x) = @_;
    my $count = 0;
    while ($x > 0) {
        $count++;
        $x--;
        last if ($x <= 2);
    }
    return $count;
}

# --- Nested conditionals with inc/dec on separate paths ---
# Two parameters create a 2x2 branch matrix. Each branch path
# uses either ++ or -- to adjust a result variable differently,
# then a final inc ensures minimum is above zero.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 1 && $result <= 12
sub nested_inc_dec_matrix {
    my ($a, $b) = @_;
    my $r = $a;
    if ($a > 5) {
        if ($b > 5) {
            $r++;
            $r++;
        } else {
            $r--;
            $r++;
            $r++;
        }
    } else {
        if ($b > 5) {
            $r++;
        } else {
            $r++;
        }
    }
    return $r;
}

# --- For-loop with ternary and inc/dec ---
# Uses $i++ as the loop step. Inside, a ternary selects between
# incrementing or decrementing $val based on loop index parity.
# Bounded loop creates path explosion manageable by unroll.
# sig: (I64) -> I64
# pre: $start >= 0 && $start <= 5
# post: $result >= 2 && $result <= 7
sub for_ternary_inc_dec {
    my ($start) = @_;
    my $val = $start;
    my $i;
    for ($i = 0; $i < 4; $i++) {
        $val = ($i % 2 == 0) ? $val + 1 : $val - 1;
    }
    $val++;
    $val++;
    return $val;
}

# --- Multi-dec countdown with conditional next ---
# A while loop uses $counter-- with a conditional next that skips
# accumulation for certain values, creating divergent paths per
# iteration. Tests interaction of next + decrement.
# sig: (I64) -> I64
# pre: $n >= 3 && $n <= 5
# post: $result >= 1 && $result <= 3
sub dec_while_with_next {
    my ($n) = @_;
    my $counter = $n;
    my $acc = 0;
    while ($counter > 0) {
        $counter--;
        next if ($counter % 2 == 0);
        $acc++;
    }
    return $acc;
}
