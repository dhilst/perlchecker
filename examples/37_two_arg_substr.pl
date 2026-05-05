# sig: (Str, I64) -> Str
# pre: length($s) >= $n && $n >= 0
# post: length($result) == length($s) - $n
sub suffix {
    my ($s, $n) = @_;
    my $r = substr($s, $n);
    return $r;
}

# sig: (Str) -> Str
# pre: length($s) >= 0
# post: length($result) == length($s)
sub full_copy {
    my ($s) = @_;
    my $r = substr($s, 0);
    return $r;
}
