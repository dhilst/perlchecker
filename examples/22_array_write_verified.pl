# sig: (Array<I64>, I64, I64) -> I64
# pre: $i >= 0
# post: $result == $v
sub array_write_verified {
    my ($arr, $i, $v) = @_;
    $arr[$i] = $v;
    return $arr[$i];
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ i <- Int(range=>[0,4], sized=>0), v <- Int(range=>[-100,100], sized=>0) ]##
    our @arr = (10, 20, 30, 40, 50);
    my $result = array_write_verified(0, $i, $v);
    $result == $v;
}, name => "array_write_verified: post holds with package \@arr";
