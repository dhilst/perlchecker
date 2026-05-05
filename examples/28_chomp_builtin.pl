# Round 28: chomp() builtin - returns number of characters removed (0 or 1)
# In Perl, chomp($s) modifies $s in-place removing trailing newline,
# and returns the number of characters removed.

# sig: (Str) -> I64
# pre: length($s) >= 2 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub chomp_returns_count {
    my ($s) = @_;
    my $r = chomp($s);
    return $r;
}

# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub chomp_bounded {
    my ($s) = @_;
    return chomp($s);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ s <- String(charset=>"a-z", length=>[2,10]) ]##
    my $result = chomp_returns_count($s);
    $result >= 0 && $result <= 1;
}, name => "chomp_returns_count: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,10]) ]##
    my $result = chomp_bounded($s);
    $result >= 0 && $result <= 1;
}, name => "chomp_bounded: post holds";
