# sig: (Str) -> Str
# pre: length($s) >= 3 && length($s) <= 10
# post: length($result) == 2
sub last_two {
    my ($s) = @_;
    return substr($s, -2, 2);
}

# Counterexample: substr($s, -2, 2) != substr($s, 0, 2) in general
# sig: (Str) -> Str
# pre: length($s) >= 5 && length($s) <= 10
# post: $result eq substr($s, 0, 2)
sub last_two_wrong {
    my ($s) = @_;
    return substr($s, -2, 2);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;
Property {
    ##[ s <- String(charset=>"a-z", length=>[3,10]) ]##
    length(last_two($s)) == 2;
}, name => "last_two: post holds";
