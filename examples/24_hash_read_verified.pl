# sig: (Hash<Str, I64>, Str) -> I64
# post: $result == $h{$k}
sub hash_read_verified {
    my ($h, $k) = @_;
    return $h{$k};
}
