# sig: (I64) -> I64
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (I64) -> I64
# post: $result == $x + 1
sub function_call_verified {
    my ($x) = @_;
    return inc($x);
}
