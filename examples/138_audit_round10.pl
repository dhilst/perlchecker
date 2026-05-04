# Round 138: Soundness audit — replace() first-occurrence vs global
#
# The checker encodes replace(s, from, to) as Z3's str.replace, which
# replaces only the FIRST occurrence.  But the Perl implementation
# (s/\Q$o\E/$n/g) replaces ALL occurrences.
#
# Exploit: when we know the input contains two copies of the pattern,
# the checker (first-replace only) believes one copy survives, but
# Perl (global replace) removes both.

# sig: (Str) -> Str
# pre: starts_with($s, "abab") == 1
# post: $result ne "XX"
sub replace_not_xx {
    my ($s) = @_;
    return replace($s, "ab", "X");
}

sub replace  { my ($s, $o, $n) = @_; $s =~ s/\Q$o\E/$n/g; $s }

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 1;

# Concrete test: $s = "abab"
# Perl: replace("abab","ab","X") => "XX"
# Postcondition says result ne "XX", but Perl gives "XX" => FALSE
my $s = "abab";
my $result = replace_not_xx($s);
ok($result eq "XX", "replace('abab','ab','X') is 'XX' in Perl — postcondition is false");
