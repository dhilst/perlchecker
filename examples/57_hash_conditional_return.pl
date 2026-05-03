# =============================================================
# Round 57: Hash + conditional + early return path stress
# =============================================================
# Exercises path expansion where the verifier must track hash state
# (store/load via Z3 array theory) through multiple conditional
# branches and early returns. Each function creates paths where
# hash contents differ depending on which branch was taken, and
# postconditions require reasoning about hash state across paths.

# --- Conditional hash population with early return on sentinel ---
# Stores different values based on input range, then returns early
# if a sentinel key holds zero. 4 distinct exit paths.
# sig: (Hash<Str, Int>, Int) -> Int
# pre: $x >= 0 && $x <= 100
# post: $result >= 1 && $result <= 100
sub hash_store_guard {
    my ($h, $x) = @_;
    if ($x <= 25) {
        $h{"level"} = 1;
    } elsif ($x <= 50) {
        $h{"level"} = 2;
    } elsif ($x <= 75) {
        $h{"level"} = 3;
    } else {
        $h{"level"} = 4;
    }
    $h{"score"} = $x;
    return $h{"level"} if ($h{"score"} == 0);
    return $h{"score"};
}

# --- Multi-key hash with cross-key reasoning ---
# Stores into two keys based on conditions, then reads both back
# and combines them. The verifier must track two array stores and
# prove the sum is bounded across all 4 path combinations.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $a >= 0 && $a <= 10 && $b >= 0 && $b <= 10
# post: $result >= 0 && $result <= 20
sub hash_dual_store {
    my ($h, $a, $b) = @_;
    if ($a > 5) {
        $h{"left"} = 10;
    } else {
        $h{"left"} = $a;
    }
    if ($b > 5) {
        $h{"right"} = 10;
    } else {
        $h{"right"} = $b;
    }
    my $total = $h{"left"} + $h{"right"};
    return $total;
}

# --- Hash lookup cascade with early returns ---
# Stores a classification, then uses cascading return-if statements
# based on the stored hash value. Creates 5 exit paths and requires
# the verifier to track the hash store through each guard.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $x >= 0 && $x <= 100 && $bonus >= 0 && $bonus <= 10
# post: $result >= 0 && $result <= 50
sub hash_cascade_return {
    my ($h, $x, $bonus) = @_;
    if ($x < 20) {
        $h{"tier"} = 1;
    } elsif ($x < 40) {
        $h{"tier"} = 2;
    } elsif ($x < 60) {
        $h{"tier"} = 3;
    } elsif ($x < 80) {
        $h{"tier"} = 4;
    } else {
        $h{"tier"} = 5;
    }
    return 0 if ($h{"tier"} == 1);
    return 10 + $bonus if ($h{"tier"} == 2);
    return 20 + $bonus if ($h{"tier"} == 3);
    return 30 + $bonus if ($h{"tier"} == 4);
    return 40 + $bonus;
}

# --- Hash state through loop iterations with conditional stores ---
# Iterates a bounded loop, conditionally storing into the hash each
# iteration based on the loop counter. After the loop, reads the
# final hash state. Combines loop unrolling with hash tracking.
# sig: (Hash<Str, Int>, Int) -> Int
# pre: $base >= 1 && $base <= 5
# post: $result >= 1 && $result <= 20
sub hash_loop_accum {
    my ($h, $base) = @_;
    $h{"acc"} = $base;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        if ($i == 0) {
            $h{"acc"} = $h{"acc"} + 1;
        } elsif ($i == 1) {
            $h{"acc"} = $h{"acc"} + 2;
        } else {
            $h{"acc"} = $h{"acc"} + $base;
        }
    }
    return $h{"acc"};
}

# --- Hash with ternary stores and multi-exit path explosion ---
# Combines ternary expressions for hash value computation with
# multiple exit paths based on hash reads. Creates path explosion
# from 2 ternary stores * 3 exit paths = 12 paths total.
# sig: (Hash<Str, Int>, Int, Int) -> Int
# pre: $x >= 0 && $x <= 20 && $y >= 0 && $y <= 20
# post: $result >= 0 && $result <= 40
sub hash_ternary_exits {
    my ($h, $x, $y) = @_;
    $h{"a"} = ($x > 10) ? 20 : $x;
    $h{"b"} = ($y > 10) ? 20 : $y;
    my $sum = $h{"a"} + $h{"b"};
    return 0 if ($sum == 0);
    return $sum if ($sum <= 20);
    return 40 unless ($sum < 40);
    return $sum;
}
