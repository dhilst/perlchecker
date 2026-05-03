# =============================================================
# Round 53: Multi-return with complex guards path stress
# =============================================================
# Exercises path expansion with multiple return statements guarded
# by different conditions (return-if, return-unless, sequential
# if-return chains). Each function has 3-5 distinct exit paths
# that the verifier must prove independently.

# --- Cascading return-if guards: 4 exit paths ---
# Each guard peels off a range; the final return covers the rest.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result >= 1 && $result <= 4
sub classify_quartile {
    my ($x) = @_;
    return 1 if ($x <= 25);
    return 2 if ($x <= 50);
    return 3 if ($x <= 75);
    return 4;
}

# --- Mixed return-if and return-unless: 5 exit paths ---
# Guards combine positive and negative conditions on two inputs.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20
# post: $result >= 0 && $result <= 40
sub guarded_sum {
    my ($a, $b) = @_;
    return 0 if ($a == 0);
    return $b unless ($a > $b);
    return $a if ($b == 0);
    return $a + $b unless ($a + $b > 30);
    return 30;
}

# --- Arithmetic with guard returns and die for unreachable path ---
# Computes a bounded ratio with early returns for edge cases.
# sig: (Int, Int) -> Int
# pre: $n >= 0 && $n <= 50 && $d >= 1 && $d <= 10
# post: $result >= 0 && $result <= 50
sub bounded_ratio {
    my ($n, $d) = @_;
    die "unreachable: d is positive" if ($d <= 0);
    return 0 if ($n == 0);
    return $n if ($d == 1);
    return 50 unless ($n / $d <= 50);
    my $r = $n / $d;
    return $r;
}

# --- Absolute difference with directional guards: 4 paths ---
# sig: (Int, Int) -> Int
# pre: $x >= -50 && $x <= 50 && $y >= -50 && $y <= 50
# post: $result >= 0 && $result <= 100
sub abs_diff {
    my ($x, $y) = @_;
    return 0 if ($x == $y);
    return $x - $y if ($x > $y);
    return $y - $x unless ($x > $y);
    die "unreachable";
}

# --- Multi-condition priority classifier: 5 exit paths ---
# Priority is determined by thresholds on two variables.
# sig: (Int, Int) -> Int
# pre: $urgency >= 0 && $urgency <= 10 && $impact >= 0 && $impact <= 10
# post: $result >= 1 && $result <= 5
sub priority_level {
    my ($urgency, $impact) = @_;
    return 5 if ($urgency >= 9);
    return 4 if ($impact >= 9);
    return 3 if ($urgency + $impact >= 12);
    return 2 unless ($urgency + $impact < 6);
    return 1;
}
