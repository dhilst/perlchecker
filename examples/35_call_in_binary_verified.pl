# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 2
sub call_in_binary {
    my ($x) = @_;
    my $z = inc($x) + 1;
    return $z;
}
