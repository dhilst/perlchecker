# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 1
sub function_call_verified {
    my ($x) = @_;
    return inc($x);
}
