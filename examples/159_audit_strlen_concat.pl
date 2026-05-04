# =============================================================
# Round 159: Soundness audit — length() after string concatenation
# =============================================================
# Verify that length($a . $b) == length($a) + length($b) holds
# and that length() after replace() on concatenated strings is
# correctly modeled.
#
# Bug found: replace() on a concatenated string (up to 64 chars)
# only iterated MAX_STR_LEN (32) times, missing occurrences past
# position 32.  Fixed by iterating 2*MAX_STR_LEN times.

# --- Basic: length of concat equals sum of lengths ---
# sig: (Str, Str) -> Int
# pre: length($a) >= 0 && length($b) >= 0
# post: $result == length($a) + length($b)
sub len_concat_sum {
    my ($a, $b) = @_;
    return length($a . $b);
}

# --- length(substr(concat, ...)) is bounded ---
# sig: (Str, Str) -> Int
# pre: length($a) >= 1 && length($b) >= 1
# post: $result >= 0 && $result <= length($a) + length($b)
sub len_substr_of_concat {
    my ($a, $b) = @_;
    my $c = $a . $b;
    my $s = substr($c, 0, length($a));
    return length($s);
}

# --- length after replace on concat (the bug case) ---
# Replace all "x" with "yy" in a 64-char string of x's.
# Perl: length == 128.  Before fix, checker only replaced 32 of 64.
# sig: (Str, Str) -> Int
# pre: $a eq "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" && $b eq "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# post: $result == 128
sub len_replace_on_concat {
    my ($a, $b) = @_;
    my $c = $a . $b;
    my $d = replace($c, "x", "yy");
    return length($d);
}

# --- length after replace on concat (deletion case) ---
# Replace all "x" with "" in a 64-char string of x's => length 0.
# sig: (Str, Str) -> Int
# pre: $a eq "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" && $b eq "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
# post: $result == 0
sub len_replace_delete_on_concat {
    my ($a, $b) = @_;
    my $c = $a . $b;
    my $d = replace($c, "x", "");
    return length($d);
}

# --- length of triple concat ---
# sig: (Str, Str, Str) -> Int
# pre: length($a) >= 0 && length($b) >= 0 && length($c) >= 0
# post: $result == length($a) + length($b) + length($c)
sub len_triple_concat {
    my ($a, $b, $c) = @_;
    return length($a . $b . $c);
}

# --- length after reverse of concat ---
# sig: (Str, Str) -> Int
# pre: length($a) >= 0 && length($b) >= 0
# post: $result == length($a) + length($b)
sub len_reverse_concat {
    my ($a, $b) = @_;
    my $c = $a . $b;
    return length(reverse($c));
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ a <- String(charset=>"a-z", length=>[0,20]), b <- String(charset=>"a-z", length=>[0,20]) ]##
    my $r = len_concat_sum($a, $b);
    $r == length($a) + length($b);
}, name => "len_concat_sum: post holds";

Property {
    ##[ a <- String(charset=>"a-z", length=>[1,20]), b <- String(charset=>"a-z", length=>[1,20]) ]##
    my $r = len_substr_of_concat($a, $b);
    $r >= 0 && $r <= length($a) + length($b);
}, name => "len_substr_of_concat: post holds";

Property {
    ##[ a <- String(charset=>"x", length=>[32,32]), b <- String(charset=>"x", length=>[32,32]) ]##
    my $r = len_replace_on_concat($a, $b);
    $r == 128;
}, name => "len_replace_on_concat: post holds";

Property {
    ##[ a <- String(charset=>"x", length=>[32,32]), b <- String(charset=>"x", length=>[32,32]) ]##
    my $r = len_replace_delete_on_concat($a, $b);
    $r == 0;
}, name => "len_replace_delete_on_concat: post holds";

Property {
    ##[ a <- String(charset=>"a-z", length=>[0,10]), b <- String(charset=>"a-z", length=>[0,10]), c <- String(charset=>"a-z", length=>[0,10]) ]##
    my $r = len_triple_concat($a, $b, $c);
    $r == length($a) + length($b) + length($c);
}, name => "len_triple_concat: post holds";

Property {
    ##[ a <- String(charset=>"a-z", length=>[0,20]), b <- String(charset=>"a-z", length=>[0,20]) ]##
    my $r = len_reverse_concat($a, $b);
    $r == length($a) + length($b);
}, name => "len_reverse_concat: post holds";
