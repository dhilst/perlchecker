# Audit: nested foreach — inner loop must not clobber outer index
# sig: (Array<I64>, Array<I64>) -> I64
# pre: scalar(@a) == 2 && scalar(@b) == 2
# post: $result == $a[0] + $a[1] + $a[0] + $a[1]
sub nested_foreach_sum {
    my ($a, $b) = @_;
    my $sum = 0;
    foreach my $x (@a) {
        foreach my $y (@b) {
            $sum = $sum + $x;
        }
    }
    return $sum;
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ a0 <- Int(range=>[-5,5], sized=>0), a1 <- Int(range=>[-5,5], sized=>0), b0 <- Int(range=>[-5,5], sized=>0), b1 <- Int(range=>[-5,5], sized=>0) ]##
    my @a = ($a0, $a1);
    my @b = ($b0, $b1);
    my $result = nested_foreach_sum(\@a, \@b);
    $result == $a[0] + $a[1] + $a[0] + $a[1];
}, name => "nested_foreach_sum: post holds";
