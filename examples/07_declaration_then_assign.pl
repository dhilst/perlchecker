# sig: (Int) -> Int
# post: $result == $x + 1
sub declaration_then_assign {
    my ($x) = @_;
    my $y;
    $y = $x + 1;
    return $y;
}
