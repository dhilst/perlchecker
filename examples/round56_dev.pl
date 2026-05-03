# =============================================================
# Round 56: Ternary chain path explosion stress
# =============================================================
# Exercises path expansion with deeply nested ternary expressions
# (3-5 levels deep) as value expressions and in conditions.
# Each nested ternary doubles the path count: a 5-deep ternary
# creates up to 32 paths from a single expression. Combining
# multiple ternary assignments multiplies paths further.

# --- 4-level nested ternary classification ---
# Classifies input into one of 5 ranges using a 4-deep ternary chain.
# Creates 5 distinct paths through a single expression.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result >= 1 && $result <= 5
sub classify_range {
    my ($x) = @_;
    my $r = ($x < 20) ? 1 : (($x < 40) ? 2 : (($x < 60) ? 3 : (($x < 80) ? 4 : 5)));
    return $r;
}

# --- Two interacting ternary chains with arithmetic ---
# Two 3-deep ternary assignments whose results are combined,
# creating 4 * 4 = 16 path combinations through arithmetic.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 30 && $b >= 0 && $b <= 30
# post: $result >= 2 && $result <= 80
sub ternary_pair_combine {
    my ($a, $b) = @_;
    my $x = ($a < 10) ? 1 : (($a < 20) ? $a : (($a < 25) ? 30 : 40));
    my $y = ($b < 10) ? 1 : (($b < 20) ? $b : (($b < 25) ? 30 : 40));
    my $r = $x + $y;
    return $r;
}

# --- Ternary in loop bound controlling iteration count ---
# A ternary determines how many times the loop runs, then inside
# the loop another ternary picks the increment. Paths = bound_paths * (iter_paths ^ iters).
# sig: (Int, Int) -> Int
# pre: $mode >= 0 && $mode <= 2 && $val >= 1 && $val <= 5
# post: $result >= 0 && $result <= 25
sub ternary_loop_bound {
    my ($mode, $val) = @_;
    my $limit = ($mode == 0) ? 2 : (($mode == 1) ? 3 : 5);
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $limit; $i++) {
        my $inc = ($val > $i) ? $val : 1;
        $sum = $sum + $inc;
    }
    return $sum;
}

# --- 5-deep ternary scoring with compound postcondition ---
# A single 5-level ternary assigns a score. The postcondition
# verifies membership in the exact set of possible values.
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 50
# post: $result >= 0 && $result <= 50
sub deep_score {
    my ($n) = @_;
    my $s = ($n < 10) ? 0 : (($n < 20) ? 10 : (($n < 30) ? 20 : (($n < 40) ? 30 : (($n < 50) ? 40 : 50))));
    return $s;
}

# --- Cascading ternary assignments where each depends on prior ---
# Three sequential ternary assignments where each uses the result
# of the previous one, creating a chain of dependent ITE trees.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 20
# post: $result >= 1 && $result <= 6
sub cascading_ternaries {
    my ($x) = @_;
    my $a = ($x < 5) ? 1 : (($x < 10) ? 2 : (($x < 15) ? 3 : 4));
    my $b = ($a < 3) ? $a : (($a == 3) ? 4 : 5);
    my $c = ($b <= 2) ? $b : (($b <= 4) ? $b + 1 : 6);
    return $c;
}
