# sig: (Str) -> I64
# pre: length($s) >= 1
# post: $result >= 0
sub single_quote_concat {
    my ($s) = @_;
    my $prefix = 'hello_';
    my $r = length($prefix . $s);
    return $r;
}

# sig: (I64) -> I64
# pre: $x >= 0 && $x <= 10
# post: $result == 1
sub sq_compare {
    my ($x) = @_;
    my $a = 'test';
    my $b = 'test';
    my $r = ($a eq $b) ? 1 : 0;
    return $r;
}
