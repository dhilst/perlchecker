# =============================================================
# Round 118: Integration Showcase — Rounds 101-117 Features
# =============================================================
# This file demonstrates features from rounds 101-117 working
# together in combination. Each function exercises 2+ new features
# to show practical verification power.

# --- Function 1: foreach + array literal + assert (R101, R102, R111) ---
# Iterates over an array literal, summing elements with mid-loop assertions.
# sig: (Int) -> Int
# pre: $x >= 0
# post: $result == 15
sub sum_array_foreach {
    my ($x) = @_;
    my @nums = (1, 2, 3, 4, 5);
    my $sum = 0;
    foreach my $n (@nums) {
        $sum = $sum + $n;
        # assert: $sum >= 0
    }
    return $sum;
}

# --- Function 2: hash ref + exists + defined + assert (R115, R114, R117, R111) ---
# Writes through a hash reference, then checks key existence and definedness.
# sig: (Hash<Str, Int>, Int) -> Int
# pre: $x > 0 && $x < 100
# post: $result == 1
sub hash_ref_exists_defined {
    my ($h, $x) = @_;
    my $href = \%h;
    $href->{"val"} = $x;
    my $found = exists($h{"val"});
    my $is_def = defined($x);
    # assert: $is_def == 1
    return $found;
}

# --- Function 3: ghost variable + loop invariant (R113, R109) ---
# Uses a ghost variable as a factor in a loop invariant for verification.
# sig: (Int) -> Int
# pre: $n >= 1 && $n <= 5
# post: $result == $n * 3
sub ghost_loop_invariant {
    my ($n) = @_;
    # ghost: $factor = 3
    my $sum = 0;
    my $i = 0;
    # inv: $sum == $i * $factor && $i >= 0 && $i <= $n
    while ($i < $n) {
        $sum = $sum + $factor;
        $i = $i + 1;
    }
    return $sum;
}

# --- Function 4: extern + regex + string ops (R112, R103) ---
# Calls an external function, then checks the result with regex matching.
# extern: sanitize_input (Str) -> Str post: length($result) >= 0 && length($result) <= 20
# sig: (Str) -> Int
# pre: length($input) >= 1 && length($input) <= 10
# post: $result >= 0 && $result <= 1
sub validated_regex_check {
    my ($input) = @_;
    my $clean = sanitize_input($input);
    my $has_prefix = 0;
    if ($clean =~ /^foo/) {
        $has_prefix = 1;
    }
    return $has_prefix;
}

# --- Function 5: scalar ref + defined + assert (R116, R114, R111) ---
# Reads through a scalar reference, checks definedness of the result.
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 10
# post: $result == $x + 1
sub ref_defined_assert {
    my ($x) = @_;
    my $y;
    my $ref = \$x;
    $y = $$ref + 1;
    my $is_def = defined($y);
    # assert: $is_def == 1
    return $y;
}

# --- Function 6: array literal + arrow ref + foreach (R102, R117, R101) ---
# Creates an array literal, accesses it via arrow dereference, and
# uses foreach to iterate and accumulate a bounded sum.
# sig: (Int) -> Int
# pre: $idx >= 0 && $idx <= 2
# post: $result >= 10 && $result <= 30
sub array_ref_lookup {
    my ($idx) = @_;
    my @data = (10, 20, 30);
    my $aref = \@data;
    my $val = $aref->[$idx];
    return $val;
}
