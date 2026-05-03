# =============================================================
# Round 60: String builtin composition path stress
# =============================================================
# Exercises Z3 string theory by chaining multiple string
# operations (substr, contains, length, concat, replace,
# starts_with, ends_with) in sequence with conditional
# branching based on intermediate results. The verifier must
# reason about string transformations across divergent paths.

# --- Chain: length + contains + branch + concat/substr ---
# Checks if input contains a substring; if yes, concat a suffix,
# otherwise take a prefix via substr. Both paths produce a string
# whose length is bounded by the precondition.
# sig: (Str) -> Int
# pre: length($s) >= 4 && length($s) <= 10
# post: $result >= 4 && $result <= 13
sub length_contains_branch {
    my ($s) = @_;
    my $has_ab = contains($s, "ab");
    my $r;
    if ($has_ab == 1) {
        my $ext = $s . "xyz";
        $r = length($ext);
    } else {
        my $pre = substr($s, 0, 4);
        $r = length($pre);
    }
    return $r;
}

# --- Chain: starts_with + ends_with + replace + length ---
# Tests prefix/suffix and branches to apply different replace
# operations. Each path returns the length of the transformed
# string. Z3 must track string length through replace semantics.
# sig: (Str) -> Int
# pre: length($s) >= 5 && length($s) <= 8
# post: $result >= 5 && $result <= 10
sub prefix_suffix_replace {
    my ($s) = @_;
    my $sp = starts_with($s, "he");
    my $ep = ends_with($s, "lo");
    my $r;
    if ($sp == 1) {
        my $t = replace($s, "he", "she");
        $r = length($t);
    } elsif ($ep == 1) {
        my $t = replace($s, "lo", "low");
        $r = length($t);
    } else {
        $r = length($s);
    }
    return $r;
}

# --- Chain: substr + contains on substr result + branch ---
# Extracts a prefix substring, then checks if that substring
# contains a character. Branches on the contains result. Exercises
# nested string operation composition.
# sig: (Str) -> Int
# pre: length($s) >= 6 && length($s) <= 10
# post: $result >= 0 && $result <= 6
sub substr_then_contains {
    my ($s) = @_;
    my $pre = substr($s, 0, 4);
    my $has_x = contains($pre, "x");
    my $r;
    if ($has_x == 1) {
        my $idx = index($pre, "x");
        $r = $idx + 1;
    } else {
        my $full_has = contains($s, "x");
        $r = $full_has + length($pre) - 4;
    }
    return $r;
}

# --- Chain: concat + length arithmetic + multi-branch ---
# Builds two concatenated strings and branches on their length
# comparison. Each branch does further string ops. Creates 3
# paths with different string operation chains.
# sig: (Str, Str) -> Int
# pre: length($a) >= 2 && length($a) <= 4 && length($b) >= 2 && length($b) <= 4
# post: $result >= 2 && $result <= 8
sub concat_length_multi {
    my ($a, $b) = @_;
    my $ab = $a . $b;
    my $la = length($a);
    my $lb = length($b);
    my $r;
    if ($la > $lb) {
        $r = length($ab) - $lb;
    } elsif ($la < $lb) {
        $r = length($ab) - $la;
    } else {
        $r = length($ab) / 2;
    }
    return $r;
}

# --- Chain: replace + contains + starts_with pipeline ---
# Applies a replace, then checks if the result starts_with or
# contains something. Multi-step reasoning about how replace
# affects subsequent string predicates.
# sig: (Str) -> Int
# pre: length($s) >= 4 && length($s) <= 8 && starts_with($s, "ab") == 1
# post: $result >= 1 && $result <= 3
sub replace_then_check {
    my ($s) = @_;
    my $t = replace($s, "ab", "xy");
    my $sw = starts_with($t, "xy");
    my $has_z = contains($t, "z");
    my $r;
    if ($sw == 1 && $has_z == 1) {
        $r = 3;
    } elsif ($sw == 1) {
        $r = 2;
    } else {
        $r = 1;
    }
    return $r;
}
