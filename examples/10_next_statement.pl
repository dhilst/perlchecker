# sig: (I64) -> I64
# pre: $n >= 1 && $n <= 3
# post: $result >= 0
sub sum_even_indices {
    my ($n) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $n; $i += 1) {
        if ($i % 2 != 0) {
            next;
        }
        $sum += $i;
    }
    return $sum;
}
