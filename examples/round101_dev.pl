# Function 1: Sum positive array elements using foreach
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 5
# post: $result >= 0
sub sum_positive {
    my ($arr) = @_;
    my $sum = 0;
    foreach my $x (@arr) {
        if ($x > 0) {
            $sum = $sum + $x;
        }
    }
    return $sum;
}

# Function 2: Count elements matching a condition
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 4
# post: $result >= 0 && $result <= 4
sub count_positive {
    my ($arr) = @_;
    my $count = 0;
    foreach my $val (@arr) {
        if ($val > 0) {
            $count = $count + 1;
        }
    }
    return $count;
}

# Function 3: Find maximum using foreach
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 5
# post: $result >= -1000
sub find_max {
    my ($arr) = @_;
    my $best = -1000;
    foreach my $x (@arr) {
        if ($x > $best) {
            $best = $x;
        }
    }
    return $best;
}
