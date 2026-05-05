# =============================================================
# Round 153: Soundness audit — min() and max() builtins
# =============================================================
# Verify the ITE encoding of min/max handles:
#   - basic semantics (returns smaller/larger)
#   - identity: min(x,x)==x, max(x,x)==x
#   - negative numbers: min(-5,-3)==-5, max(-5,-3)==-3
#   - commutativity: min(x,y)==min(y,x), max(x,y)==max(y,x)

# --- min(x, x) == x (identity) ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result == $x
sub min_identity {
    my ($x) = @_;
    return min($x, $x);
}

# --- max(x, x) == x (identity) ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result == $x
sub max_identity {
    my ($x) = @_;
    return max($x, $x);
}

# --- min(-5, -3) == -5 (negative numbers) ---
# sig: (I64) -> I64
# pre: $x == 0
# post: $result == -5
sub min_negative_concrete {
    my ($x) = @_;
    return min(-5, -3);
}

# --- max(-5, -3) == -3 (negative numbers) ---
# sig: (I64) -> I64
# pre: $x == 0
# post: $result == -3
sub max_negative_concrete {
    my ($x) = @_;
    return max(-5, -3);
}

# --- min is commutative: min(x,y) == min(y,x) ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result == min($y, $x)
sub min_commutative {
    my ($x, $y) = @_;
    return min($x, $y);
}

# --- max is commutative: max(x,y) == max(y,x) ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result == max($y, $x)
sub max_commutative {
    my ($x, $y) = @_;
    return max($x, $y);
}

# --- min returns value <= both args ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result <= $x && $result <= $y
sub min_le_both {
    my ($x, $y) = @_;
    return min($x, $y);
}

# --- max returns value >= both args ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result >= $x && $result >= $y
sub max_ge_both {
    my ($x, $y) = @_;
    return max($x, $y);
}

# --- min(x, y) is one of x or y ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result == $x || $result == $y
sub min_is_arg {
    my ($x, $y) = @_;
    return min($x, $y);
}

# --- max(x, y) is one of x or y ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result == $x || $result == $y
sub max_is_arg {
    my ($x, $y) = @_;
    return max($x, $y);
}

# --- min(x, y) <= max(x, y) always ---
# sig: (I64, I64) -> I64
# pre: $x >= -100 && $x <= 100 && $y >= -100 && $y <= 100
# post: $result <= max($x, $y)
sub min_le_max {
    my ($x, $y) = @_;
    return min($x, $y);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;
use List::Util qw(min max);

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0) ]##
    my $r = min_identity($x);
    $r == $x;
}, name => "min_identity: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0) ]##
    my $r = max_identity($x);
    $r == $x;
}, name => "max_identity: post holds";

Property {
    ##[ x <- Int(range=>[0,0], sized=>0) ]##
    my $r = min_negative_concrete($x);
    $r == -5;
}, name => "min_negative_concrete: post holds";

Property {
    ##[ x <- Int(range=>[0,0], sized=>0) ]##
    my $r = max_negative_concrete($x);
    $r == -3;
}, name => "max_negative_concrete: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = min_commutative($x, $y);
    $r == min($y, $x);
}, name => "min_commutative: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = max_commutative($x, $y);
    $r == max($y, $x);
}, name => "max_commutative: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = min_le_both($x, $y);
    $r <= $x && $r <= $y;
}, name => "min_le_both: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = max_ge_both($x, $y);
    $r >= $x && $r >= $y;
}, name => "max_ge_both: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = min_is_arg($x, $y);
    $r == $x || $r == $y;
}, name => "min_is_arg: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = max_is_arg($x, $y);
    $r == $x || $r == $y;
}, name => "max_is_arg: post holds";

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0), y <- Int(range=>[-100,100], sized=>0) ]##
    my $r = min_le_max($x, $y);
    $r <= max($x, $y);
}, name => "min_le_max: post holds";
