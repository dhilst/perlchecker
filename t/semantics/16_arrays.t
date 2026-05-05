use strict;
use warnings;
no warnings 'uninitialized';
use Test::More;

# Positive indexing
{
    my @a = (10, 20, 30, 40, 50);
    ok($a[0] == 10 && $a[4] == 50, "array: first and last element access");
}
{
    my @a = (1);
    is($a[0], 1, "array: single-element array");
}

# Negative indexing
{
    my @a = (10, 20, 30, 40, 50);
    is($a[-1], 50, "array: -1 is last element");
}
{
    my @a = (10, 20, 30, 40, 50);
    is($a[-5], 10, "array: -N (N=length) is first element");
}
{
    my @a = (10, 20, 30);
    is($a[-1], $a[scalar(@a) - 1], "array: negative index equivalent to length + index");
}

# Out-of-bounds
{
    my @a = (1, 2, 3);
    ok(!defined($a[10]), "array: OOB positive index returns undef");
}
{
    my @a = (1, 2, 3);
    is($a[10] + 0, 0, "array: OOB read numifies to 0");
}
{
    my @a = (1, 2, 3);
    is("" . ($a[10] // ""), "", "array: OOB read stringifies to empty");
}

# Write extends array
{
    my @a = (1, 2, 3);
    $a[5] = 99;
    is(scalar(@a), 6, "array: write beyond end extends array");
}
{
    my @a = (1, 2, 3);
    $a[5] = 99;
    ok(!defined($a[3]) && !defined($a[4]) && $a[5] == 99, "array: gaps filled with undef when extending");
}

# Boolean context
{
    my @a = (1);
    ok(scalar(@a), "array: non-empty array is true in boolean context");
}
{
    my @a;
    ok(!@a, "array: empty array is false in boolean context");
}

# $#arr
{
    my @a = (10, 20, 30);
    is($#a, 2, 'array: $#arr is last valid index (length - 1)');
}
{
    my @a;
    is($#a, -1, 'array: $#arr is -1 for empty array');
}

# Write via negative index
{
    my @a = (10, 20, 30);
    $a[-1] = 99;
    ok($a[0] == 10 && $a[1] == 20 && $a[2] == 99, "array: write via negative index modifies correct element");
}
{
    my @a = (10, 20, 30);
    $a[-3] = 99;
    is($a[0], 99, "array: write via -N (N=length) modifies first element");
}

done_testing;
