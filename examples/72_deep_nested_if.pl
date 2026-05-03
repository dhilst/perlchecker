# =============================================================
# Round 72: Deep 5-level nested if/unless path explosion
# =============================================================
# Functions with 5 levels of nested conditional blocks mixing
# if and unless, creating 2^5 = 32 paths. Each level accumulates
# into a result variable, stressing SSA phi-node generation at
# every merge point.

# --- Function 1: Pure 5-level nested if/else ---
# Each level adds a different amount based on branch taken.
# Level 1: +10 or +1, Level 2: +8 or +2, Level 3: +6 or +3,
# Level 4: +4 or +4, Level 5: +5 or +0
# Min path: 1+2+3+4+0 = 10, Max path: 10+8+6+4+5 = 33
# sig: (Int, Int, Int, Int, Int) -> Int
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20 && $c >= 0 && $c <= 20 && $d >= 0 && $d <= 20 && $e >= 0 && $e <= 20
# post: $result >= 10 && $result <= 33
sub five_level_if {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 10) {
        $r += 10;
        if ($b > 10) {
            $r += 8;
            if ($c > 10) {
                $r += 6;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 3;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            }
        } else {
            $r += 2;
            if ($c > 10) {
                $r += 6;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 3;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            }
        }
    } else {
        $r += 1;
        if ($b > 10) {
            $r += 8;
            if ($c > 10) {
                $r += 6;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 3;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            }
        } else {
            $r += 2;
            if ($c > 10) {
                $r += 6;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 3;
                if ($d > 10) {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 4;
                    if ($e > 10) {
                        $r += 5;
                    } else {
                        $r += 0;
                    }
                }
            }
        }
    }
    return $r;
}

# --- Function 2: Mixed if/unless 5-level nesting ---
# Alternates if and unless at each level to stress negated-condition
# phi generation. unless(cond) means "if not cond", so the true-branch
# of unless runs when condition is false.
# Level 1 (if $a>5):    T->+10, F->+2
# Level 2 (unless $b>8): T(b<=8)->+7, F(b>8)->+3
# Level 3 (if $c>4):    T->+6, F->+1
# Level 4 (unless $d>6): T(d<=6)->+5, F(d>6)->+2
# Level 5 (if $e>3):    T->+4, F->+1
# Min: 2+3+1+2+1 = 9, Max: 10+7+6+5+4 = 32
# sig: (Int, Int, Int, Int, Int) -> Int
# pre: $a >= 0 && $a <= 15 && $b >= 0 && $b <= 15 && $c >= 0 && $c <= 15 && $d >= 0 && $d <= 15 && $e >= 0 && $e <= 15
# post: $result >= 9 && $result <= 32
sub mixed_if_unless_deep {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 5) {
        $r += 10;
    } else {
        $r += 2;
    }
    unless ($b > 8) {
        $r += 7;
    } else {
        $r += 3;
    }
    if ($c > 4) {
        $r += 6;
    } else {
        $r += 1;
    }
    unless ($d > 6) {
        $r += 5;
    } else {
        $r += 2;
    }
    if ($e > 3) {
        $r += 4;
    } else {
        $r += 1;
    }
    return $r;
}

# --- Function 3: Deeply nested if-inside-unless pattern ---
# True 5-level deep nesting (not sequential): each conditional is
# nested inside the previous one's branch. Creates exactly 32 leaf
# paths since each level has 2 branches fully expanded.
# The accumulator gains: L1: 16/1, L2: 8/1, L3: 4/1, L4: 2/1, L5: 1/0
# Min: 1+1+1+1+0 = 4, Max: 16+8+4+2+1 = 31
# sig: (Int, Int, Int, Int, Int) -> Int
# pre: $a >= 0 && $a <= 20 && $b >= 0 && $b <= 20 && $c >= 0 && $c <= 20 && $d >= 0 && $d <= 20 && $e >= 0 && $e <= 20
# post: $result >= 4 && $result <= 31
sub deep_nested_alternating {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 10) {
        $r += 16;
        unless ($b > 10) {
            $r += 8;
            if ($c > 10) {
                $r += 4;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 1;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            }
        } else {
            $r += 1;
            if ($c > 10) {
                $r += 4;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 1;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            }
        }
    } else {
        $r += 1;
        unless ($b > 10) {
            $r += 8;
            if ($c > 10) {
                $r += 4;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 1;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            }
        } else {
            $r += 1;
            if ($c > 10) {
                $r += 4;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            } else {
                $r += 1;
                unless ($d > 10) {
                    $r += 2;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                } else {
                    $r += 1;
                    if ($e > 10) {
                        $r += 1;
                    } else {
                        $r += 0;
                    }
                }
            }
        }
    }
    return $r;
}

# --- Function 4: 5-level nesting with die on impossible paths ---
# Uses preconditions to make certain branches unreachable, then
# places die on those paths. The reachable paths accumulate
# a bounded result.
# With $a >= 10, the else-branch at level 1 (a <= 5) is unreachable.
# With $b <= 5, the if-branch at level 2 (b > 10) is unreachable.
# Reachable paths: level 1 always takes if, level 2 always takes unless-true
# Levels 3-5 still have 2 branches each = 8 paths.
# Reachable: r starts at 0, +10 (L1), +7 (L2), then L3: +6 or +1,
# L4: +5 or +2, L5: +4 or +1.
# Min reachable: 10+7+1+2+1 = 21, Max reachable: 10+7+6+5+4 = 32
# sig: (Int, Int, Int, Int, Int) -> Int
# pre: $a >= 10 && $a <= 15 && $b >= 0 && $b <= 5 && $c >= 0 && $c <= 15 && $d >= 0 && $d <= 15 && $e >= 0 && $e <= 15
# post: $result >= 21 && $result <= 32
sub nested_with_die_paths {
    my ($a, $b, $c, $d, $e) = @_;
    my $r = 0;
    if ($a > 5) {
        $r += 10;
        unless ($b > 10) {
            $r += 7;
            if ($c > 7) {
                $r += 6;
                unless ($d > 10) {
                    $r += 5;
                    if ($e > 7) {
                        $r += 4;
                    } else {
                        $r += 1;
                    }
                } else {
                    $r += 2;
                    if ($e > 7) {
                        $r += 4;
                    } else {
                        $r += 1;
                    }
                }
            } else {
                $r += 1;
                unless ($d > 10) {
                    $r += 5;
                    if ($e > 7) {
                        $r += 4;
                    } else {
                        $r += 1;
                    }
                } else {
                    $r += 2;
                    if ($e > 7) {
                        $r += 4;
                    } else {
                        $r += 1;
                    }
                }
            }
        } else {
            die "unreachable: b > 10 impossible";
        }
    } else {
        die "unreachable: a <= 5 impossible";
    }
    return $r;
}
