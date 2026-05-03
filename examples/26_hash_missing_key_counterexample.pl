# sig: (Hash<Str, Int>, Str) -> Int
# post: $result == 0
sub hash_missing_key_counterexample {
    my ($h, $k) = @_;
    return $h{$k};
}
