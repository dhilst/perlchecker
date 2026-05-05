# Audit round 13: fix index() with empty needle beyond string length
# In Perl, index($s, "", $pos) returns min($pos, length($s)).
# Previously Z3's str.indexof returned -1 when pos > length(s),
# which was unsound. Now fixed: the encoding guards empty needles.

# sig: (Str, I64) -> I64
# pre: length($s) >= 1 && $pos >= 0
# post: $result >= 0 && $result <= length($s)
sub index_empty_needle {
    my ($s, $pos) = @_;
    return index($s, "", $pos);
}

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest::Generator qw(:common);
use Test::More tests => 1;
sub check_prop { my ($name,$g,$c,$n)=@_; $n//=1000; for(1..$n){my @a=$g->($_); $c->(@a)||do{diag("FAIL $name: @a");return 0}} 1 }
my $gen_s = String(charset=>"a-z", length=>[1,10]);
my $gen_p = Int(range=>[0,50], sized=>0);
ok(check_prop("index_empty_needle", sub { ($gen_s->generate($_[0]), $gen_p->generate($_[0])) }, sub { my $r = index_empty_needle($_[0], $_[1]); $r >= 0 && $r <= length($_[0]) }), "index_empty_needle: post holds");
