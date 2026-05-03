# =============================================================
# Round 81: scalar(@arr) in complex expressions path stress
# =============================================================
# Functions using scalar(@arr) as part of arithmetic expressions,
# loop bounds, and conditional comparisons, creating paths where
# array length interacts with integer computations and the verifier
# must track how length evolves through push/pop operations.

# --- Function 1: Arithmetic expressions with scalar(@arr) ---
# Pushes elements then computes expressions using scalar(@arr) - 1,
# scalar(@arr) * 2, and branches on the computed values.
# After 3 pushes: len=3, so len-1=2, len*2=6, sum=8.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 5 && $b >= 1 && $b <= 5 && $c >= 1 && $c <= 5
# post: $result == 8
sub scalar_arithmetic {
    my ($arr, $n, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $len_minus_one = scalar(@arr) - 1;
    my $len_times_two = scalar(@arr) * 2;
    my $result = $len_minus_one + $len_times_two;
    return $result;
}

# --- Function 2: Branching on scalar(@arr) comparisons ---
# Pushes 4 elements, pops one, then branches on comparisons involving
# scalar(@arr) and the popped value. The length after 4 push + 1 pop = 3.
# Creates 4 paths depending on relationships between popped value and length.
# sig: (Array<Int>, Int, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 5 && $b >= 1 && $b <= 5 && $c >= 1 && $c <= 5 && $d >= 1 && $d <= 5
# post: $result >= 1 && $result <= 8
sub scalar_branch_paths {
    my ($arr, $n, $a, $b, $c, $d) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    push(@arr, $d);
    my $top = pop(@arr);
    my $len = scalar(@arr);
    if ($top > $len) {
        if ($top > 4) {
            return $top + $len;
        } else {
            return $top;
        }
    } else {
        if ($top > 1) {
            return $len;
        } else {
            return 1;
        }
    }
}

# --- Function 3: scalar(@arr) as loop bound ---
# Pushes 3 elements, then iterates scalar(@arr) times accumulating
# a counter. The loop bound depends on array length tracked symbolically.
# After 3 pushes: len=3, loop runs 3 times, acc=3.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10 && $c >= 1 && $c <= 10
# post: $result == 3
sub scalar_loop_bound {
    my ($arr, $n, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $bound = scalar(@arr);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $bound; $i++) {
        $acc = $acc + 1;
    }
    return $acc;
}

# --- Function 4: Combined push/pop with scalar(@arr) in arithmetic ---
# Pushes 4 elements, pops 2, then uses scalar(@arr) in expressions
# combined with popped values to create multi-path branching.
# After 4 push + 2 pop: len=2. top1+top2 in [2..20].
# If sum > len (sum>2): return sum + len = sum + 2, range [5..22]
# If sum <= len (sum<=2, only when both are 1): return len * 3 = 6
# sig: (Array<Int>, Int, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 5 && $b >= 1 && $b <= 5 && $c >= 1 && $c <= 5 && $d >= 1 && $d <= 5
# post: $result >= 5 && $result <= 12
sub scalar_pop_arithmetic {
    my ($arr, $n, $a, $b, $c, $d) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    push(@arr, $d);
    my $top1 = pop(@arr);
    my $top2 = pop(@arr);
    my $len = scalar(@arr);
    my $sum = $top1 + $top2;
    if ($sum > $len) {
        return $sum + $len;
    } else {
        return $len * 3;
    }
}

# --- Function 5: Nested conditions on scalar(@arr) expressions ---
# Pushes 3 elements, then uses scalar(@arr) in multiple arithmetic
# expressions and branches on the results combined with input values.
# After 3 pushes: len=3. Tests len+x, len*y creating 4 paths.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $x >= 1 && $x <= 5 && $y >= 1 && $y <= 5 && $z >= 1 && $z <= 5
# post: $result >= 3 && $result <= 15
sub scalar_nested_paths {
    my ($arr, $n, $x, $y, $z) = @_;
    push(@arr, $x);
    push(@arr, $y);
    push(@arr, $z);
    my $len = scalar(@arr);
    my $sum = $len + $x;
    my $prod = $len * $y;
    if ($sum > 5) {
        if ($prod > 9) {
            return $prod;
        } else {
            return $sum;
        }
    } else {
        if ($prod > 6) {
            return $prod;
        } else {
            return $len + $z;
        }
    }
}
