# sig: (I64, I64, I64, I64, I64, I64, I64, I64, I64, I64, I64) -> I64
# post: $result >= 0
sub path_limit_exceeded {
    my ($a, $b, $c, $d, $e, $f, $g, $h, $i, $j, $k) = @_;
    my $x = 0;
    if ($a > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($b > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($c > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($d > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($e > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($f > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($g > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($h > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($i > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($j > 0) { $x = $x + 1; } else { $x = $x + 1; }
    if ($k > 0) { $x = $x + 1; } else { $x = $x + 1; }
    return $x;
}
