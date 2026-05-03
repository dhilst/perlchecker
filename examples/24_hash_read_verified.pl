# sig: (Hash<Str, Int>, Str) -> Int
# post: $result == $h{$k}
sub hash_read_verified {
    my ($h, $k) = @_;
    return $h{$k};
}
