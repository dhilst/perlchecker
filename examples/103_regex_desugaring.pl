# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub starts_with_hello {
    my ($s) = @_;
    if ($s =~ /^hello/) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub contains_test {
    my ($s) = @_;
    if ($s =~ /test/) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub not_ending_with_x {
    my ($s) = @_;
    if ($s !~ /x$/) {
        return 1;
    }
    return 0;
}
