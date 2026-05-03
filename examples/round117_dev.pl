# Round 117: Arrow dereference for array and hash references

# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 42
sub arrow_array_write {
    my ($x) = @_;
    my @data = (1, 2, 3);
    my $aref = \@data;
    $aref->[0] = 42;
    return $data[0];
}

# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 2
sub arrow_array_read {
    my ($x) = @_;
    my @data = (10, 20, 30);
    my $aref = \@data;
    my $val = $aref->[1];
    return $val / 10;
}

# sig: (Hash<Str, Int>, Str) -> Int
# pre: $key eq "name"
# post: $result == 1
sub arrow_hash_write {
    my ($h, $key) = @_;
    my $href = \%h;
    $href->{"name"} = 99;
    return exists($h{"name"});
}
