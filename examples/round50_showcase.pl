# =============================================================
# Round 50: Consolidation Showcase
# =============================================================
# Exercises the full power of perlchecker across all major features:
# arithmetic, branches, loops, early exit, ternary, bitwise ops,
# arrays (push/pop/scalar), strings, function calls, and modifiers.

# --- Feature: Clamping with early return + unless modifier ---
# sig: (Int, Int, Int) -> Int
# pre: $target >= 0 && $target <= 10 && $lo >= 0 && $hi >= $lo && $hi <= 10
# post: $result >= $lo && $result <= $hi
sub clamp_full {
    my ($target, $lo, $hi) = @_;
    return $lo if ($target < $lo);
    return $hi unless ($target <= $hi);
    return $target;
}

# --- Feature: Array push/pop round-trip ---
# sig: (Array<Int>, Int, Int, Int) -> Int
# pre: scalar(@arr) == 0 && $a >= 0 && $a <= 100 && $b >= 0 && $b <= 100 && $c >= 0 && $c <= 100
# post: $result == $a + $b + $c
sub stack_sum {
    my ($arr, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $x = pop(@arr);
    my $y = pop(@arr);
    my $z = pop(@arr);
    my $sum = $x + $y + $z;
    return $sum;
}

# --- Feature: String operations with length and starts_with ---
# sig: (Str) -> Int
# pre: length($s) >= 5 && starts_with($s, "hello") == 1
# post: $result >= 5
sub hello_length {
    my ($s) = @_;
    my $r = length($s);
    return $r;
}

# --- Feature: Bitwise manipulation round-trip ---
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result == $x
sub bit_roundtrip {
    my ($x) = @_;
    my $high = ($x >> 4) & 15;
    my $low = $x & 15;
    my $r = ($high << 4) | $low;
    return $r;
}

# --- Feature: Loop + conditions + last + ternary ---
# sig: (Int, Int) -> Int
# pre: $n >= 1 && $n <= 4 && $threshold >= 0 && $threshold <= 20
# post: $result >= 0 && $result <= $threshold
sub complex_accumulate {
    my ($n, $threshold) = @_;
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        my $val = ($i % 2 == 0) ? $i * 2 : $i;
        $acc += $val;
        last if ($acc >= $threshold);
    }
    my $r = ($acc > $threshold) ? $threshold : $acc;
    return $r;
}

# --- Feature: Intra-file function call + elsif chain ---
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result >= 0 && $result <= 100
sub identity {
    my ($x) = @_;
    return $x;
}

# sig: (Int) -> Int
# pre: $score >= 0 && $score <= 100
# post: $result >= 1 && $result <= 4
sub grade_band {
    my ($score) = @_;
    my $s = identity($score);
    if ($s >= 90) {
        return 4;
    } elsif ($s >= 70) {
        return 3;
    } elsif ($s >= 50) {
        return 2;
    } else {
        return 1;
    }
}

# --- Feature: Loop with next (skip) + modulo ---
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 0
sub sum_odd_indices {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i % 2 == 0);
        $sum += $i;
    }
    return $sum;
}

# --- Feature: Die guarded by precondition (unreachable) ---
# sig: (Int, Int) -> Int
# pre: $b != 0 && $a >= 0 && $a <= 100 && $b >= 1 && $b <= 10
# post: $result >= 0
sub safe_divide {
    my ($a, $b) = @_;
    die "division by zero" if ($b == 0);
    my $r = $a / $b;
    return $r;
}
