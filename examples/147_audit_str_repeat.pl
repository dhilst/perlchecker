# sig: (Str) -> Str
# pre: length($s) >= 1 && length($s) <= 5
# post: $result eq ""
sub repeat_zero {
    my ($s) = @_;
    return $s x 0;
}

# sig: (Str) -> Str
# pre: length($s) >= 1 && length($s) <= 5
# post: $result eq ""
sub repeat_neg {
    my ($s) = @_;
    return $s x -3;
}

# sig: (Str) -> Str
# pre: length($s) >= 1 && length($s) <= 5
# post: $result eq ""
sub repeat_neg_large {
    my ($s) = @_;
    return $s x -100;
}

# sig: (Str) -> I64
# pre: length($s) == 2
# post: $result == 6
sub repeat_three_len {
    my ($s) = @_;
    my $r = $s x 3;
    return length($r);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,5]) ]##
    ("$s" x 0) eq "";
}, name => "repeat_zero: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,5]) ]##
    ("$s" x -3) eq "";
}, name => "repeat_neg: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[1,5]) ]##
    ("$s" x -100) eq "";
}, name => "repeat_neg_large: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[2,2]) ]##
    my $r = $s x 3;
    length($r) == 6;
}, name => "repeat_three_len: post holds";
