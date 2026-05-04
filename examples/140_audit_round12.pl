# Round 140: Soundness fix — index() on strings longer than MAX_STR_LEN
#
# Bug: The previous encode_index_of hand-rolled a loop that only searched
# positions 0..=32 (MAX_STR_LEN).  Strings produced by concatenation can
# exceed this bound, so index() would wrongly return -1 for matches at
# position > 32.  This allowed false "verified" results.
#
# Fix: Use Z3's native str.indexof which handles arbitrary-length strings.
# Clamp negative start to 0 to match Perl's semantics.

# --- Function 1: index on concatenated string finds match beyond position 32 ---
# The concatenation $a . $b produces a 34-char string.  "x" is guaranteed
# to appear in $b (at position 33), so index must NOT return -1.
# sig: (Str, Str) -> Int
# pre: length($a) == 32 && length($b) == 2 && ends_with($b, "x") == 1 && contains($a, "x") == 0 && starts_with($b, "x") == 0
# post: $result >= 0
sub index_finds_past_32 {
    my ($a, $b) = @_;
    my $combined = $a . $b;
    return index($combined, "x", 0);
}

# --- Function 2: basic index still works correctly ---
# sig: (Str) -> Int
# pre: length($s) >= 5 && starts_with($s, "hello") == 1
# post: $result == 2
sub index_basic {
    my ($s) = @_;
    return index($s, "l", 0);
}

# --- Function 3: index with negative start behaves like start=0 ---
# sig: (Str) -> Int
# pre: length($s) >= 3 && starts_with($s, "abc") == 1
# post: $result == 0
sub index_negative_start {
    my ($s) = @_;
    return index($s, "a", -10);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 3;
sub check_prop { my ($name,$g,$c,$n)=@_; $n//=1000; for(1..$n){my @a=$g->($_); $c->(@a)||do{diag("FAIL $name: @a");return 0}} 1 }

ok(check_prop("index_finds_past_32",
    sub { my $a = join("", map { chr(ord("a") + int(rand(25))) } 1..32); return ($a, "a" . "x") },
    sub { my ($a, $b) = @_; my $combined = $a . $b; index($combined, "x", 0) >= 0 },
), "index_finds_past_32: post holds");

ok(check_prop("index_basic",
    sub { return ("hello" . join("", map { chr(ord("a") + int(rand(26))) } 1..5)) },
    sub { my ($s) = @_; index($s, "l", 0) == 2 },
), "index_basic: post holds");

ok(check_prop("index_negative_start",
    sub { return ("abc" . join("", map { chr(ord("d") + int(rand(22))) } 1..5)) },
    sub { my ($s) = @_; index($s, "a", -10) == 0 },
), "index_negative_start: post holds");
