# Soundness audit: do-while body must execute at least once, and the
# verifier must not silently drop the loop when max_loop_unroll == 0.
#
# With --max-loop-unroll 0, the desugaring emits only the body (no While
# node, no LoopBoundExceeded guard).  The verifier then falsely concludes
# that the loop terminates after one iteration.
#
# Example: do { $x += 10 } while ($x < 100); starting from $x == 0
# Perl executes the body 10 times => $x == 100.
# With unroll 0 the verifier sees only one body => $x == 10 and VERIFIES
# a postcondition that is wrong ($result == 10 is false in Perl).

# sig: (I64) -> I64
# pre: $n == 0
# post: $result == 10
sub do_while_unsound {
    my ($n) = @_;
    my $x = $n;
    do {
        $x += 10;
    } while ($x < 100);
    return $x;
}
