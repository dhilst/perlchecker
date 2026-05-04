# Round 164: Soundness audit — die/warn path handling
#
# Bug found: the Z3 string-length bound (MAX_STR_LEN=32) was asserted
# unconditionally, overriding user-supplied preconditions that allow
# longer strings. This made `die if (length($s) > N)` for N > 32
# appear unreachable when the precondition allowed length > 32,
# yielding a false "verified" verdict.
#
# Fix: extract explicit length upper-bounds from the path condition
# and use max(user_bound, MAX_STR_LEN) per variable.

# --- Case 1: die unreachable when precondition keeps string short ---
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result == length($s)
sub die_short_string_unreachable {
    my ($s) = @_;
    die "empty" if (length($s) == 0);
    return length($s);
}

# --- Case 2: warn is a no-op (execution continues) ---
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 10
# post: $result == $x + 1
sub warn_no_effect {
    my ($x) = @_;
    warn "debug info";
    return $x + 1;
}

# --- Case 3: die unless guard with precondition ---
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 10
# post: $result == $x * 2
sub die_unless_guard {
    my ($x) = @_;
    die "non-positive" unless ($x > 0);
    return $x * 2;
}

# --- Case 4: die guard for long strings, precondition excludes die path ---
# (Before the fix, this would have falsely verified even with a broad
#  precondition because MAX_STR_LEN=32 made length>50 unsatisfiable)
# sig: (Str) -> Int
# pre: length($s) >= 0 && length($s) <= 50
# post: $result == length($s)
sub die_long_guard_safe {
    my ($s) = @_;
    die "too long" if (length($s) > 50);
    return length($s);
}
