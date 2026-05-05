use strict;
use warnings;
use Test::More;

sub prop_ok {
    my ($name, $body) = @_;
    for my $t (1..1000) {
        unless ($body->($t)) {
            fail("$name (falsified at trial $t)");
            return;
        }
    }
    pass("$name (1000 trials)");
}

sub randi { int(rand($_[1] - $_[0] + 1)) + $_[0] }

# Return values
is((1 == 1), 1,   "cmp_num: true comparison returns 1");
is((1 == 2), "",  "cmp_num: false comparison returns '' (not 0)");

# == and !=
prop_ok("==: reflexive", sub {
    my $x = randi(-1000, 1000);
    $x == $x;
});

prop_ok("==: symmetric", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    ($x == $y) == ($y == $x);
});

prop_ok("!=: negation of ==", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    !($x == $y) == ($x != $y);
});

# <, <=, >, >=
prop_ok("<: irreflexive", sub {
    my $x = randi(-1000, 1000);
    !($x < $x);
});

prop_ok("<=: reflexive", sub {
    my $x = randi(-1000, 1000);
    $x <= $x;
});

prop_ok("</>: antisymmetric pair", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    ($x < $y) == ($y > $x);
});

prop_ok("<=/>=: antisymmetric pair", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    ($x <= $y) == ($y >= $x);
});

prop_ok("<: transitive", sub {
    my ($x, $y, $z) = (randi(-100, 100), randi(-100, 100), randi(-100, 100));
    !($x < $y && $y < $z) || ($x < $z);
});

prop_ok("cmp_num: trichotomy", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    ($x < $y) || ($x == $y) || ($x > $y);
});

# <=> (spaceship)
prop_ok("<=>: x <=> x == 0", sub {
    my $x = randi(-1000, 1000);
    ($x <=> $x) == 0;
});

is(1 <=> 2, -1,  "<=>: smaller <=> larger == -1");
is(2 <=> 1, 1,   "<=>: larger <=> smaller == 1");

prop_ok("<=>: antisymmetry (a<=>b == -(b<=>a))", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    ($x <=> $y) == -(($y <=> $x));
});

prop_ok("<=>: result is always -1, 0, or 1", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    my $r = $x <=> $y;
    $r == -1 || $r == 0 || $r == 1;
});

prop_ok("<=>: (x<=>y)==-1 iff x < y", sub {
    my ($x, $y) = (randi(-1000, 1000), randi(-1000, 1000));
    (($x <=> $y) == -1) == ($x < $y);
});

done_testing;
