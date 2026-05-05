# Round 165: Audit compound assignment operators.
#
# Soundness bug found: **= with negative exponent produces a float in
# Perl (e.g. 2**-1 = 0.5) but perlchecker modeled it as truncated-
# integer 0. This let false postconditions verify. Fixed by adding a
# safety constraint (exp >= 0) that discards float-producing paths.

# --- .= (concat-assign) soundness check ---
# sig: (Str) -> Str
# pre: $s eq "hello"
# post: $result eq "helloworld"
sub concat_assign_check {
    my ($s) = @_;
    $s .= "world";
    return $s;
}

# --- <<= with negative RHS (direction reversal) ---
# sig: (I64) -> I64
# pre: $x >= 0 && $x < 256
# post: $result == int($x / 4)
sub shl_neg_rhs {
    my ($x) = @_;
    $x <<= -2;
    return $x;
}

# --- >>= with negative RHS (direction reversal) ---
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result == $x * 4
sub shr_neg_rhs {
    my ($x) = @_;
    $x >>= -2;
    return $x;
}

# --- %= with negative divisor (floor-mod semantics) ---
# sig: (I64) -> I64
# pre: $x > 0 && $x < 100
# post: $result <= 0 && $result > -3
sub mod_neg_divisor {
    my ($x) = @_;
    $x %= -3;
    return $x;
}

# --- **= with positive exponent (still works) ---
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 5
# post: $result == $x * $x
sub square_assign {
    my ($x) = @_;
    $x **= 2;
    return $x;
}

# --- compound += basic check ---
# sig: (I64, I64) -> I64
# pre: $a >= 0 && $b >= 0
# post: $result == $a + $b
sub add_assign_check {
    my ($a, $b) = @_;
    $a += $b;
    return $a;
}
