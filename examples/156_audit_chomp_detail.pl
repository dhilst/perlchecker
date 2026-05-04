# Round 156: Audit chomp() return value and side-effect interaction
# Verifies:
# 1. chomp() returns 1 when newline present, 0 when not
# 2. Side-effect correctly removes trailing \n
# 3. Multiple trailing newlines: only ONE is removed

# sig: (Str) -> Int
# pre: $s eq "hello\n"
# post: $result == 1
sub chomp_returns_one_when_newline {
    my ($s) = @_;
    my $r = chomp($s);
    return $r;
}

# sig: (Str) -> Int
# pre: $s eq "hello"
# post: $result == 0
sub chomp_returns_zero_when_no_newline {
    my ($s) = @_;
    my $r = chomp($s);
    return $r;
}

# sig: (Str) -> Str
# pre: $s eq "hello\n"
# post: $result eq "hello"
sub chomp_side_effect_removes_newline {
    my ($s) = @_;
    my $r = chomp($s);
    return $s;
}

# sig: (Str) -> Str
# pre: $s eq "hello\n\n"
# post: $result eq "hello\n"
sub chomp_only_removes_one_newline {
    my ($s) = @_;
    my $r = chomp($s);
    return $s;
}

# sig: (Str) -> Str
# pre: $s eq "hello"
# post: $result eq "hello"
sub chomp_no_newline_no_change {
    my ($s) = @_;
    my $r = chomp($s);
    return $s;
}

# Symbolic test: chomp return value matches whether string changed
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result >= 0 && $result <= 1
sub chomp_return_bounded_symbolic {
    my ($s) = @_;
    my $r = chomp($s);
    return $r;
}

# Symbolic: after chomp, length decreased by exactly the return value
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 0
sub chomp_length_consistency {
    my ($s) = @_;
    my $orig_len = length($s);
    my $r = chomp($s);
    my $new_len = length($s);
    return $orig_len - $new_len - $r;
}
