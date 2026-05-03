# sig: (Str, Int) -> Int
# pre: length($s) >= 5 && $i >= 0 && $i < length($s)
# post: $result == 1
sub char_at_length_one {
    my ($s, $i) = @_;
    my $c = char_at($s, $i);
    my $len = length($c);
    return $len;
}

# Complex path exploration: multiple branches based on character comparisons
# sig: (Str) -> Int
# pre: length($s) == 3
# post: $result >= 0 && $result <= 3
sub count_a_chars {
    my ($s) = @_;
    my $count = 0;
    if (char_at($s, 0) eq "a") {
        $count += 1;
    }
    if (char_at($s, 1) eq "a") {
        $count += 1;
    }
    if (char_at($s, 2) eq "a") {
        $count += 1;
    }
    return $count;
}
