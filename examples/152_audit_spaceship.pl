# =============================================================
# Round 152: Soundness audit — spaceship <=> and cmp desugaring
# =============================================================
# Validates ite(a < b, -1, ite(a == b, 0, 1)) encoding for <=>
# and ite(a lt b, -1, ite(a eq b, 0, 1)) encoding for cmp.
# Tests: equal-value zero, antisymmetry, arithmetic on result,
# and cmp string ordering correctness.

# --- Spaceship with equal values must be exactly 0 ---
# sig: (I64) -> I64
# pre: $x >= -100 && $x <= 100
# post: $result == 0
sub spaceship_equal {
    my ($x) = @_;
    return $x <=> $x;
}

# --- Spaceship antisymmetry: (a <=> b) == -(b <=> a) ---
# sig: (I64, I64) -> I64
# pre: $a >= -50 && $a <= 50 && $b >= -50 && $b <= 50
# post: $result == 0
sub spaceship_antisymmetry {
    my ($a, $b) = @_;
    my $fwd = $a <=> $b;
    my $rev = $b <=> $a;
    return $fwd + $rev;
}

# --- Spaceship result is always in {-1, 0, 1} ---
# sig: (I64, I64) -> I64
# pre: $a >= -1000 && $a <= 1000 && $b >= -1000 && $b <= 1000
# post: ($result == -1 || $result == 0 || $result == 1)
sub spaceship_range {
    my ($a, $b) = @_;
    return $a <=> $b;
}

# --- Spaceship arithmetic: (a <=> b) + 1 in {0, 1, 2} ---
# sig: (I64, I64) -> I64
# pre: $a >= -100 && $a <= 100 && $b >= -100 && $b <= 100
# post: $result >= 0 && $result <= 2
sub spaceship_plus_one {
    my ($a, $b) = @_;
    my $cmp = $a <=> $b;
    return $cmp + 1;
}

# --- Spaceship consistency: (a <=> b) < 0 iff a < b ---
# sig: (I64, I64) -> I64
# pre: $a >= -100 && $a <= 100 && $b >= -100 && $b <= 100 && $a < $b
# post: $result == -1
sub spaceship_lt_implies_neg {
    my ($a, $b) = @_;
    return $a <=> $b;
}

# --- Spaceship consistency: (a <=> b) > 0 iff a > b ---
# sig: (I64, I64) -> I64
# pre: $a >= -100 && $a <= 100 && $b >= -100 && $b <= 100 && $a > $b
# post: $result == 1
sub spaceship_gt_implies_pos {
    my ($a, $b) = @_;
    return $a <=> $b;
}

# --- String cmp with equal strings must be 0 ---
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result == 0
sub cmp_equal {
    my ($s) = @_;
    return $s cmp $s;
}

# --- String cmp antisymmetry: (a cmp b) == -(b cmp a) ---
# sig: (Str, Str) -> I64
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: $result == 0
sub cmp_antisymmetry {
    my ($a, $b) = @_;
    my $fwd = $a cmp $b;
    my $rev = $b cmp $a;
    return $fwd + $rev;
}

# --- String cmp result is always in {-1, 0, 1} ---
# sig: (Str, Str) -> I64
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: ($result == -1 || $result == 0 || $result == 1)
sub cmp_range {
    my ($a, $b) = @_;
    return $a cmp $b;
}

# --- Spaceship squared: result^2 is 0 or 1 ---
# sig: (I64, I64) -> I64
# pre: $a >= -100 && $a <= 100 && $b >= -100 && $b <= 100
# post: ($result == 0 || $result == 1)
sub spaceship_squared {
    my ($a, $b) = @_;
    my $cmp = $a <=> $b;
    return $cmp * $cmp;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[-100,100], sized=>0) ]##
    my $result = spaceship_equal($x);
    $result == 0;
}, name => "spaceship_equal: post holds";

Property {
    ##[ a <- Int(range=>[-50,50], sized=>0), b <- Int(range=>[-50,50], sized=>0) ]##
    my $result = spaceship_antisymmetry($a, $b);
    $result == 0;
}, name => "spaceship_antisymmetry: post holds";

Property {
    ##[ a <- Int(range=>[-1000,1000], sized=>0), b <- Int(range=>[-1000,1000], sized=>0) ]##
    my $result = spaceship_range($a, $b);
    ($result == -1 || $result == 0 || $result == 1);
}, name => "spaceship_range: post holds";

Property {
    ##[ a <- Int(range=>[-100,100], sized=>0), b <- Int(range=>[-100,100], sized=>0) ]##
    my $result = spaceship_plus_one($a, $b);
    $result >= 0 && $result <= 2;
}, name => "spaceship_plus_one: post holds";

Property {
    ##[ a <- Int(range=>[-100,100], sized=>0), b <- Int(range=>[-100,100], sized=>0) ]##
    ($a < $b) ? (spaceship_lt_implies_neg($a, $b) == -1) : 1;
}, name => "spaceship_lt_implies_neg: post holds";

Property {
    ##[ a <- Int(range=>[-100,100], sized=>0), b <- Int(range=>[-100,100], sized=>0) ]##
    ($a > $b) ? (spaceship_gt_implies_pos($a, $b) == 1) : 1;
}, name => "spaceship_gt_implies_pos: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,10]) ]##
    my $result = cmp_equal($s);
    $result == 0;
}, name => "cmp_equal: post holds";

Property {
    ##[ a <- String(charset=>"a-z", length=>[1,5]), b <- String(charset=>"a-z", length=>[1,5]) ]##
    my $result = cmp_antisymmetry($a, $b);
    $result == 0;
}, name => "cmp_antisymmetry: post holds";

Property {
    ##[ a <- String(charset=>"a-z", length=>[1,5]), b <- String(charset=>"a-z", length=>[1,5]) ]##
    my $result = cmp_range($a, $b);
    ($result == -1 || $result == 0 || $result == 1);
}, name => "cmp_range: post holds";

Property {
    ##[ a <- Int(range=>[-100,100], sized=>0), b <- Int(range=>[-100,100], sized=>0) ]##
    my $result = spaceship_squared($a, $b);
    ($result == 0 || $result == 1);
}, name => "spaceship_squared: post holds";
