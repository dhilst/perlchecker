# Soundness audit: Perl's $ anchor matches before an optional trailing \n.
# Previously, $s =~ /^hello$/ was desugared to ($s eq "hello"), missing
# the case where $s is "hello\n".  Fixed to also accept trailing newline.

# VERIFIED: result is bounded regardless of trailing-newline semantics.
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub anchored_match_bounded {
    my ($s) = @_;
    if ($s =~ /^hello$/) {
        return 1;
    }
    return 0;
}

# VERIFIED: unanchored contains is unaffected by the $ issue.
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub contains_unaffected {
    my ($s) = @_;
    if ($s =~ /hello/) {
        return 1;
    }
    return 0;
}

# VERIFIED: correctly accounts for trailing newline in postcondition.
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 5 && $result <= 6
sub anchored_len_correct_post {
    my ($s) = @_;
    if ($s =~ /^hello$/) {
        return length($s);
    }
    return 5;
}

# VERIFIED: dollar-only anchor with correct postcondition.
# sig: (Str) -> I64
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub dollar_anchor_bounded {
    my ($s) = @_;
    if ($s =~ /world$/) {
        return 1;
    }
    return 0;
}
