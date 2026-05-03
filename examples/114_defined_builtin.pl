# Round 114: defined() builtin with companion definedness tracking

# Test 1: Bare declaration leaves variable undefined
# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 0
sub check_undefined {
    my ($x) = @_;
    my $y;
    return defined($y);
}

# Test 2: Assignment makes variable defined
# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 1
sub check_defined_after_assign {
    my ($x) = @_;
    my $y;
    $y = $x + 1;
    return defined($y);
}

# Test 3: Function parameters are always defined
# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 1
sub check_param_defined {
    my ($x) = @_;
    return defined($x);
}

# Test 4: defined() in conditional logic
# sig: (Int) -> Int
# pre: $x >= 0
# post: $result >= 0
sub check_defined_branch {
    my ($x) = @_;
    my $y;
    if ($x > 5) {
        $y = $x;
    } else {
        $y = 0;
    }
    my $r = 0;
    if (defined($y) == 1) {
        $r = $y;
    }
    return $r;
}
