# Round 49: Path expansion stress tests
# Exercises the symbolic execution engine with many branching paths,
# combining arithmetic, ternary, loops with last/next, and array ops.

# Complex path stress test: nested conditions with caps
# This creates 2^4 = 16 paths through the function
# sig: (Int, Int, Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10 && $c >= 0 && $c <= 10 && $d >= 0 && $d <= 10
# post: $result >= 0 && $result <= 20
sub sum_with_caps {
    my ($a, $b, $c, $d) = @_;
    my $sum = 0;
    if ($a > 5) {
        $sum = $sum + 5;
    } else {
        $sum = $sum + $a;
    }
    if ($b > 5) {
        $sum = $sum + 5;
    } else {
        $sum = $sum + $b;
    }
    if ($c > 5) {
        $sum = $sum + 5;
    } else {
        $sum = $sum + $c;
    }
    if ($d > 5) {
        $sum = $sum + 5;
    } else {
        $sum = $sum + $d;
    }
    return $sum;
}

# Loop with early exit via last + ternary transform on array elements
# sig: (Array<Int>, Int) -> Int
# pre: $len >= 1 && $len <= 4
# post: $result >= 0 && $result <= 1
sub search_with_transform {
    my ($arr, $len) = @_;
    my $found = 0;
    my $i;
    for ($i = 0; $i < $len; $i = $i + 1) {
        my $val = ($arr[$i] > 3) ? $arr[$i] - 3 : $arr[$i];
        if ($val == 0) {
            $found = 1;
            last;
        }
    }
    return $found;
}

# Nested ternary chain creating complex constraint paths
# sig: (Int) -> Int
# pre: $x >= -100 && $x <= 100
# post: $result >= 0 && $result <= 4
sub classify {
    my ($x) = @_;
    my $r = ($x < -50) ? 0 : (($x < 0) ? 1 : (($x == 0) ? 2 : (($x <= 50) ? 3 : 4)));
    return $r;
}

# Combined arithmetic + ternary paths with loop and next
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 0
sub sum_transformed_odds {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i = $i + 1) {
        next if ($i % 2 == 0);
        my $contrib = ($i > 3) ? 3 : $i;
        $sum = $sum + $contrib;
    }
    return $sum;
}

# Multi-branch classification with early return
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 0 && $result <= 3
sub quadrant {
    my ($a, $b) = @_;
    if ($a <= 5) {
        if ($b <= 5) {
            return 0;
        } else {
            return 1;
        }
    } else {
        if ($b <= 5) {
            return 2;
        } else {
            return 3;
        }
    }
}
