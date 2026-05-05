# =============================================================
# Domain: Numeric Builtins (abs, min, max)
# =============================================================
# Real Perl code commonly uses abs(), and POSIX::fmin/fmax or
# custom min/max. Having these as verified builtins would let
# users write cleaner contracts.
#
# BOUNDARY PUSH: abs() as a builtin function (not user-defined),
# analogous to how length()/substr()/index()/scalar() are
# already builtin. abs($x) returns the absolute value of I64.
# =============================================================

# --- Use abs() builtin to get absolute value ---
# sig: (I64) -> I64
# pre: $x >= -1000 && $x <= 1000
# post: $result >= 0 && $result <= 1000
sub absolute {
    my ($x) = @_;
    my $r = abs($x);
    return $r;
}

# --- Distance between two integers using abs ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result >= 0
sub distance {
    my ($x, $y) = @_;
    my $d = abs($x - $y);
    return $d;
}

# --- Absolute difference is symmetric ---
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $y >= 0
# post: $result == 0
sub abs_symmetry_check {
    my ($x, $y) = @_;
    my $d1 = abs($x - $y);
    my $d2 = abs($y - $x);
    return $d1 - $d2;
}
