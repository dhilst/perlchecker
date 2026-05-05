# Round 180: Audit — hash write in one branch, then exists after merge
#
# After "$h{$k} = 1" in only the then-branch of an if, check that
# exists($h{$k}) still works correctly on BOTH paths.
# In Perl: exists($h{$k}) returns 1 on the then-path, but is
# unconstrained on the else-path (key may or may not exist).
# The postcondition $result >= 0 should verify if the tool is sound.

# sig: (Hash<Str, I64>, Str, I64) -> I64
# pre: length($k) >= 1
# post: $result >= 0
sub hash_write_then_exists {
    my ($h, $k, $flag) = @_;
    if ($flag > 0) {
        $h{$k} = 42;
    }
    my $r = exists($h{$k});
    return $r;
}
