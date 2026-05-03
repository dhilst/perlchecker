# Round 28: chomp() builtin - removes trailing newline from string
# chomp($s) returns $s with trailing "\n" removed (if present)

# sig: (Str) -> Int
# pre: length($s) >= 2 && length($s) <= 10
# post: $result >= 1
sub chomp_not_empty {
    my ($s) = @_;
    my $r = chomp($s);
    my $len = length($r);
    return $len;
}

# sig: (Str) -> Str
# pre: length($s) >= 1 && length($s) <= 10
# post: length($result) >= length($s) - 1 && length($result) <= length($s)
sub chomp_length_bounds {
    my ($s) = @_;
    return chomp($s);
}
