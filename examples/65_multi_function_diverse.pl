# =============================================================
# Round 65: Multi-function diverse feature verification
# =============================================================
# A file with 8 functions that each exercise a DIFFERENT feature
# combination, verifying the checker handles multiple independent
# verification tasks in one file efficiently. Each function uses
# a distinct subset of the supported Perl features.

# --- Function 1: Pure arithmetic + ternary (no loops) ---
# Uses multiplication, addition, and ternary operator with nested
# conditional expressions. No loops, no side effects.
# sig: (I64, I64) -> I64
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10
# post: $result >= 2 && $result <= 110
sub arith_ternary_only {
    my ($a, $b) = @_;
    my $prod = $a * $b;
    my $sum = $a + $b;
    my $r = ($prod > 50) ? $prod : $sum;
    $r = ($r > 100) ? 100 + $a : $r;
    return $r;
}

# --- Function 2: For-loop + last + accumulator ---
# Bounded loop with early exit via last. Accumulates values until
# a threshold is reached. Tests loop unrolling + last interaction.
# sig: (I64, I64) -> I64
# pre: $step >= 1 && $step <= 5 && $limit >= 5 && $limit <= 20
# post: $result >= 1 && $result <= 25
sub loop_last_accumulate {
    my ($step, $limit) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < 5; $i++) {
        $acc = $acc + $step;
        last if ($acc >= $limit);
    }
    return $acc;
}

# --- Function 3: String ops + conditional branches ---
# Uses length, contains, concat, and branches on string properties.
# No loops, pure string reasoning with path divergence.
# sig: (Str) -> I64
# pre: length($s) >= 3 && length($s) <= 8
# post: $result >= 3 && $result <= 11
sub string_branch_ops {
    my ($s) = @_;
    my $len = length($s);
    my $has_x = contains($s, "x");
    my $r;
    if ($has_x == 1) {
        my $ext = $s . "end";
        $r = length($ext);
    } else {
        $r = $len;
    }
    return $r;
}

# --- Function 4: Bitwise + shifts + comparison ---
# Uses &, |, ^, <<, and comparisons to compute a masked/shifted
# result. No loops, purely bitwise computation with branches.
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 255 && $y >= 0 && $y <= 255
# post: $result >= 0 && $result <= 255
sub bitwise_shift_compare {
    my ($x, $y) = @_;
    my $masked = $x & 15;
    my $shifted = $masked << 2;
    my $combined = $shifted | ($y & 3);
    my $r;
    if ($combined > 255) {
        $r = 255;
    } else {
        $r = $combined;
    }
    return $r;
}

# --- Function 5: Array access + loop + next ---
# Reads array length in a loop, skipping even indices with next.
# Accesses array elements at odd indices but returns the bounded
# count of visited odd indices. Tests array + loop + next interaction.
# sig: (Array<I64>, I64) -> I64
# pre: scalar(@arr) >= 4 && $n >= 2 && $n <= 4
# post: $result >= 1 && $result <= 2
sub array_loop_next {
    my ($arr, $n) = @_;
    my $count = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i % 2 == 0);
        my $elem = $arr[$i];
        $count = $count + 1;
    }
    return $count;
}

# --- Function 6: Hash + conditional + die guard ---
# Reads a hash value, uses die as a guard, then branches on the
# value. Tests hash access + die path pruning + conditional.
# sig: (Hash<Str, I64>, Str) -> I64
# pre: $h{$k} >= 1 && $h{$k} <= 100
# post: $result >= 1 && $result <= 100
sub hash_conditional_guard {
    my ($h, $k) = @_;
    my $val = $h{$k};
    die "value must be positive" if ($val <= 0);
    my $r;
    if ($val > 50) {
        $r = $val;
    } else {
        $r = $val;
    }
    return $r;
}

# --- Function 7: Do-while + unless ---
# Uses do-while loop with an unless conditional inside the body.
# Tests do-while loop unrolling + unless branching.
# sig: (I64) -> I64
# pre: $x >= 10 && $x <= 30
# post: $result >= 0 && $result < 10
sub dowhile_unless_reduce {
    my ($x) = @_;
    my $r = $x;
    do {
        unless ($r < 5) {
            $r = $r - 7;
        }
    } while ($r >= 10);
    return $r;
}

# --- Function 8: Multiple return paths with die guards + inc/dec ---
# Uses multiple die guards to prune paths, increment/decrement,
# and early return via ternary. Tests die pruning + inc/dec + ternary.
# sig: (I64, I64) -> I64
# pre: $a >= 1 && $a <= 20 && $b >= 1 && $b <= 20
# post: $result >= 1 && $result <= 21
sub multi_return_die_inc {
    my ($a, $b) = @_;
    die "a must be positive" if ($a <= 0);
    die "b must be positive" if ($b <= 0);
    my $r = $a;
    $r++;
    my $cmp = ($r > $b) ? $r : $b;
    return $cmp;
}
