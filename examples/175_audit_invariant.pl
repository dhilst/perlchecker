# Soundness audit: loop invariant preservation must respect precondition.
#
# Before fix: the preservation step freshened ALL variables including function
# parameters. This lost precondition constraints, causing valid invariants
# that depend on parameter bounds to be spuriously rejected.
#
# After fix: only variables assigned in the loop body are freshened.
# Parameters keep their original (precondition-constrained) values during
# preservation, matching actual Perl semantics where parameters are immutable
# within the loop.

# This invariant requires $n >= 11 (from precondition) for preservation:
# $sum + $n - 10 >= 0 only when $n >= 10 and $sum >= 0.
# Before fix: checker rejected this (freshened $n had no lower bound).
# After fix: checker correctly verifies it.
# sig: (I64) -> I64
# pre: $n >= 11 && $n <= 100
# post: $result >= 0
sub param_bounded_accumulator {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum >= 0 && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + $n - 10;
        $i = $i + 1;
    }
    return $sum;
}

# Standard invariant (no parameter dependence) still works.
# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 100
# post: $result == $n * 5
sub standard_multiply {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum == $i * 5 && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + 5;
        $i = $i + 1;
    }
    return $sum;
}

# Wrong postcondition must still be caught.
# sig: (I64) -> I64
# pre: $n >= 11 && $n <= 100
# post: $result == 999
sub wrong_post_caught {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum >= 0 && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + $n - 10;
        $i = $i + 1;
    }
    return $sum;
}

# Multi-parameter: both parameters constrained, invariant uses both.
# sig: (I64, I64) -> I64
# pre: $n >= 1 && $n <= 50 && $k >= 2 && $k <= 10
# post: $result == $n * $k
sub two_param_loop {
    my ($n, $k) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum == $i * $k && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + $k;
        $i = $i + 1;
    }
    return $sum;
}
