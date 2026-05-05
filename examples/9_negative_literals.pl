# Round 9: Negative integer literals — validation and demonstration
#
# Negative values work via prefix negation (op_neg applied to atoms).
# This file exercises negative literals in annotations, assignments,
# arithmetic, conditionals, and postconditions.

# --- Basic negation of a variable ---
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: $result == -$x
sub flip_sign {
    my ($x) = @_;
    my $r = -$x;
    return $r;
}

# --- Negative literal in assignment ---
# sig: (I64) -> I64
# pre: $x >= -50 && $x <= 50
# post: $result >= 0
sub distance_from_origin {
    my ($x) = @_;
    my $neg = -1;
    my $r = ($x >= 0) ? $x : $x * $neg;
    return $r;
}

# --- Custom absolute value using negation ---
# sig: (I64) -> I64
# pre: $x >= -10 && $x <= 10
# post: ($x >= 0 && $result == $x) || ($x < 0 && $result == -$x)
sub my_abs {
    my ($x) = @_;
    my $r = ($x >= 0) ? $x : -$x;
    return $r;
}

# --- Negative bounds in precondition ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= -1
# post: $result >= 1 && $result <= 100
sub negate_negative {
    my ($x) = @_;
    my $r = -1 * $x;
    return $r;
}

# --- Negative offset arithmetic ---
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 50
# post: $result == $x + -5
sub subtract_five {
    my ($x) = @_;
    my $offset = -5;
    my $r = $x + $offset;
    return $r;
}

# --- Clamping to a negative lower bound ---
# sig: (I64) -> I64
# pre: $x >= -200 && $x <= 200
# post: $result >= -100 && $result <= 200
sub clamp_lower {
    my ($x) = @_;
    my $floor = -100;
    my $r = ($x < $floor) ? $floor : $x;
    return $r;
}

# --- Double negation is identity ---
# sig: (I64) -> I64
# pre: $x >= -50 && $x <= 50
# post: $result == $x
sub double_negate {
    my ($x) = @_;
    my $tmp = -$x;
    my $r = -$tmp;
    return $r;
}

# --- Sign function returning -1, 0, or 1 ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: ($x > 0 && $result == 1) || ($x == 0 && $result == 0) || ($x < 0 && $result == -1)
sub sign {
    my ($x) = @_;
    my $r = ($x > 0) ? 1 : ($x == 0) ? 0 : -1;
    return $r;
}
