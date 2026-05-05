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

# if / elsif / else
{
    my $r;
    if (1) { $r = "a" } else { $r = "b" }
    is($r, "a", "if: true condition takes then-branch");
}
{
    my $r;
    if (0) { $r = "a" } else { $r = "b" }
    is($r, "b", "if: false condition takes else-branch");
}
{
    my $r;
    if (0) { $r = 1 }
    elsif (1) { $r = 2 }
    else { $r = 3 }
    is($r, 2, "elsif: first true elsif branch taken");
}
{
    my $r;
    if (0) { $r = 1 }
    elsif (0) { $r = 2 }
    else { $r = 3 }
    is($r, 3, "elsif: all false falls through to else");
}
{
    my $r;
    if (1) { $r = 1 }
    elsif (1) { $r = 2 }
    else { $r = 3 }
    is($r, 1, "elsif: first true branch wins, later not evaluated");
}

# unless
{
    my $r = 0;
    unless (0) { $r = 1 }
    is($r, 1, "unless: executes when condition is false");
}
{
    my $r = 0;
    unless (1) { $r = 1 }
    is($r, 0, "unless: skips when condition is true");
}

prop_ok("unless: equivalent to if(!condition)", sub {
    my $x = randi(0, 1);
    my ($a, $b) = (0, 0);
    if (!$x) { $a = 1 }
    unless ($x) { $b = 1 }
    $a == $b;
});

# while
{
    my $i = 0;
    my $sum = 0;
    while ($i < 5) { $sum += $i; $i++ }
    is($sum, 10, "while: loops until condition false (0+1+2+3+4=10)");
}
{
    my $count = 0;
    while (0) { $count++ }
    is($count, 0, "while: false condition means zero iterations");
}

# until
{
    my $i = 0;
    until ($i >= 5) { $i++ }
    is($i, 5, "until: loops until condition becomes true");
}
{
    my $count = 0;
    until (1) { $count++ }
    is($count, 0, "until: true condition means zero iterations");
}

# do-while / do-until
{
    my $count = 0;
    do { $count++ } while (0);
    is($count, 1, "do-while: body executes at least once even if condition false");
}
{
    my $count = 0;
    do { $count++ } until (1);
    is($count, 1, "do-until: body executes at least once even if condition true");
}
{
    my $i = 0;
    do { $i++ } while ($i < 3);
    is($i, 3, "do-while: loops until condition false");
}

# for (C-style)
{
    my $sum = 0;
    for (my $i = 0; $i < 5; $i++) { $sum += $i }
    is($sum, 10, "for: C-style loop sums 0..4 = 10");
}
{
    my $count = 0;
    for (my $i = 0; $i < 0; $i++) { $count++ }
    is($count, 0, "for: zero iterations when condition initially false");
}
{
    my $count = 0;
    for (my $i = 10; $i > 0; $i -= 3) { $count++ }
    is($count, 4, "for: countdown with step (10,7,4,1 then stops)");
}

# foreach
{
    my @a = (10, 20, 30);
    my $sum = 0;
    foreach my $x (@a) { $sum += $x }
    is($sum, 60, "foreach: iterates over all elements in order");
}
{
    my @a = ();
    my $count = 0;
    foreach my $x (@a) { $count++ }
    is($count, 0, "foreach: empty array means zero iterations");
}
{
    my @a = (1, 2, 3);
    my @collected;
    foreach my $x (@a) { push @collected, $x }
    is_deeply(\@collected, [1, 2, 3], "foreach: preserves element order");
}

# last (break)
{
    my $count = 0;
    while (1) {
        $count++;
        last if ($count >= 3);
    }
    is($count, 3, "last: breaks out of loop");
}
{
    my @a = (1, 2, 3, 4, 5);
    my $sum = 0;
    foreach my $x (@a) {
        last if ($x > 3);
        $sum += $x;
    }
    is($sum, 6, "last: exits foreach early (1+2+3=6)");
}

# next (continue)
{
    my $sum = 0;
    for (my $i = 0; $i < 5; $i++) {
        next if ($i % 2 == 0);
        $sum += $i;
    }
    is($sum, 4, "next: skips even numbers (1+3=4)");
}
{
    my @a = (1, 2, 3, 4, 5);
    my $sum = 0;
    foreach my $x (@a) {
        next if ($x == 3);
        $sum += $x;
    }
    is($sum, 12, "next: skips element 3 in foreach (1+2+4+5=12)");
}

# Statement modifiers
{
    my $x = 0;
    $x = 1 if (1);
    is($x, 1, "stmt_mod: assignment if true");
}
{
    my $x = 0;
    $x = 1 if (0);
    is($x, 0, "stmt_mod: assignment if false — no change");
}
{
    my $x = 0;
    $x = 1 unless (0);
    is($x, 1, "stmt_mod: assignment unless false");
}
{
    my $x = 0;
    $x = 1 unless (1);
    is($x, 0, "stmt_mod: assignment unless true — no change");
}

# Nested loops with last/next
{
    my $inner_done = 0;
    my $outer_done = 0;
    for (my $i = 0; $i < 3; $i++) {
        for (my $j = 0; $j < 3; $j++) {
            last if ($j == 1);
            $inner_done++;
        }
        $outer_done++;
    }
    is($inner_done, 3, "last: in nested loop, only breaks inner (3 outer * 1 inner)");
    is($outer_done, 3, "last: outer loop unaffected by inner last");
}
{
    my $count = 0;
    for (my $i = 0; $i < 3; $i++) {
        for (my $j = 0; $j < 3; $j++) {
            next if ($j == 1);
            $count++;
        }
    }
    is($count, 6, "next: in nested loop, only skips in inner (3*2=6)");
}

# last/next unless
{
    my $count = 0;
    for (my $i = 0; $i < 5; $i++) {
        last unless ($i < 3);
        $count++;
    }
    is($count, 3, "last unless: breaks when condition becomes false");
}
{
    my $sum = 0;
    for (my $i = 0; $i < 5; $i++) {
        next unless ($i % 2 == 1);
        $sum += $i;
    }
    is($sum, 4, "next unless: skips when condition is false (1+3=4)");
}

# for-loop variable scoping
{
    my $outer_i = 99;
    for (my $i = 0; $i < 3; $i++) { }
    is($outer_i, 99, "for: loop variable does not leak to outer scope");
}

# Statement modifier: die if/unless
{
    eval { die "boom" if (1) };
    like($@, qr/boom/, "stmt_mod: die if true — dies");
}
{
    eval { die "boom" if (0) };
    is($@, "", "stmt_mod: die if false — does not die");
}
{
    eval { die "boom" unless (0) };
    like($@, qr/boom/, "stmt_mod: die unless false — dies");
}

# Multiple elsif branches
{
    my $r;
    my $x = 4;
    if ($x == 1) { $r = "a" }
    elsif ($x == 2) { $r = "b" }
    elsif ($x == 3) { $r = "c" }
    elsif ($x == 4) { $r = "d" }
    else { $r = "e" }
    is($r, "d", "elsif: 4th branch selected with 3+ elsif");
}

done_testing;
