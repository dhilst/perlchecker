# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub reverse_preserves_length {
    my ($s) = @_;
    my $r = reverse($s);
    if (length($r) == length($s)) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result >= 0 && $result <= 1
sub double_reverse_identity {
    my ($s) = @_;
    my $r = reverse(reverse($s));
    if ($r eq $s) {
        return 1;
    }
    return 0;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,5]) ]##
    my $result = reverse_preserves_length($s);
    $result == 1;
}, name => "reverse_preserves_length: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,5]) ]##
    my $result = double_reverse_identity($s);
    $result >= 0 && $result <= 1;
}, name => "double_reverse_identity: post holds";
