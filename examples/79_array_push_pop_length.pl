# =============================================================
# Round 79: Array push/pop + length path stress
# =============================================================
# Functions that use push/pop to modify arrays, then branch on
# scalar(@arr) length and popped values, creating paths where the
# verifier must track how array length and contents change through
# sequences of operations and branch on the resulting state.

# --- Function 1: Push sequence then branch on length and popped values ---
# Pushes three computed values, pops two, then creates 4 paths by
# branching on the popped values relative to thresholds. The verifier
# must track what was stored at each index through push/pop.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10 && $c >= 0 && $c <= 10
# post: $result >= 1 && $result <= 4
sub push_pop_classify {
    my ($arr, $n, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $top = pop(@arr);
    my $second = pop(@arr);
    if ($top > 5) {
        if ($second > 5) {
            return 4;
        } else {
            return 3;
        }
    } else {
        if ($second > 5) {
            return 2;
        } else {
            return 1;
        }
    }
}

# --- Function 2: Multiple pushes with length-driven branching ---
# Pushes a variable number of elements (based on input bounds), then
# uses scalar(@arr) to determine which region the final length falls in.
# Creates paths combining push count with length classification.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $x >= 1 && $x <= 5 && $y >= 1 && $y <= 5 && $z >= 1 && $z <= 5
# post: $result >= 1 && $result <= 3
sub push_triple_branch_length {
    my ($arr, $n, $x, $y, $z) = @_;
    push(@arr, $x);
    push(@arr, $y);
    push(@arr, $z);
    my $len = scalar(@arr);
    if ($len > 2) {
        return 3;
    } elsif ($len > 1) {
        return 2;
    } else {
        return 1;
    }
}

# --- Function 3: Push then pop sequence with value-dependent paths ---
# Pushes 4 values, pops them all back, and classifies the result based
# on sums and comparisons of the popped values. Creates 8 paths through
# the 3 nested conditions on popped values.
# sig: (Array<Int>, Int, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $w >= 0 && $w <= 5 && $x >= 0 && $x <= 5 && $y >= 0 && $y <= 5 && $z >= 0 && $z <= 5
# post: $result >= 3 && $result <= 10
sub push_four_pop_four_classify {
    my ($arr, $n, $w, $x, $y, $z) = @_;
    push(@arr, $w);
    push(@arr, $x);
    push(@arr, $y);
    push(@arr, $z);
    my $d = pop(@arr);
    my $c = pop(@arr);
    my $b = pop(@arr);
    my $a = pop(@arr);
    my $sum = 0;
    if ($d > $c) {
        $sum = $sum + $d;
    } else {
        $sum = $sum + $c;
    }
    if ($b > $a) {
        $sum = $sum + $b;
    } else {
        $sum = $sum + $a;
    }
    if ($sum > 5) {
        return $sum;
    } else {
        return $sum + 3;
    }
}

# --- Function 4: Push/pop with length and element access paths ---
# Pushes elements, reads array at specific indices, pops one, then
# creates paths branching on both the popped value and array element
# values. Stresses the verifier's tracking of stored values.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 5 && $b >= 1 && $b <= 5 && $c >= 1 && $c <= 5
# post: $result >= 2 && $result <= 12
sub push_access_pop_paths {
    my ($arr, $n, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $top = pop(@arr);
    my $first = $arr[0];
    my $second = $arr[1];
    my $len = scalar(@arr);
    if ($first > 3) {
        if ($second > 3) {
            return $first + $second + $len;
        } else {
            return $first + $len;
        }
    } else {
        if ($top > 3) {
            return $top + $len;
        } else {
            return $len + $top;
        }
    }
}

# --- Function 5: Push accumulation then pop-driven state machine ---
# Pushes values based on input, pops them in sequence, using each
# popped value to update a state variable through branches. Creates
# many paths as each pop creates an independent branch point.
# sig: (Array<Int>, Int, Int, Int, Int) -> Int
# pre: scalar(@arr) == $n && $n == 0 && $p >= 0 && $p <= 10 && $q >= 0 && $q <= 10 && $r >= 0 && $r <= 10
# post: $result >= 0 && $result <= 6
sub pop_driven_state {
    my ($arr, $n, $p, $q, $r) = @_;
    push(@arr, $p);
    push(@arr, $q);
    push(@arr, $r);
    my $state = 0;
    my $v3 = pop(@arr);
    if ($v3 > 5) {
        $state = $state + 2;
    } else {
        $state = $state + 1;
    }
    my $v2 = pop(@arr);
    if ($v2 > 5) {
        $state = $state + 2;
    } else {
        $state = $state + 1;
    }
    my $v1 = pop(@arr);
    if ($v1 > 5) {
        $state = $state + 2;
    } else {
        $state = $state + 1;
    }
    return $state;
}
