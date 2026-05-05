# =============================================================
# Round 99: All operators path coverage stress
# =============================================================
# Comprehensive functions exercising every arithmetic, comparison,
# logical, bitwise, and string operator in branching contexts,
# proving the verifier handles the full operator vocabulary.

# --- Function 1: All arithmetic operators (+, -, *, /, %, **) ---
# Uses every arithmetic operator in branching logic. Preconditions
# ensure division is safe and values remain small.
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 4 && $y >= 1 && $y <= 4
# post: $result >= 1 && $result <= 280
sub arith_all_ops {
    my ($x, $y) = @_;
    my $sum = $x + $y;
    my $diff = $x - $y + 10;
    my $prod = $x * $y;
    my $quot = ($x + $y) / $y;
    my $rem = $x % $y;
    my $pw = $x ** 2;
    my $r = 0;
    if ($sum > 5) {
        $r = $prod + $pw;
    } elsif ($diff > 10) {
        $r = $quot + $rem + 1;
    } else {
        $r = $sum + $diff + $prod;
    }
    return $r;
}

# --- Function 2: All comparison operators (==, !=, <, >, <=, >=, <=>) ---
# Multi-way classification using every numeric comparison operator.
# Each branch uses a different comparison to select a return value.
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 0 && $result <= 30
sub compare_all_ops {
    my ($a, $b) = @_;
    my $cmp = $a <=> $b;
    if ($a == $b) {
        return 10;
    }
    if ($a != $b && $a < $b) {
        if ($a <= 3) {
            return $a + $b;
        }
        return $b;
    }
    if ($a > $b && $a >= 5) {
        return $a;
    }
    if ($cmp > 0) {
        return $a + 1;
    }
    return 0;
}

# --- Function 3: All bitwise operators (&, |, ^, ~, <<, >>) ---
# Exercises every bitwise operator with conditional logic.
# Works on small bounded non-negative integers for tractability.
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 7 && $y >= 0 && $y <= 7
# post: $result >= 0
sub bitwise_all_ops {
    my ($x, $y) = @_;
    my $a = $x & $y;
    my $o = $x | $y;
    my $xr = $x ^ $y;
    my $inv = (~$x) & 7;
    my $shl = $x << 1;
    my $shr = $y >> 1;
    my $r = 0;
    if ($a > 0) {
        $r = $shl + $shr;
    } elsif ($xr > 0) {
        $r = $o + $inv;
    } else {
        $r = $shl;
    }
    return $r;
}

# --- Function 4: All logical operators (&&, ||, !, and, or, not) ---
# Complex conditions mixing all logical operators in branching.
# Tests the verifier's ability to reason about logical combinations.
# sig: (I64, I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10 && $z >= 0 && $z <= 10
# post: $result >= 0 && $result <= 30
sub logical_all_ops {
    my ($x, $y, $z) = @_;
    my $r = 0;
    if ($x > 5 && $y > 5) {
        $r = $x + $y;
    } elsif ($x > 3 || $y > 3) {
        if (!($z == 0)) {
            $r = $z + 1;
        } else {
            $r = 1;
        }
    } elsif (not ($x == 0) and $z > 0) {
        $r = $x + $z;
    } elsif ($x == 0 or $y == 0) {
        $r = $z;
    } else {
        $r = 0;
    }
    return $r;
}
