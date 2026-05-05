# Soundness audit: I64 division truncation
#
# Perl's `/` returns a float (7/2 == 3.5), but the checker previously
# modeled I64 / I64 as truncating division (7/2 == 3). This unsoundness
# meant the checker could verify postconditions that fail at runtime.
#
# Fix: bare `/` on I64 operands is now rejected; use int($x / $y) to
# make truncation explicit (matching Perl's int() semantics).

# This function correctly uses int() for truncating division.
# sig: (I64, I64) -> I64
# pre: $y != 0
# post: $result == int($x / $y)
sub truncating_division {
    my ($x, $y) = @_;
    return int($x / $y);
}
