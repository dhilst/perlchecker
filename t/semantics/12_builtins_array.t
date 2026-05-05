use strict;
use warnings;
use Test::More;

# scalar(@arr)
{
    my @a;
    is(scalar(@a), 0, "scalar: empty array has length 0");
}
{
    my @a = (1, 2, 3);
    is(scalar(@a), 3, "scalar: initialized array has correct length");
}
{
    my @a = (1, 2, 3);
    push @a, 4;
    is(scalar(@a), 4, "scalar: push increases length by 1");
}
{
    my @a = (1, 2, 3);
    pop @a;
    is(scalar(@a), 2, "scalar: pop decreases length by 1");
}

# push()
{
    my @a;
    push @a, 42;
    is($a[0], 42, "push: appended element accessible at index 0");
}
{
    my @a = (1, 2);
    push @a, 3;
    ok($a[0] == 1 && $a[1] == 2 && $a[2] == 3, "push: preserves existing elements and appends");
}
{
    my @a = (1, 2, 3);
    my $len = push @a, 4;
    is($len, 4, "push: returns new array length");
}

# pop()
{
    my @a = (1, 2, 3);
    my $v = pop @a;
    is($v, 3, "pop: returns last element");
}
{
    my @a = (1, 2, 3);
    pop @a;
    ok(scalar(@a) == 2 && $a[0] == 1 && $a[1] == 2, "pop: removes last element, preserves others");
}
{
    my @a;
    no warnings 'uninitialized';
    my $v = pop @a;
    ok(!defined($v), "pop: empty array returns undef");
}
{
    my @a = (42);
    pop @a;
    is(scalar(@a), 0, "pop: single-element array becomes empty");
}

# Array indexing
{
    my @a = (10, 20, 30);
    ok($a[0] == 10 && $a[1] == 20 && $a[2] == 30, "array: positional indexing");
}
{
    my @a = (10, 20, 30);
    ok($a[-1] == 30 && $a[-2] == 20 && $a[-3] == 10, "array: negative indexing wraps from end");
}
{
    my @a = (10, 20, 30);
    no warnings 'uninitialized';
    ok(!defined($a[5]), "array: out-of-bounds read returns undef");
}
{
    my @a = (10, 20, 30);
    $a[1] = 99;
    ok($a[0] == 10 && $a[1] == 99 && $a[2] == 30, "array: element assignment");
}

# Array initialization
{
    my @a = ();
    is(scalar(@a), 0, "array: empty list initialization");
}
{
    my @a = (1, 2, 3, 4, 5);
    is(scalar(@a), 5, "array: list literal initialization");
}

done_testing;
