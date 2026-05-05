# Round 143 audit: chomp() side-effect was not modeled (unsound)
# In Perl, chomp($s) removes the trailing newline from $s IN PLACE.
# The checker was only returning the count but not updating the variable.

# sig: (Str) -> I64
# pre: $s eq "hello\n"
# post: $result == 5
sub chomp_shortens_string {
    my ($s) = @_;
    my $n = chomp($s);
    return length($s);
}

# sig: (Str) -> I64
# pre: $s eq "world\n"
# post: $result == 1
sub chomp_returns_count {
    my ($s) = @_;
    my $n = chomp($s);
    return $n;
}

# sig: (Str) -> I64
# pre: $s eq "no_newline"
# post: $result == 10
sub chomp_no_newline_unchanged {
    my ($s) = @_;
    my $n = chomp($s);
    return length($s);
}

# sig: (Str) -> Str
# pre: $s eq "abc\n"
# post: $result eq "abc"
sub chomp_value_check {
    my ($s) = @_;
    my $n = chomp($s);
    return $s;
}
