use strict;
use warnings;
no warnings 'uninitialized';
use Test::More;

# Creation and access
{
    my %h = ("a" => 1, "b" => 2, "c" => 3);
    ok($h{"a"} == 1 && $h{"b"} == 2 && $h{"c"} == 3, "hash: initialization with fat-comma pairs");
}
{
    my %h;
    is(scalar(keys %h), 0, "hash: empty hash has 0 keys");
}

# Missing key semantics
{
    my %h = ("a" => 1);
    ok(!defined($h{"missing"}), "hash: missing key returns undef");
}
{
    my %h = ("a" => 1);
    is($h{"missing"} + 0, 0, "hash: missing key numifies to 0");
}
{
    my %h = ("a" => 1);
    is("" . ($h{"missing"} // ""), "", "hash: missing key stringifies to empty");
}

# Write semantics
{
    my %h;
    $h{"new"} = 42;
    ok(exists($h{"new"}) && $h{"new"} == 42, "hash: write creates key");
}
{
    my %h = ("key" => 1);
    $h{"key"} = 99;
    is($h{"key"}, 99, "hash: overwrite replaces value");
}

# exists vs defined vs truthiness
{
    my %h;
    $h{"zero"} = 0;
    ok(exists($h{"zero"}) && defined($h{"zero"}) && !$h{"zero"}, "hash: value 0 — exists, defined, but false");
}
{
    my %h;
    $h{"empty"} = "";
    ok(exists($h{"empty"}) && defined($h{"empty"}) && !$h{"empty"}, "hash: value '' — exists, defined, but false");
}
{
    my %h;
    $h{"undef"} = undef;
    ok(exists($h{"undef"}) && !defined($h{"undef"}), "hash: value undef — exists but not defined");
}
{
    my %h;
    ok(!exists($h{"absent"}) && !defined($h{"absent"}), "hash: absent key — neither exists nor defined");
}

# Read does NOT autovivify
{
    my %h;
    my $v = $h{"key"};
    ok(!exists($h{"key"}), "hash: reading missing key does not create it");
}
{
    my %h;
    my $d = defined($h{"key"});
    ok(!exists($h{"key"}), "hash: defined() check does not autovivify");
}

# Special keys
{
    my %h;
    $h{""} = 42;
    ok(exists($h{""}) && $h{""} == 42, "hash: empty string is a valid key");
}
{
    my %h;
    $h{"0"} = 42;
    ok(exists($h{"0"}) && $h{"0"} == 42, "hash: '0' is a valid key");
}

done_testing;
