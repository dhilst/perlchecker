# =============================================================
# Domain: last statement for loop early exit (break)
# =============================================================
# Perl's `last` statement exits the innermost enclosing loop
# immediately. In unrolled loops, this is modeled with a break
# flag that prevents subsequent iterations from executing.
#
# BOUNDARY PUSH: `last;` inside while/for loops, desugared via
# a break-flag approach at parse time.
# =============================================================

# --- last exits the loop, result is value at break point ---
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 5
# post: $result == 3
sub find_three {
    my ($x) = @_;
    my $i = 0;
    my $r = 0;
    while ($i < 5) {
        $i = $i + 1;
        if ($i == 3) {
            $r = $i;
            last;
        }
    }
    return $r;
}

# --- last inside conditional prevents further iterations ---
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 5
# post: $result >= 1
sub sum_until_break {
    my ($x) = @_;
    my $sum = 0;
    my $i = 0;
    while ($i < $x) {
        $i = $i + 1;
        $sum = $sum + $i;
        if ($sum >= 5) {
            last;
        }
    }
    return $sum;
}

# --- last with for loop ---
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= 0
sub for_with_last {
    my ($x) = @_;
    my $r = 0;
    my $i;
    for ($i = 0; $i < $x; $i = $i + 1) {
        if ($i == 3) {
            last;
        }
        $r = $r + 1;
    }
    return $r;
}
