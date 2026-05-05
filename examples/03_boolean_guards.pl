# sig: (I64, I64) -> I64
# pre: !($x == 0) || $y >= 0
# post: ($x <= $y && $result == $y - $x) || ($x > $y && $result == $x - $y)
sub boolean_guards {
    my ($x, $y) = @_;
    if ($x <= $y && !($x == $y)) {
        return $y - $x;
    } else {
        if ($x != $y) {
            return $x - $y;
        } else {
            return 0;
        }
    }
}
