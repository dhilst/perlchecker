# =============================================================
# Round 85: Hash update + conditional read path stress
# =============================================================
# Functions that write different values to hash keys in different
# branches, then read those keys back in later branches. The
# verifier must track which hash values were stored depending on
# earlier branch decisions (Z3 array store/select across paths).

# --- Function 1: Overwrite same key in nested branches, read later ---
# Writes to "val" key in 4 different paths (nested if/else), then
# reads it back and returns. The verifier must track that the final
# value of $h{"val"} depends on which of the 4 paths was taken.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $x >= 0 && $x <= 20 && $y >= 0 && $y <= 20
# post: $result >= 1 && $result <= 40
sub hash_nested_overwrite {
    my ($h, $x, $y) = @_;
    if ($x > 10) {
        if ($y > 10) {
            $h{"val"} = $x + $y;
        } else {
            $h{"val"} = $x;
        }
    } else {
        if ($y > 10) {
            $h{"val"} = $y;
        } else {
            $h{"val"} = 1;
        }
    }
    my $r = $h{"val"};
    return $r;
}

# --- Function 2: Conditional overwrite with last-write-wins ---
# First stores a default, then conditionally overwrites with a
# second value. The verifier must reason that the second write
# supersedes the first only on certain paths.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 0 && $result <= 20
sub hash_last_write_wins {
    my ($h, $a, $b) = @_;
    $h{"x"} = $a;
    $h{"y"} = $b;
    if ($a > 5) {
        $h{"x"} = $a + $b;
    }
    if ($b > 5) {
        $h{"y"} = $a + $b;
    }
    my $rx = $h{"x"};
    my $ry = $h{"y"};
    if ($rx > $ry) {
        return $rx;
    }
    return $ry;
}

# --- Function 3: Multi-key write then cross-key comparison ---
# Stores into three hash keys conditionally, then reads all three
# and selects which to return based on their relative values.
# The verifier must track 3 parallel store/select chains.
# sig: (Hash<Str, Int>, Int, Int, Int) -> Int
# pre: $p >= 1 && $p <= 10 && $q >= 1 && $q <= 10 && $r >= 1 && $r <= 10
# post: $result >= 1 && $result <= 20
sub hash_three_key_compare {
    my ($h, $p, $q, $r) = @_;
    if ($p > 5) {
        $h{"a"} = $p + $q;
    } else {
        $h{"a"} = $p;
    }
    if ($q > 5) {
        $h{"b"} = $q + $r;
    } else {
        $h{"b"} = $q;
    }
    if ($r > 5) {
        $h{"c"} = $r + $p;
    } else {
        $h{"c"} = $r;
    }
    my $va = $h{"a"};
    my $vb = $h{"b"};
    my $vc = $h{"c"};
    if ($va >= $vb && $va >= $vc) {
        return $va;
    }
    if ($vb >= $vc) {
        return $vb;
    }
    return $vc;
}

# --- Function 4: Hash update in bounded loop with conditional ---
# Each loop iteration conditionally updates the hash key "acc"
# based on whether the current accumulator is above a threshold.
# The verifier must track the hash state across 3 unrolled iterations.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $init >= 1 && $init <= 5 && $step >= 1 && $step <= 3
# post: $result >= 4 && $result <= 14
sub hash_loop_conditional_update {
    my ($h, $init, $step) = @_;
    $h{"acc"} = $init;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        if ($h{"acc"} > 6) {
            $h{"acc"} = $h{"acc"} + 1;
        } else {
            $h{"acc"} = $h{"acc"} + $step;
        }
    }
    return $h{"acc"};
}

# --- Function 5: Sequential overwrites with branch-dependent read ---
# Writes to two keys, then based on a third condition decides which
# key to read and return. The verifier must maintain both store
# chains and select the right one at the return point.
# sig: (Hash<Str, Int>, Int, Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10 && $sel >= 0 && $sel <= 1
# post: $result >= 0 && $result <= 20
sub hash_select_by_condition {
    my ($h, $x, $y, $sel) = @_;
    if ($x > 5) {
        $h{"left"} = $x + $y;
    } else {
        $h{"left"} = $x;
    }
    if ($y > 5) {
        $h{"right"} = $x + $y;
    } else {
        $h{"right"} = $y;
    }
    if ($sel == 1) {
        return $h{"left"};
    }
    return $h{"right"};
}
