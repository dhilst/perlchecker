use strict;
use warnings;
use Test::More;

# Scalar references
{
    my $x = 42;
    my $ref = \$x;
    is($$ref, 42, "ref: scalar reference and dereference");
}
{
    my $x = 42;
    my $ref = \$x;
    $$ref = 99;
    is($x, 99, 'ref: write-through — $$ref = 99 modifies original');
}
{
    my $x = 42;
    my $ref = \$x;
    $x = 100;
    is($$ref, 100, "ref: reflects changes to original variable");
}
{
    my $x = 42;
    my $ref = \$x;
    is(ref($ref), "SCALAR", "ref: ref() returns 'SCALAR' for scalar ref");
}

# Array references
{
    my @a = (1, 2, 3);
    my $ref = \@a;
    ok($ref->[0] == 1 && $ref->[1] == 2 && $ref->[2] == 3, "ref: array reference with arrow access");
}
{
    my @a = (1, 2, 3);
    my $ref = \@a;
    $ref->[1] = 99;
    is($a[1], 99, "ref: array ref write-through via arrow");
}
{
    my @a = (1, 2, 3);
    my $ref = \@a;
    is(ref($ref), "ARRAY", "ref: ref() returns 'ARRAY' for array ref");
}
{
    my @a = (1, 2, 3);
    my $ref = \@a;
    is(scalar(@{$ref}), 3, "ref: deref array ref for length");
}

# Hash references
{
    my %h = ("a" => 1, "b" => 2);
    my $ref = \%h;
    ok($ref->{"a"} == 1 && $ref->{"b"} == 2, "ref: hash reference with arrow access");
}
{
    my %h = ("a" => 1);
    my $ref = \%h;
    $ref->{"a"} = 99;
    is($h{"a"}, 99, "ref: hash ref write-through via arrow");
}
{
    my %h = ("a" => 1);
    my $ref = \%h;
    is(ref($ref), "HASH", "ref: ref() returns 'HASH' for hash ref");
}

# Reference identity
{
    my $x = 42;
    my $r1 = \$x;
    my $r2 = \$x;
    is($$r1, $$r2, "ref: two refs to same variable see same value");
}
{
    my $x = 42;
    my $r1 = \$x;
    my $r2 = \$x;
    $$r1 = 99;
    is($$r2, 99, "ref: write through one ref visible through another");
}

done_testing;
