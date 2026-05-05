use strict;
use warnings;
no warnings 'uninitialized';
use Test::More;

# Hash read/write
{
    my %h;
    $h{"key"} = 42;
    is($h{"key"}, 42, "hash: write and read back");
}
{
    my %h;
    ok(!defined($h{"missing"}), "hash: missing key returns undef");
}
{
    my %h;
    is($h{"missing"} + 0, 0, "hash: missing key numifies to 0");
}
{
    my %h = ("a" => 1, "b" => 2);
    ok($h{"a"} == 1 && $h{"b"} == 2, "hash: initialization with key-value pairs");
}
{
    my %h = ("a" => 1, "b" => 2);
    $h{"a"} = 99;
    ok($h{"a"} == 99 && $h{"b"} == 2, "hash: overwrite existing key");
}

# exists()
{
    my %h = ("a" => 1);
    ok(exists($h{"a"}), "exists: returns true for present key");
}
{
    my %h = ("a" => 1);
    is(exists($h{"b"}), "", "exists: returns '' for absent key");
}
{
    my %h;
    $h{"key"} = 0;
    ok(exists($h{"key"}), "exists: key with value 0 still exists");
}
{
    my %h;
    $h{"key"} = undef;
    ok(exists($h{"key"}), "exists: key with value undef still exists");
}
{
    my %h;
    my $v = $h{"key"};
    ok(!exists($h{"key"}), "exists: read does not autovivify key");
}

# defined()
ok(defined(42),      "defined: integer is defined");
ok(defined("hello"), "defined: string is defined");
ok(defined(""),      "defined: empty string is defined");
ok(defined(0),       "defined: zero is defined");
ok(!defined(undef),  "defined: undef is not defined");
{
    my %h;
    ok(!defined($h{"missing"}), "defined: missing hash value is undef");
}
{
    my %h = ("key" => 42);
    ok(defined($h{"key"}), "defined: present hash value is defined");
}

# exists vs defined
{
    my %h;
    $h{"key"} = undef;
    ok(exists($h{"key"}) && !defined($h{"key"}), "exists/defined: key can exist but have undef value");
}

done_testing;
