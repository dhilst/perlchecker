# Round 117: Arrow dereference for array and hash references

# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 42
sub arrow_array_write {
    my ($x) = @_;
    my @data = (1, 2, 3);
    my $aref = \@data;
    $aref->[0] = 42;
    return $data[0];
}

# sig: (I64) -> I64
# pre: $x >= 0
# post: $result == 2
sub arrow_array_read {
    my ($x) = @_;
    my @data = (10, 20, 30);
    my $aref = \@data;
    my $val = $aref->[1];
    return $val / 10;
}

# sig: (Hash<Str, I64>, Str) -> I64
# pre: $key eq "name"
# post: $result == 1
sub arrow_hash_write {
    my ($h, $key) = @_;
    my $href = \%h;
    $href->{"name"} = 99;
    return exists($h{"name"});
}
