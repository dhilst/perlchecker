# =============================================================
# Round 59: Bitwise + conditional + spaceship path stress
# =============================================================
# Exercises cross-theory reasoning in the SMT solver: bitvector
# operations (AND, OR, XOR, shifts) interleaved with integer
# comparisons and spaceship operator results. Each function
# creates multiple conditional paths where the verifier must
# track bitvector semantics alongside integer arithmetic.

# --- Bitwise AND masking with conditional branches ---
# Masks input to low 4 bits, then branches on result to produce
# different offsets. The verifier must reason that (x & 0xF) is
# always in [0,15] and track through 3 branch paths.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result >= 0 && $result <= 25
sub mask_and_branch {
    my ($x) = @_;
    my $low = $x & 15;
    my $r;
    if ($low < 5) {
        $r = $low;
    } elsif ($low < 10) {
        $r = $low + 5;
    } else {
        $r = $low + 10;
    }
    return $r;
}

# --- Spaceship comparison combined with bitwise OR ---
# Uses spaceship to classify relationship, then combines result
# with bitwise OR of masked inputs. Cross-theory: bitvector for
# OR/AND, integer for spaceship ITE desugaring.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0 && $result <= 31
sub spaceship_bitor_path {
    my ($a, $b) = @_;
    my $cmp = $a <=> $b;
    my $ma = $a & 15;
    my $mb = $b & 15;
    my $combined = $ma | $mb;
    my $r;
    if ($cmp < 0) {
        $r = $combined & 15;
    } elsif ($cmp == 0) {
        $r = $combined & 7;
    } else {
        $r = ($combined & 15) + ($cmp * 16);
    }
    return $r;
}

# --- Shift accumulator with conditional paths and early exit ---
# Iterates a bounded loop, left-shifting an accumulator each
# iteration. Branches on a masked check to decide whether to add
# or XOR. Exercises bitvector shift + XOR + conditional in loop.
# sig: (Int) -> Int
# pre: $seed >= 1 && $seed <= 3
# post: $result >= 0 && $result <= 100
sub shift_accum_paths {
    my ($seed) = @_;
    my $acc = $seed;
    my $i;
    for ($i = 0; $i < 3; $i++) {
        my $check = $acc & 3;
        if ($check < 2) {
            $acc = $acc + ($seed << 1);
        } else {
            $acc = $acc + 1;
        }
        last if ($acc > 50);
    }
    return $acc;
}

# --- Bitwise XOR with spaceship and ternary on paths ---
# XORs two masked values, uses spaceship on the XOR result vs a
# threshold to pick a return path. Combines bitvector XOR with
# spaceship ITE in a multi-exit function.
# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 255 && $y >= 0 && $y <= 255
# post: $result >= 0 && $result <= 30
sub xor_spaceship_select {
    my ($x, $y) = @_;
    my $mx = $x & 15;
    my $my = $y & 15;
    my $xored = $mx ^ $my;
    my $cmp = $xored <=> 8;
    my $r;
    if ($cmp < 0) {
        $r = $xored;
    } elsif ($cmp == 0) {
        $r = 15;
    } else {
        $r = $xored + 15;
    }
    return $r;
}

# --- Multi-op pipeline: shift + AND + spaceship + abs ---
# Pipelines: right-shift input, mask, compare with spaceship,
# then uses abs to ensure non-negative. Exercises 4 operations
# from different theories in a single data flow with branches.
# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 255 && $y >= 0 && $y <= 255
# post: $result >= 0 && $result <= 15
sub pipeline_multi_theory {
    my ($x, $y) = @_;
    my $sx = ($x >> 4) & 15;
    my $sy = ($y >> 4) & 15;
    my $cmp = $sx <=> $sy;
    my $diff;
    if ($cmp > 0) {
        $diff = $sx - $sy;
    } elsif ($cmp < 0) {
        $diff = $sy - $sx;
    } else {
        $diff = 0;
    }
    return $diff;
}
