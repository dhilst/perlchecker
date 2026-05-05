# =============================================================
# Round 51: Deep nested conditionals path explosion stress
# =============================================================
# Exercises the symbolic execution engine with 2^5 = 32 path
# explosion scenarios. Uses die to prune impossible paths and
# combines ternary, arithmetic, and conditional branching.

# --- 5 sequential if/else branches, each caps a component ---
# Creates 2^5 = 32 paths through the function
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20 && $c >= 0 && $c <= 20 && $d >= 0 && $d <= 20 && $e >= 0 && $e <= 20
# post: $result >= 0 && $result <= 50
sub five_way_cap {
    my ($a, $b, $c, $d, $e) = @_;
    my $sum = 0;
    if ($a > 10) {
        $sum = $sum + 10;
    } else {
        $sum = $sum + $a;
    }
    if ($b > 10) {
        $sum = $sum + 10;
    } else {
        $sum = $sum + $b;
    }
    if ($c > 10) {
        $sum = $sum + 10;
    } else {
        $sum = $sum + $c;
    }
    if ($d > 10) {
        $sum = $sum + 10;
    } else {
        $sum = $sum + $d;
    }
    if ($e > 10) {
        $sum = $sum + 10;
    } else {
        $sum = $sum + $e;
    }
    return $sum;
}

# --- Die-as-assert to prune impossible paths ---
# The die statements reduce the path space by eliminating branches
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 100 && $y >= 1 && $y <= 100
# post: $result >= 2 && $result <= 200
sub guarded_sum {
    my ($x, $y) = @_;
    die "x must be positive" if ($x <= 0);
    die "y must be positive" if ($y <= 0);
    my $r;
    if ($x > 50) {
        if ($y > 50) {
            $r = $x + $y;
        } else {
            $r = $x + $y;
        }
    } else {
        if ($y > 50) {
            $r = $x + $y;
        } else {
            $r = $x + $y;
        }
    }
    return $r;
}

# --- Deeply nested if/else creating exponential paths with bounded result ---
# Each branch adjusts a score; die prunes impossible intermediate states
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $v1 >= 0 && $v1 <= 10 && $v2 >= 0 && $v2 <= 10 && $v3 >= 0 && $v3 <= 10 && $v4 >= 0 && $v4 <= 10 && $v5 >= 0 && $v5 <= 10
# post: $result >= 0 && $result <= 5
sub count_high {
    my ($v1, $v2, $v3, $v4, $v5) = @_;
    my $count = 0;
    if ($v1 > 5) {
        $count = $count + 1;
    }
    if ($v2 > 5) {
        $count = $count + 1;
    }
    if ($v3 > 5) {
        $count = $count + 1;
    }
    if ($v4 > 5) {
        $count = $count + 1;
    }
    if ($v5 > 5) {
        $count = $count + 1;
    }
    return $count;
}

# --- Ternary chain with die guard creating constrained paths ---
# sig: (I64, I64, I64) -> I64
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10 && $c >= 1 && $c <= 10
# post: $result >= 3 && $result <= 30
sub ternary_multiply {
    my ($a, $b, $c) = @_;
    die "a must be positive" if ($a <= 0);
    die "b must be positive" if ($b <= 0);
    die "c must be positive" if ($c <= 0);
    my $x = ($a > 5) ? $a : $a * 2;
    my $y = ($b > 5) ? $b : $b * 2;
    my $z = ($c > 5) ? $c : $c * 2;
    my $r = $x + $y + $z;
    return $r;
}

# --- Mixed branching: nested ifs + early return + ternary ---
# 32 paths from 5 conditions; result is always bounded
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 4 && $b >= 0 && $b <= 4 && $c >= 0 && $c <= 4 && $d >= 0 && $d <= 4 && $e >= 0 && $e <= 4
# post: $result >= 0 && $result <= 20
sub weighted_flags {
    my ($a, $b, $c, $d, $e) = @_;
    my $w = 0;
    $w = $w + (($a > 2) ? 4 : $a);
    $w = $w + (($b > 2) ? 4 : $b);
    $w = $w + (($c > 2) ? 4 : $c);
    $w = $w + (($d > 2) ? 4 : $d);
    $w = $w + (($e > 2) ? 4 : $e);
    return $w;
}
