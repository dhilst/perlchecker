# Round 186 audit part G: IntToStr with division and modulo results
# ==================================================================

# In Perl, int(7/2) = 3 (truncation). "" . int(7/2) = "3".
# sig: (I64, I64) -> Str
# pre: $a == 7 && $b == 2
# post: $result eq "3"
sub div_to_str {
    my ($a, $b) = @_;
    my $q = int($a / $b);
    return "" . $q;
}

# Negative division: int(-7/2) = -3 in Perl (towards zero).
# sig: (I64, I64) -> Str
# pre: $a == -7 && $b == 2
# post: $result eq "-3"
sub neg_div_to_str {
    my ($a, $b) = @_;
    my $q = int($a / $b);
    return "" . $q;
}

# Perl's modulo uses floored division: (-7) % 3 = 2 (not -1).
# sig: (I64, I64) -> Str
# pre: $a == -7 && $b == 3
# post: $result eq "2"
sub neg_mod_to_str_correct {
    my ($a, $b) = @_;
    my $r = $a % $b;
    return "" . $r;
}

# Zero result from modulo: 6 % 3 = 0. "" . 0 = "0".
# sig: (I64, I64) -> Str
# pre: $a == 6 && $b == 3
# post: $result eq "0"
sub zero_mod_to_str {
    my ($a, $b) = @_;
    my $r = $a % $b;
    return "" . $r;
}
