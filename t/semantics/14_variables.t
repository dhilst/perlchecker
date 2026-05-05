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

# my — lexical scoping
{
    my $x = 42;
    is($x, 42, "my: declaration with initialization");
}
{
    my $x = 1;
    {
        my $x = 2;
        is($x, 2, "my: inner scope shadows outer");
    }
    is($x, 1, "my: outer preserved after inner scope ends");
}
{
    my $x = 10;
    {
        $x = 20;
    }
    is($x, 20, "my: assignment without my modifies enclosing scope");
}

# Compound assignment
prop_ok("compound: += equivalent to x = x + y", sub {
    my ($x, $y) = (randi(-100, 100), randi(-100, 100));
    my $a = $x; $a += $y;
    $a == ($x + $y);
});

prop_ok("compound: -= equivalent to x = x - y", sub {
    my ($x, $y) = (randi(-100, 100), randi(-100, 100));
    my $a = $x; $a -= $y;
    $a == ($x - $y);
});

prop_ok("compound: *= equivalent to x = x * y", sub {
    my ($x, $y) = (randi(-100, 100), randi(-100, 100));
    my $a = $x; $a *= $y;
    $a == ($x * $y);
});

prop_ok("compound: /= equivalent to x = x / y", sub {
    my ($x, $y) = (randi(1, 100), randi(1, 100));
    my $a = $x; $a /= $y;
    $a == ($x / $y);
});

prop_ok("compound: %= equivalent to x = x % y", sub {
    my ($x, $y) = (randi(0, 100), randi(1, 100));
    my $a = $x; $a %= $y;
    $a == ($x % $y);
});

prop_ok("compound: **= equivalent to x = x ** y", sub {
    my ($x, $y) = (randi(1, 5), randi(0, 3));
    my $a = $x; $a **= $y;
    $a == ($x ** $y);
});

{
    my $s = "hello"; $s .= " world";
    is($s, "hello world", "compound: .= appends string");
}

prop_ok("compound: <<= equivalent to x = x << y", sub {
    my $x = randi(0, 100);
    my $a = $x; $a <<= 2;
    $a == ($x << 2);
});

prop_ok("compound: >>= equivalent to x = x >> y", sub {
    my $x = randi(0, 1000);
    my $a = $x; $a >>= 2;
    $a == ($x >> 2);
});

prop_ok("compound: &= equivalent to x = x & y", sub {
    my ($x, $y) = (randi(0, 255), randi(0, 255));
    my $a = $x; $a &= $y;
    $a == ($x & $y);
});

prop_ok("compound: |= equivalent to x = x | y", sub {
    my ($x, $y) = (randi(0, 255), randi(0, 255));
    my $a = $x; $a |= $y;
    $a == ($x | $y);
});

prop_ok("compound: ^= equivalent to x = x ^ y", sub {
    my ($x, $y) = (randi(0, 255), randi(0, 255));
    my $a = $x; $a ^= $y;
    $a == ($x ^ $y);
});

# ++ / -- (post and pre)
prop_ok("++: post-increment returns old value, variable incremented", sub {
    my $x = randi(-100, 100);
    my $a = $x;
    my $old = $a++;
    $old == $x && $a == ($x + 1);
});

prop_ok("--: post-decrement returns old value, variable decremented", sub {
    my $x = randi(-100, 100);
    my $a = $x;
    my $old = $a--;
    $old == $x && $a == ($x - 1);
});

prop_ok("++: pre-increment returns new value", sub {
    my $x = randi(-100, 100);
    my $a = $x;
    my $new = ++$a;
    $new == ($x + 1) && $a == ($x + 1);
});

prop_ok("--: pre-decrement returns new value", sub {
    my $x = randi(-100, 100);
    my $a = $x;
    my $new = --$a;
    $new == ($x - 1) && $a == ($x - 1);
});

done_testing;
