# Round 44: Loop control modifiers (next if, last if, next unless, last unless)

# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 0
sub sum_odd {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next if ($i % 2 == 0);
        $sum += $i;
    }
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 0 && $result <= 6
sub sum_until_three {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        $sum += $i;
        last if ($i >= 3);
    }
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result >= 0
sub sum_skip_zero {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        next unless ($i > 0);
        $sum += $i;
    }
    return $sum;
}

# sig: (Int) -> Int
# pre: $n >= 2 && $n <= 5
# post: $result >= 0 && $result <= 1
sub stop_unless_small {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i++) {
        $sum += $i;
        last unless ($sum < 1);
    }
    return $sum;
}
