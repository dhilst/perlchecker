# =============================================================
# Round 69: Exponentiation + overflow guard path stress
# =============================================================
# Functions using ** (power) operator in conditional branches with
# bounds checking to prevent overflow, creating paths where the
# verifier must reason about exponential growth constraints.

# --- Function 1: Square with overflow guard ---
# Computes x**2, but dies if the result would exceed a threshold.
# Precondition bounds x tightly so die is unreachable.
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 10
# post: $result >= 1 && $result <= 100
sub square_bounded {
    my ($x) = @_;
    my $sq = $x ** 2;
    die "overflow" if ($sq > 100);
    return $sq;
}

# --- Function 2: Cube with conditional clamping ---
# Computes x**3 and clamps to a ceiling if too large.
# With x in [1,4], x**3 is in [1,64]. Clamp at 50 means
# result is always in [1,50].
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 4
# post: $result >= 1 && $result <= 50
sub cube_clamped {
    my ($x) = @_;
    my $c = $x ** 3;
    my $r;
    if ($c > 50) {
        $r = 50;
    } else {
        $r = $c;
    }
    return $r;
}

# --- Function 3: Power-based branching ---
# Uses x**2 vs 2**x to decide which path to take, then returns
# a bounded result. For x in [1,3]: x**2 in [1,9], 2**x in [2,8].
# When x**2 >= 2**x, returns x**2 - 2**x + 1 (non-negative).
# When x**2 < 2**x, returns 2**x - x**2 + 1 (positive).
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 3
# post: $result >= 1
sub power_compare_branch {
    my ($x) = @_;
    my $sq = $x ** 2;
    my $exp = 2 ** $x;
    my $r;
    if ($sq >= $exp) {
        $r = $sq - $exp + 1;
    } else {
        $r = $exp - $sq + 1;
    }
    return $r;
}

# --- Function 4: Nested exponent guards with multiple paths ---
# Checks x**2 and y**2 independently, creating 4 paths based on
# whether each exceeds a threshold. Returns a category 1-4.
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 5 && $y >= 0 && $y <= 5
# post: $result >= 1 && $result <= 4
sub dual_square_classify {
    my ($x, $y) = @_;
    my $xsq = $x ** 2;
    my $ysq = $y ** 2;
    my $r;
    if ($xsq > 9) {
        if ($ysq > 9) {
            $r = 4;
        } else {
            $r = 3;
        }
    } else {
        if ($ysq > 9) {
            $r = 2;
        } else {
            $r = 1;
        }
    }
    return $r;
}

# --- Function 5: Exponent accumulation with early exit ---
# Iterates and accumulates x**2 at each step. Exits early if
# accumulator exceeds threshold. With x in [1,2] and n in [1,3],
# accumulates at most 3*4=12. Returns result clamped to [0,10].
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 2 && $n >= 1 && $n <= 3
# post: $result >= 1 && $result <= 10
sub power_accumulate_exit {
    my ($x, $n) = @_;
    my $acc = 0;
    my $sq = $x ** 2;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        $acc += $sq;
        last if ($acc >= 10);
    }
    my $r;
    if ($acc > 10) {
        $r = 10;
    } else {
        $r = $acc;
    }
    return $r;
}
