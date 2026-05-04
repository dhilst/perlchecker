# =============================================================
# Round 177: scalar(@arr) tracking after push/pop/assign
# =============================================================
# After a sequence of push/pop/assignment, scalar(@arr) must
# reflect the correct length.

# --- push twice, pop once: net +1 ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n >= 0 && $n <= 10
# post: $result == $n + 1
sub push_push_pop_len {
    my ($arr, $n) = @_;
    push(@arr, 10);
    push(@arr, 20);
    my $x = pop(@arr);
    return scalar(@arr);
}

# --- assign beyond current length extends the array ---
# In Perl, $arr[10] = 5 on a 3-element array makes scalar(@arr) == 11.
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) == 3
# post: $result == 11
sub assign_extends_length {
    my ($arr) = @_;
    $arr[10] = 5;
    return scalar(@arr);
}

# --- assign within bounds does not change length ---
# sig: (Array<Int>, Int) -> Int
# pre: scalar(@arr) == $n && $n >= 3
# post: $result == $n
sub assign_within_bounds_len {
    my ($arr, $n) = @_;
    $arr[1] = 99;
    return scalar(@arr);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

our @arr;

Property {
    ##[ n <- Int(range=>[0,10], sized=>0) ]##
    @arr = (0) x $n;
    my $result = push_push_pop_len(undef, $n);
    $result == $n + 1;
}, name => "push_push_pop_len: post holds";

Property {
    ##[ ]##
    @arr = (1, 2, 3);
    my $result = assign_extends_length(undef);
    $result == 11;
}, name => "assign_extends_length: post holds";

Property {
    ##[ n <- Int(range=>[3,10], sized=>0) ]##
    @arr = (0) x $n;
    my $result = assign_within_bounds_len(undef, $n);
    $result == $n;
}, name => "assign_within_bounds_len: post holds";
