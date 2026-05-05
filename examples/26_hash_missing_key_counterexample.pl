# sig: (Hash<Str, I64>, Str) -> I64
# post: $result == 0
sub hash_missing_key_counterexample {
    my ($h, $k) = @_;
    return $h{$k};
}
