# =============================================================
# Round 76: Do-while + break flag path stress
# =============================================================
# Functions using do-while loops with complex exit conditions,
# combining last (break) with the loop's own condition, creating
# dual-exit-path patterns where the verifier must track both the
# break flag and the loop condition.

# --- Function 1: Do-while with conditional last creating dual exit ---
# The loop can exit via last (when acc exceeds threshold) OR via
# the while condition (when i reaches n). Both exits lead to
# different final states that must satisfy the postcondition.
# sig: (I64, I64) -> I64
# pre: $n >= 2 && $n <= 4 && $threshold >= 3 && $threshold <= 8
# post: $result >= 2 && $result <= 10
sub dowhile_dual_exit {
    my ($n, $threshold) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        $acc += 2;
        $i++;
        last if ($acc >= $threshold);
    } while ($i < $n);
    return $acc;
}

# --- Function 2: Do-while with multi-variable condition and last ---
# Loop condition depends on two variables (i < limit && acc < cap).
# Inside, last fires on a third condition. Three possible exit
# triggers stress the path tracker: last, i >= limit, acc >= cap.
# sig: (I64, I64) -> I64
# pre: $limit >= 2 && $limit <= 4 && $cap >= 4 && $cap <= 10
# post: $result >= 2 && $result <= 10
sub dowhile_multi_cond_last {
    my ($limit, $cap) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        $i++;
        $acc += $i;
        last if ($i >= 3);
    } while ($i < $limit && $acc < $cap);
    return $acc;
}

# --- Function 3: Do-while with conditional assignment + last ---
# Each iteration conditionally assigns different increments based
# on parity of the counter. Last breaks when a running total
# crosses a dynamic threshold derived from the input parameter.
# Iter 0: inc=2, total=2; Iter 1: inc=1, total=3; Iter 2: inc=2, total=5;
# Iter 3: inc=1, total=6; Iter 4: inc=2, total=8.
# Break when total >= x. With x in [3,5], breaks at total in [3,5].
# With x>5 won't break early, loop ends after 5 iters with total=8.
# sig: (I64) -> I64
# pre: $x >= 3 && $x <= 5
# post: $result >= 3 && $result <= 8
sub dowhile_cond_assign_last {
    my ($x) = @_;
    my $total = 0;
    my $i = 0;
    do {
        my $inc = ($i % 2 == 0) ? 2 : 1;
        $total += $inc;
        $i++;
        last if ($total >= $x);
    } while ($i < 5);
    return $total;
}

# --- Function 4: Do-until with next for skip patterns ---
# Uses do-until (negated condition) where next skips the
# accumulation for even iterations, creating paths where some
# iterations contribute and others don't.
# sig: (I64) -> I64
# pre: $n >= 3 && $n <= 5
# post: $result >= 1 && $result <= 3
sub dountil_next_skip {
    my ($n) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        $i++;
        next if ($i % 2 == 0);
        $acc++;
    } until ($i >= $n);
    return $acc;
}

# --- Function 5: Do-while with nested if and last on flag ---
# A flag variable is set inside a nested conditional. The last
# statement checks the flag, creating a pattern where the break
# depends on path-sensitive state from earlier in the iteration.
# With a in [2,4] and b in [1,2]:
#   Threshold = a*b. Loop adds a each iter, breaks when sum > threshold.
#   Min: a=2, b=2, threshold=4. iter1: sum=2, iter2: sum=4, iter3: sum=6>4, break. result=6.
#   But could also: a=2, b=1, threshold=2. iter1: sum=2, 2>2? no. iter2: sum=4, 4>2 yes. result=4.
#   Max: a=4, b=2, threshold=8. iter1:4, iter2:8, iter3:12>8, break. result=12.
#   If loop runs all 5: a=4,b=2 → 12 on iter3.
# sig: (I64, I64) -> I64
# pre: $a >= 2 && $a <= 4 && $b >= 1 && $b <= 2
# post: $result >= 4 && $result <= 12
sub dowhile_flag_break {
    my ($a, $b) = @_;
    my $sum = 0;
    my $i = 0;
    my $done = 0;
    do {
        $i++;
        $sum += $a;
        if ($sum > $a * $b) {
            $done = 1;
        }
        last if ($done == 1);
    } while ($i < 5);
    return $sum;
}
