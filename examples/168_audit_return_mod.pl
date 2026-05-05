#!/usr/bin/env perl
use strict;
use warnings;

# Test 1: Multiple early returns with 'if' modifier - basic case
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 20
# post: $result >= 1 && $result <= 3
sub classify_basic {
    my ($x) = @_;
    return 1 if ($x > 10);
    return 2 if ($x > 5);
    return 3;
}

# Test 2: return with 'unless' modifier
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 20
# post: $result >= 0 && $result <= 1
sub is_positive {
    my ($x) = @_;
    return 0 unless ($x > 0);
    return 1;
}

# Test 3: Multiple return-if in sequence with tight constraints
# This tests that path conditions accumulate correctly
# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 100
# post: $result == 1 || $result == 2 || $result == 3 || $result == 4
sub multi_return {
    my ($x) = @_;
    return 1 if ($x > 75);
    return 2 if ($x > 50);
    return 3 if ($x > 25);
    return 4;
}

# Test 4: return-unless with arithmetic in return value
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= 0
sub abs_like {
    my ($x) = @_;
    return $x unless ($x < 5);
    return 5 - $x;
}

# Test 5: return-if with expression in return value
# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result >= 1 && $result <= 11
sub add_one_if_positive {
    my ($x) = @_;
    return $x + 1 if ($x > 0);
    return 1;
}

# Test 6: return value depends on path condition being true
# On the x>=y path, x-y>=0; on the x<y path, y-x>0.
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result >= 0
sub safe_difference {
    my ($x, $y) = @_;
    return $x - $y if ($x >= $y);
    return $y - $x;
}

# Test 7: unless with compound condition and path reasoning
# After "return 0 unless (x>3 && y>3)", continuation has x>3 AND y>3
# sig: (I64, I64) -> I64
# pre: $x >= 0 && $x <= 10 && $y >= 0 && $y <= 10
# post: $result == 0 || $result >= 8
sub unless_compound {
    my ($x, $y) = @_;
    return 0 unless ($x > 3 && $y > 3);
    return $x + $y;
}
