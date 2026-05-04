# =============================================================
# Round 166: Fix ternary ITE safety-constraint unsoundness
# =============================================================
# The safety constraints from a dead ternary branch must not block
# the live path. Before this fix, `($x != 0) ? int(10/$x) : 0`
# would unconditionally assert $x != 0, hiding the else-path.

# --- Ternary guards division safely ---
# When $x == 0, Perl takes the else branch (returns 0).
# The then-branch division is only reachable when $x != 0.
# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result >= -10 && $result <= 10
sub guarded_div {
    my ($x) = @_;
    my $r = ($x != 0) ? int(10 / $x) : 0;
    return $r;
}

# --- Nested ternary with guarded division ---
# sig: (Int) -> Int
# pre: $x >= -10 && $x <= 10
# post: $result >= 0 && $result <= 100
sub nested_guarded {
    my ($x) = @_;
    my $r = ($x > 0) ? (($x > 5) ? int(100 / $x) : ($x * $x)) : 0;
    return $r;
}

# --- Ternary as function argument ---
# sig: (Int, Int) -> Int
# pre: $a >= 1 && $a <= 10 && $b >= 1 && $b <= 10
# post: $result >= 1 && $result <= 10
sub ternary_as_arg {
    my ($a, $b) = @_;
    my $r = ($a > $b) ? $a : $b;
    return $r;
}

# --- Bool-result ternary ---
# When both branches yield booleans, tests the AND/OR encoding.
# sig: (Int, Int) -> Int
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= -10 && $result <= 20
sub bool_ternary_in_cond {
    my ($x, $y) = @_;
    my $r = (($x > 5) ? ($y > 3) : ($y < 7)) ? ($x + $y) : ($x - $y);
    return $r;
}
