# =============================================================
# Round 77: Multi-variable state machine path stress
# =============================================================
# Functions that simulate simple state machines using integer state
# variables, with transitions driven by conditionals. Each function
# creates paths for each possible state sequence, stressing the
# symbolic execution engine's path enumeration.

# --- Function 1: Two-step state machine with 3 states ---
# State starts at 0. In step 1, transitions to 1 or 2 based on input a.
# In step 2, transitions based on current state + input b.
# Final state is always in [0, 2].
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1
# post: $result >= 0 && $result <= 2
sub two_step_fsm {
    my ($a, $b) = @_;
    my $state = 0;
    if ($a == 1) {
        $state = 1;
    } else {
        $state = 2;
    }
    if ($state == 1) {
        if ($b == 1) {
            $state = 2;
        } else {
            $state = 0;
        }
    } elsif ($state == 2) {
        if ($b == 1) {
            $state = 0;
        } else {
            $state = 1;
        }
    }
    return $state;
}

# --- Function 2: Three-step state machine with output accumulator ---
# State transitions through 3 steps, accumulating an output value
# based on which state we're in at each step. The output is bounded
# by the number of steps times max per-step contribution.
# sig: (Int, Int, Int) -> Int
# pre: $x >= 0 && $x <= 1 && $y >= 0 && $y <= 1 && $z >= 0 && $z <= 1
# post: $result >= 3 && $result <= 9
sub fsm_accumulator {
    my ($x, $y, $z) = @_;
    my $state = 0;
    my $out = 0;
    if ($x == 1) {
        $state = 1;
    } else {
        $state = 2;
    }
    if ($state == 0) {
        $out = $out + 1;
    } elsif ($state == 1) {
        $out = $out + 2;
    } else {
        $out = $out + 3;
    }
    if ($y == 1) {
        if ($state == 1) {
            $state = 2;
        } else {
            $state = 0;
        }
    } else {
        if ($state == 2) {
            $state = 1;
        } else {
            $state = 2;
        }
    }
    if ($state == 0) {
        $out = $out + 1;
    } elsif ($state == 1) {
        $out = $out + 2;
    } else {
        $out = $out + 3;
    }
    if ($z == 1) {
        if ($state == 0) {
            $state = 1;
        } elsif ($state == 1) {
            $state = 0;
        } else {
            $state = 2;
        }
    } else {
        if ($state == 0) {
            $state = 2;
        } elsif ($state == 1) {
            $state = 1;
        } else {
            $state = 0;
        }
    }
    if ($state == 0) {
        $out = $out + 1;
    } elsif ($state == 1) {
        $out = $out + 2;
    } else {
        $out = $out + 3;
    }
    return $out;
}

# --- Function 3: State machine with elsif dispatch ---
# Simulates a 4-state machine (states 0-3) for 2 steps.
# Uses elsif chains for state dispatch. Transitions depend on
# the input at each step. Proves final state is in [0, 3].
# sig: (Int, Int) -> Int
# pre: $inp1 >= 0 && $inp1 <= 1 && $inp2 >= 0 && $inp2 <= 1
# post: $result >= 0 && $result <= 3
sub four_state_dispatch {
    my ($inp1, $inp2) = @_;
    my $state = 0;
    if ($state == 0) {
        if ($inp1 == 1) {
            $state = 1;
        } else {
            $state = 3;
        }
    } elsif ($state == 1) {
        if ($inp1 == 1) {
            $state = 2;
        } else {
            $state = 0;
        }
    } elsif ($state == 2) {
        if ($inp1 == 1) {
            $state = 3;
        } else {
            $state = 1;
        }
    } else {
        if ($inp1 == 1) {
            $state = 0;
        } else {
            $state = 2;
        }
    }
    if ($state == 0) {
        if ($inp2 == 1) {
            $state = 1;
        } else {
            $state = 3;
        }
    } elsif ($state == 1) {
        if ($inp2 == 1) {
            $state = 2;
        } else {
            $state = 0;
        }
    } elsif ($state == 2) {
        if ($inp2 == 1) {
            $state = 3;
        } else {
            $state = 1;
        }
    } else {
        if ($inp2 == 1) {
            $state = 0;
        } else {
            $state = 2;
        }
    }
    return $state;
}

# --- Function 4: Dual state variables with interaction ---
# Two state variables evolve together: s1 and s2 each take values
# 0 or 1. Transitions of one depend on the other, creating
# cross-variable path dependencies. After 2 steps, return s1 + s2.
# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 1 && $b >= 0 && $b <= 1
# post: $result >= 0 && $result <= 2
sub dual_state_interaction {
    my ($a, $b) = @_;
    my $s1 = 0;
    my $s2 = 1;
    if ($a == 1) {
        if ($s2 == 1) {
            $s1 = 1;
        } else {
            $s1 = 0;
        }
        if ($s1 == 1) {
            $s2 = 0;
        } else {
            $s2 = 1;
        }
    } else {
        if ($s2 == 0) {
            $s1 = 1;
        } else {
            $s1 = 0;
        }
        $s2 = $s1;
    }
    if ($b == 1) {
        if ($s1 == $s2) {
            $s1 = 1;
            $s2 = 1;
        } else {
            $s1 = 0;
            $s2 = 0;
        }
    } else {
        if ($s1 == 1) {
            $s2 = 1 - $s2;
        }
        if ($s2 == 1) {
            $s1 = 1 - $s1;
        }
    }
    return $s1 + $s2;
}

# --- Function 5: State machine with conditional output mapping ---
# A 3-state machine runs for 2 steps, then maps the final state
# to an output value via elsif. The combination of state transitions
# + final mapping creates many paths. Proves output is in [10, 30].
# sig: (Int, Int) -> Int
# pre: $p >= 0 && $p <= 2 && $q >= 0 && $q <= 2
# post: $result >= 10 && $result <= 30
sub fsm_output_mapping {
    my ($p, $q) = @_;
    my $state = 0;
    if ($p == 0) {
        $state = 0;
    } elsif ($p == 1) {
        $state = 1;
    } else {
        $state = 2;
    }
    if ($state == 0) {
        if ($q == 0) {
            $state = 0;
        } elsif ($q == 1) {
            $state = 1;
        } else {
            $state = 2;
        }
    } elsif ($state == 1) {
        if ($q == 0) {
            $state = 2;
        } elsif ($q == 1) {
            $state = 0;
        } else {
            $state = 1;
        }
    } else {
        if ($q == 0) {
            $state = 1;
        } elsif ($q == 1) {
            $state = 2;
        } else {
            $state = 0;
        }
    }
    my $output;
    if ($state == 0) {
        $output = 10;
    } elsif ($state == 1) {
        $output = 20;
    } else {
        $output = 30;
    }
    return $output;
}
