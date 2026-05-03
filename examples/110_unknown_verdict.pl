# Round 110: UNKNOWN verdict for loops exceeding unroll bound
#
# Strategy dispatch:
#   - Loop with # inv: annotation -> inductive verification (any iteration count)
#   - Loop without invariant -> bounded unrolling; if bound exceeded -> UNKNOWN verdict

# A function with a loop that exceeds the unroll bound.
# This should produce UNKNOWN with guidance, not a hard error.
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 100
# post: $result >= 0
sub needs_invariant {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    while ($i < $n) {
        $sum = $sum + 1;
        $i = $i + 1;
    }
    return $sum;
}

# Same function but WITH an invariant -- should verify.
# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 100
# post: $result >= 0 && $result == $n
sub has_invariant {
    my ($n) = @_;
    my $sum = 0;
    my $i = 0;
    # inv: $sum == $i && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + 1;
        $i = $i + 1;
    }
    return $sum;
}
