# =============================================================
# Round 97: Full feature integration stress test
# =============================================================
# Functions that use as many distinct features as possible in a
# single body, testing that the full pipeline (PEG -> AST ->
# type checker -> SSA/IR -> CFG -> symexec -> SMT) handles
# complex feature interactions without interference.

# --- Function 1: Kitchen sink numeric ---
# Uses: for-loop, ternary, if/elsif/else, unless, die guard,
# last, compound assign (+=), min(), max(), abs(), array access
# sig: (Array<I64>, I64) -> I64
# pre: $len >= 1 && $len <= 3
# post: $result >= 0 && $result <= 30
sub kitchen_sink_numeric {
    my ($arr, $len) = @_;
    die "bad" unless ($len >= 1);
    my $acc = 0;
    my $i;
    for ($i = 0; $i < $len; $i++) {
        my $val = abs($arr[$i]);
        my $clamped = min($val, 5);
        $acc += $clamped;
        last if ($acc > 20);
        my $bonus = ($i > 1) ? 1 : 0;
        $acc += $bonus;
        if ($i == 0) {
            $acc += 1;
        } elsif ($i == 1) {
            $acc += 0;
        } else {
            $acc += 0;
        }
        unless ($clamped > 3) {
            $acc += 0;
        }
    }
    my $final = max($acc, 0);
    return $final;
}

# --- Function 2: String feature integration ---
# Uses: contains, starts_with, length, substr, concat (.=),
# if/elsif/else, return if, ternary, unless
# sig: (Str) -> I64
# pre: length($s) >= 5 && length($s) <= 10
# post: $result >= 0 && $result <= 20
sub string_integration {
    my ($s) = @_;
    my $len = length($s);
    die "empty" unless ($len >= 1);
    my $has_ab = contains($s, "ab");
    my $starts_h = starts_with($s, "h");
    my $prefix = substr($s, 0, 3);
    my $plen = length($prefix);
    my $score = 0;
    $score += $has_ab;
    $score += $starts_h;
    $score += $plen;
    my $bonus = ($has_ab == 1) ? 2 : 0;
    $score += $bonus;
    if ($len > 8) {
        $score += 3;
    } elsif ($len > 6) {
        $score += 2;
    } else {
        $score += 1;
    }
    return $score if ($score > 15);
    unless ($starts_h == 1) {
        $score += 1;
    }
    return $score;
}

# --- Function 3: Numeric bitwise integration ---
# Uses: **, <<, >>, &, |, ternary, for-loop, next if,
# die unless, spaceship (<=>), if/else, compound assign (+=)
# sig: (I64, I64) -> I64
# pre: $x >= 1 && $x <= 3 && $n >= 1 && $n <= 3
# post: $result >= 1 && $result <= 100
sub numeric_bitwise_integration {
    my ($x, $n) = @_;
    die "bad" unless ($x >= 1);
    my $pow = $x ** 2;
    my $shifted = $x << 1;
    my $right = $shifted >> 0;
    my $masked = $pow & 15;
    my $merged = $masked | 1;
    my $cmp = $pow <=> $shifted;
    my $base = ($cmp > 0) ? $pow : $shifted;
    my $acc = $base;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i == 0 && $x == 1);
        if ($cmp >= 0) {
            $acc += $merged;
        } else {
            $acc += 1;
        }
    }
    my $final = max(min($acc, 100), 1);
    return $final;
}
