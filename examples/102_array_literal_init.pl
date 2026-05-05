# sig: (I64) -> I64
# pre: $x >= 1 && $x <= 3
# post: $result >= 10 && $result <= 30
sub lookup_table {
    my ($x) = @_;
    my @table = (10, 20, 30);
    my $idx = $x - 1;
    return $table[$idx];
}

# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 3
# post: $result >= 0
sub sum_first_n {
    my ($n) = @_;
    my @vals = (5, 10, 15, 20);
    my $sum = 0;
    my $i = 0;
    for ($i = 0; $i < $n; $i = $i + 1) {
        $sum = $sum + $vals[$i];
    }
    return $sum;
}
