# Soundness audit: extern precondition must be CHECKED at the call site.
#
# Bug: if the extern precondition was merely assumed (not checked), the verifier
# would report "Verified" even when the caller violates the precondition, because
# the violating paths become infeasible under the false assumption.
#
# This example demonstrates the fix: the caller's pre guarantees sqrt_int's pre.
#
# extern: sqrt_int (Int) -> Int pre: $a > 0 post: $result >= 0 && $result * $result <= $a

# sig: (Int) -> Int
# pre: $x > 0 && $x <= 100
# post: $result >= 0
sub use_sqrt_safe {
    my ($x) = @_;
    my $result = sqrt_int($x);
    return $result;
}

sub sqrt_int { return int(sqrt(abs($_[0]))) }

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ x <- Int(range=>[1,100], sized=>0) ]##
    my $result = use_sqrt_safe($x);
    $result >= 0;
}, name => "use_sqrt_safe: post holds";
