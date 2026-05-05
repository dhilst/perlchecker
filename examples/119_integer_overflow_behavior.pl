# Round 119: Perl integer overflow behavior diverges from perlchecker Int semantics

# sig: (I64) -> I64
# pre: $dummy == 0
# post: $result == 0
sub uv_max_plus_one_is_greater {
    my ($dummy) = @_;
    my $max = ~0;
    my $sum = $max + 1;
    # assert: $sum > $max
    if ($sum > $max) {
        return 1;
    }
    return 0;
}

# sig: (I64) -> I64
# pre: $dummy == 0
# post: $result == 0
sub iv_min_minus_one_is_smaller {
    my ($dummy) = @_;
    my $min = -(1 << 63);
    my $diff = $min - 1;
    # assert: $diff < $min
    if ($diff < $min) {
        return 1;
    }
    return 0;
}

use Test::More;

is(
    uv_max_plus_one_is_greater(0),
    0,
    'Perl does not keep ~0 + 1 strictly greater than ~0 after numeric overflow',
);

is(
    iv_min_minus_one_is_smaller(0),
    0,
    'Perl does not keep -(1<<63) - 1 strictly smaller than -(1<<63) after numeric underflow',
);

done_testing;
