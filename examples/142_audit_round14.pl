# Audit Round 14: replace() with 3+ occurrences was unsound (now fixed)
# The checker previously only iterated str.replace 2 times, so 3+ occurrences
# were not fully replaced. This caused unsound "verified" results for properties
# that depended on complete global replacement.

# sig: (Str) -> Str
# pre: $s eq "aaa"
# post: $result eq "bbb"
sub replace_all_three {
    my ($s) = @_;
    my $r = replace($s, "a", "b");
    return $r;
}

# sig: (Str) -> Str
# pre: $s eq "abcabc"
# post: $result eq "xbcxbc"
sub replace_multiple {
    my ($s) = @_;
    my $r = replace($s, "a", "x");
    return $r;
}

# sig: (Str) -> Int
# pre: $s eq "aaaa"
# post: $result == 0
sub replace_removes_all_a {
    my ($s) = @_;
    my $r = replace($s, "a", "b");
    return contains($r, "a");
}

sub replace { my ($s, $o, $n) = @_; $s =~ s/\Q$o\E/$n/g; $s }
sub contains { return index($_[0], $_[1]) >= 0 ? 1 : 0 }

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 3;

sub check_prop { my ($name,$g,$c,$n)=@_; $n//=1000; for(1..$n){my @a=$g->($_); $c->(@a)||do{diag("FAIL $name: @a");return 0}} 1 }

ok(check_prop("replace_all_three",
    sub { "aaa" },
    sub { replace_all_three($_[0]) eq "bbb" }
), "replace_all_three: aaa -> bbb");

ok(check_prop("replace_multiple",
    sub { "abcabc" },
    sub { replace_multiple($_[0]) eq "xbcxbc" }
), "replace_multiple: abcabc -> xbcxbc");

ok(check_prop("replace_removes_all_a",
    sub { "aaaa" },
    sub { replace_removes_all_a($_[0]) == 0 }
), "replace_removes_all_a: no a remaining after replace");
