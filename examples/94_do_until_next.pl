# =============================================================
# Round 94: Do-until + next path stress
# =============================================================
# Functions using do-until loops (body executes at least once, loops
# while condition is false) with next statements that skip the
# remainder of the body. The verifier must track the skip flag
# through the unconditional first iteration plus conditional
# subsequent iterations, expanding multiple paths per iteration.

# --- Function 1: Do-until with next skipping even iterations ---
# Accumulates only on odd-counter iterations. The body always runs
# at least once (i=0 is even, so next fires on first iteration).
# With n in [2,4], loop runs n times. Only odd i values contribute.
# i=0: skip. i=1: acc+=2. i=2: skip. i=3: acc+=2.
# n=2: acc=2. n=3: acc=2. n=4: acc=4.
# sig: (I64) -> I64
# pre: $n >= 2 && $n <= 4
# post: $result >= 2 && $result <= 4
sub do_until_skip_even {
    my ($n) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        $i++;
        next if ($i % 2 == 0);
        $acc += 2;
    } until ($i >= $n);
    return $acc;
}

# --- Function 2: Conditional accumulation with next in do-until ---
# Sums values where the index exceeds a threshold, skipping others.
# Loop runs exactly 4 iterations (i goes 0..3). next skips when
# i < thresh. For thresh in [1,3]:
#   thresh=1: sums i=1,2,3 -> 6. thresh=2: sums i=2,3 -> 5.
#   thresh=3: sums i=3 -> 3.
# sig: (I64) -> I64
# pre: $thresh >= 1 && $thresh <= 3
# post: $result >= 3 && $result <= 6
sub do_until_conditional_accum {
    my ($thresh) = @_;
    my $sum = 0;
    my $i = 0;
    do {
        if ($i < $thresh) {
            $i++;
            next;
        }
        $sum += $i;
        $i++;
    } until ($i >= 4);
    return $sum;
}

# --- Function 3: Do-until with next if modifier and counter ---
# Counts only values that pass a modulo filter. Iterates 4 times.
# next if (val % 3 == 0) skips multiples of 3.
# With base in [0,2]: val = base + i for i in 0..3.
#   base=0: vals 0,1,2,3 -> skip 0,3 -> count 2.
#   base=1: vals 1,2,3,4 -> skip 3 -> count 3.
#   base=2: vals 2,3,4,5 -> skip 3 -> count 3.
# sig: (I64) -> I64
# pre: $base >= 0 && $base <= 2
# post: $result >= 2 && $result <= 3
sub do_until_next_if_count {
    my ($base) = @_;
    my $count = 0;
    my $i = 0;
    do {
        my $val = $base + $i;
        $i++;
        next if ($val % 3 == 0);
        $count += 1;
    } until ($i >= 4);
    return $count;
}

# --- Function 4: Track counter with next skipping some increments ---
# Builds a weighted sum where next skips adding the weight on
# certain iterations. Iterates 3 times (i=0,1,2).
# When i==1 and step is even, skip. Otherwise add step.
# step in [1,3]:
#   step=1 (odd): no skip, acc = 1+1+1 = 3.
#   step=2 (even): skip i=1, acc = 2+0+2 = 4.
#   step=3 (odd): no skip, acc = 3+3+3 = 9.
# sig: (I64) -> I64
# pre: $step >= 1 && $step <= 3
# post: $result >= 3 && $result <= 9
sub do_until_weighted_skip {
    my ($step) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        if ($i == 1 && $step % 2 == 0) {
            $i++;
            next;
        }
        $acc += $step;
        $i++;
    } until ($i >= 3);
    return $acc;
}

# --- Function 5: Nested condition with next in do-until ---
# Classifies each iteration and either skips or accumulates
# different amounts. Runs 4 iterations.
# For x in [1,3], each iteration i (0..3):
#   if i+x > 4: next (skip). elif i+x > 2: acc += 2. else: acc += 1.
# x=1: i=0:1+1=2<=2 +1, i=1:1+1=2<=2 +1, i=2:1+2=3>2 +2, i=3:1+3=4<=4 +2 => 6
# x=2: i=0:2+0=2<=2 +1, i=1:2+1=3>2 +2, i=2:2+2=4<=4 +2, i=3:2+3=5>4 skip => 5
# x=3: i=0:3+0=3>2 +2, i=1:3+1=4<=4 +2, i=2:3+2=5>4 skip, i=3:3+3=6>4 skip => 4
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 3
# post: $result >= 4 && $result <= 6
sub do_until_nested_next {
    my ($x) = @_;
    my $acc = 0;
    my $i = 0;
    do {
        my $val = $x + $i;
        if ($val > 4) {
            $i++;
            next;
        } elsif ($val > 2) {
            $acc += 2;
        } else {
            $acc += 1;
        }
        $i++;
    } until ($i >= 4);
    return $acc;
}
