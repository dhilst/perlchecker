# =============================================================
# Audit: C-style for loop desugaring soundness
# =============================================================
# BUG FOUND: The loop desugaring used fixed flag names (__broke,
# __skipped) that collided with user variables of the same name.
# The desugaring would clobber user's $__broke or $__skipped,
# causing Z3-vs-Perl divergence.
#
# FIX: Use unique counter-suffixed names (__broke_N, __skipped_N)
# to prevent collisions with user-defined variables.
#
# This file tests:
# 1. Flag name collision is resolved (the core bug)
# 2. last skips the step (basic correctness)
# 3. next executes the step (basic correctness)
# 4. Combined last+next with step interaction

# --- KEY TEST: user variable named $__broke must not be clobbered ---
# Previously UNSOUND: desugaring declared its own $__broke = 0,
# clobbering the user's $__broke = 42.
# sig: (I64) -> I64
# pre: $x >= 5 && $x <= 10
# post: $result == 42
sub flag_name_no_clobber {
    my ($x) = @_;
    my $__broke = 42;
    my $i;
    for ($i = 0; $i < $x; $i = $i + 1) {
        if ($i == 3) {
            last;
        }
    }
    return $__broke;
}

# --- KEY TEST: user variable named $__skipped must not be clobbered ---
# sig: (I64) -> I64
# pre: $x == 5
# post: $result == 42
sub skip_flag_no_clobber {
    my ($x) = @_;
    my $__skipped = 42;
    my $i;
    for ($i = 0; $i < $x; $i = $i + 1) {
        if ($i == 2) {
            next;
        }
    }
    return $__skipped;
}

# --- last skips step: $i stays at 3 ---
# sig: (I64) -> I64
# pre: $x >= 5 && $x <= 10
# post: $result == 3
sub last_skips_step {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i < $x; $i = $i + 1) {
        if ($i == 3) {
            last;
        }
    }
    return $i;
}

# --- next executes step: sum skips $i==2 but step still increments ---
# sig: (I64) -> I64
# pre: $n == 5
# post: $result == 8
sub next_runs_step {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i = $i + 1) {
        if ($i == 2) {
            next;
        }
        $sum = $sum + $i;
    }
    # sum = 0+1+3+4 = 8 when n=5
    return $sum;
}

# --- combined last+next: step interaction ---
# next at even, last at 5: step runs after next but not after last
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 1
# post: $result == 5
sub last_next_step_interaction {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i < 8; $i = $i + 1) {
        if ($i % 2 == 0) {
            next;
        }
        if ($i == 5) {
            last;
        }
    }
    return $i;
}
