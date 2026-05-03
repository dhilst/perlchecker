# sig: (Hash<Str, Str>, Str, Str) -> Str
# post: $result eq $v
sub hash_write_verified {
    my ($h, $k, $v) = @_;
    $h{$k} = $v;
    return $h{$k};
}
