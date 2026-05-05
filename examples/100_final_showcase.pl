# =============================================================
# Round 100: Final Consolidation — Comprehensive Showcase
# =============================================================
# This file demonstrates the full power of perlchecker across all
# 100 rounds of development. Each function exercises a major
# feature category with complex path interactions and provable
# postconditions verified by the SMT solver.

# --- Function 1: The Gauntlet ---
# Uses 15+ features: for-loop, while, ternary, if/elsif/else,
# unless, die, last, next, +=, min, max, abs, array access, **,
# comparison operators, compound assignment.
# sig: (Array<I64>, I64) -> I64
# pre: $n >= 1 && $n <= 3
# post: $result >= 1 && $result <= 50
sub the_gauntlet {
    my ($data, $n) = @_;
    die "bad" unless ($n >= 1);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i == 0 && $n == 1);
        my $val = abs($data[$i]);
        my $clamped = min($val, 8);
        my $boosted = max($clamped, 1);
        $acc += $boosted;
        last if ($acc > 30);
        my $sq = $boosted ** 2;
        my $bonus = ($sq > 16) ? 3 : 1;
        $acc += $bonus;
        if ($i == 0) {
            $acc += 2;
        } elsif ($i == 1) {
            $acc += 1;
        } else {
            $acc += 0;
        }
        unless ($clamped > 5) {
            $acc += 0;
        }
    }
    my $final = max($acc, 1);
    my $bounded = min($final, 50);
    return $bounded;
}

# --- Function 2: Path Explorer ---
# 5 sequential if/else blocks = 32 paths. Each block adds a
# bounded amount to the accumulator based on input parameters.
# The postcondition bounds the total across all 32 paths.
# sig: (I64, I64, I64, I64, I64) -> I64
# pre: $a >= 0 && $a <= 3 && $b >= 0 && $b <= 3 && $c >= 0 && $c <= 3 && $d >= 0 && $d <= 3 && $e >= 0 && $e <= 3
# post: $result >= 5 && $result <= 20
sub path_explorer {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 1) {
        $r += $a;
    } else {
        $r += 1;
    }
    if ($b > 1) {
        $r += $b;
    } else {
        $r += 1;
    }
    if ($c > 1) {
        $r += $c;
    } else {
        $r += 1;
    }
    if ($d > 1) {
        $r += $d;
    } else {
        $r += 1;
    }
    if ($e > 1) {
        $r += $e;
    } else {
        $r += 1;
    }
    return $r;
}

# --- Function 3: String Wizard ---
# Chains 6+ string operations: contains, starts_with, ends_with,
# length, substr, concat with branching on each result.
# sig: (Str) -> I64
# pre: length($s) >= 6 && length($s) <= 12
# post: $result >= 1 && $result <= 18
sub string_wizard {
    my ($s) = @_;
    my $len = length($s);
    my $has_ab = contains($s, "ab");
    my $sw_h = starts_with($s, "h");
    my $ew_d = ends_with($s, "d");
    my $prefix = substr($s, 0, 3);
    my $plen = length($prefix);
    my $score = $plen;
    if ($has_ab == 1) {
        $score += 2;
    } else {
        $score += 1;
    }
    if ($sw_h == 1) {
        $score += 3;
    } elsif ($ew_d == 1) {
        $score += 2;
    } else {
        $score += 1;
    }
    if ($len > 9) {
        $score += 3;
    } elsif ($len > 7) {
        $score += 2;
    } else {
        $score += 1;
    }
    my $combo = $has_ab + $sw_h + $ew_d;
    if ($combo >= 2) {
        $score += 3;
    } else {
        $score += 1;
    }
    return $score;
}

# --- Function 4: Bit Manipulator ---
# Exercises all bitwise ops (&, |, ^, ~, <<, >>) with conditional
# logic. Works on bounded non-negative integers for tractability.
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 7 && $y >= 1 && $y <= 7
# post: $result >= 0 && $result <= 30
sub bit_manipulator {
    my ($x, $y) = @_;
    my $a = $x & $y;
    my $o = $x | $y;
    my $xr = $x ^ $y;
    my $inv = (~$x) & 7;
    my $shl = $x << 1;
    my $shr = $y >> 1;
    my $r = 0;
    if ($a > 3) {
        $r = $shl + $shr;
    } elsif ($xr > 4) {
        $r = $o + $inv;
    } elsif ($o > 5) {
        $r = $shl + $a;
    } else {
        $r = $xr + $shr + 1;
    }
    return $r;
}

# --- Function 5: State Machine ---
# 3-state FSM with 2 transition steps. State 0->1->2 with input-
# dependent transitions. Proves final state is bounded.
# sig: (I64, I64) -> I64
# pre: $input1 >= 0 && $input1 <= 3 && $input2 >= 0 && $input2 <= 3
# post: $result >= 0 && $result <= 2
sub state_machine {
    my ($input1, $input2) = @_;
    my $state = 0;
    if ($input1 > 1) {
        $state = 1;
    } else {
        $state = 0;
    }
    if ($state == 0) {
        if ($input2 > 2) {
            $state = 1;
        } else {
            $state = 0;
        }
    } elsif ($state == 1) {
        if ($input2 > 1) {
            $state = 2;
        } else {
            $state = 1;
        }
    }
    return $state;
}

# --- Function 6: Array Tracker ---
# push/pop + loop + scalar + conditional. Tracks array length
# through operations and proves bounds on the result.
# sig: (Array<I64>, I64, I64, I64, I64) -> I64
# pre: scalar(@arr) == $n && $n == 0 && $a >= 1 && $a <= 5 && $b >= 1 && $b <= 5 && $c >= 1 && $c <= 5
# post: $result >= 3 && $result <= 15
sub array_tracker {
    my ($arr, $n, $a, $b, $c) = @_;
    push(@arr, $a);
    push(@arr, $b);
    push(@arr, $c);
    my $len = scalar(@arr);
    my $top = pop(@arr);
    my $new_len = scalar(@arr);
    my $r;
    if ($top > 3) {
        $r = $top + $len + $new_len;
    } elsif ($top > 1) {
        $r = $top + $new_len;
    } else {
        $r = $len;
    }
    return $r;
}

# --- Function 7: Hash Oracle ---
# Conditional hash writes + reads + branching on values.
# Stores into multiple keys conditionally and reads back to decide
# which value to return. SMT must track store/select chains.
# sig: (Hash<Str, I64>, I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 1 && $result <= 20
sub hash_oracle {
    my ($h, $x, $y) = @_;
    if ($x > 5) {
        $h{"alpha"} = $x + $y;
    } else {
        $h{"alpha"} = $x + 1;
    }
    if ($y > 5) {
        $h{"beta"} = $x + $y;
    } else {
        $h{"beta"} = $y + 1;
    }
    my $va = $h{"alpha"};
    my $vb = $h{"beta"};
    if ($va > $vb) {
        return $va;
    }
    return $vb;
}

# --- Function 8: Guard Fortress ---
# Multiple return-if guards with die-unless for validation.
# Early return-if exits prune paths before the main computation.
# The precondition ensures die guards are unreachable (proving safety).
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 2 && $result <= 20
sub guard_fortress {
    my ($x, $y) = @_;
    die "x negative" unless ($x >= 0);
    die "y negative" unless ($y >= 0);
    return 2 if ($x == 0);
    return 3 if ($y == 0);
    die "too big" unless ($x <= 10);
    die "too big" unless ($y <= 10);
    return $x + $y;
}

# --- Function 9: Swap Dance ---
# Conditional list swaps proving ordering. After the swap pass,
# the smaller value is always first. Returns the minimum.
# sig: (I64, I64, I64) -> I64
# pre: $a >= 1 && $a <= 8 && $b >= 1 && $b <= 8 && $c >= 1 && $c <= 8
# post: $result >= 1 && $result <= 8
sub swap_dance {
    my ($a, $b, $c) = @_;
    my ($x, $y, $z) = ($a, $b, $c);
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    if ($y > $z) {
        ($y, $z) = ($z, $y);
    }
    if ($x > $y) {
        ($x, $y) = ($y, $x);
    }
    return $x;
}

# --- Function 10: Boundary Sentinel ---
# Boundary value checks with 3-way branching. Tests exact boundary
# conditions (0, thresholds) creating precise path splits.
# sig: (I64, I64) -> I64
# pre: $x >= -5 && $x <= 5 && $y >= -5 && $y <= 5
# post: $result >= 0 && $result <= 30
sub boundary_sentinel {
    my ($x, $y) = @_;
    my $r = 0;
    if ($x == 0) {
        $r += 10;
    } elsif ($x > 0) {
        $r += $x;
    } else {
        $r += 0 - $x;
    }
    if ($y == 0) {
        $r += 10;
    } elsif ($y > 0) {
        $r += $y;
    } else {
        $r += 0 - $y;
    }
    if ($r > 15) {
        $r += 5;
    }
    return $r;
}
