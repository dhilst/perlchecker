# Round 186 audit part D: IntToStr substr and character extraction
# ================================================================
# Probe whether Z3's int.to.str allows correct character extraction
# via substr from stringified integers.

# --- PROBE: First character of stringified negative is "-" ---
# sig: (I64) -> Str
# pre: $n >= -99 && $n <= -1
# post: $result eq "-"
sub neg_first_char {
    my ($n) = @_;
    my $s = "" . $n;
    return substr($s, 0, 1);
}

# --- PROBE: First character of stringified zero is "0" ---
# sig: (I64) -> Str
# pre: $n == 0
# post: $result eq "0"
sub zero_first_char {
    my ($n) = @_;
    my $s = "" . $n;
    return substr($s, 0, 1);
}

# --- PROBE: Stringified single digit is the digit itself ---
# For $n in [0..9], "" . $n should be exactly one character
# and substr should return that character.
# sig: (I64) -> I64
# pre: $n >= 0 && $n <= 9
# post: $result == 1
sub single_digit_substr_len {
    my ($n) = @_;
    my $s = "" . $n;
    my $c = substr($s, 0, 1);
    return length($c);
}

# --- PROBE (FALSE): Is "" . 10 equal to "" . 1 . "" . 0? ---
# This is TRUE in Perl: "10" eq "1" . "0" => "10"
# sig: (I64) -> I64
# pre: $n == 10
# post: $result == 1
sub concat_digits {
    my ($n) = @_;
    my $s = "" . $n;
    my $s2 = ("" . 1) . ("" . 0);
    if ($s eq $s2) { return 1; }
    return 0;
}

# --- PROBE: String comparison of stringified ints ---
# In Perl, "2" lt "10" is FALSE because string "2" > "1" (first char)
# sig: (I64, I64) -> I64
# pre: $a == 2 && $b == 10
# post: $result == 0
sub string_cmp_2_vs_10 {
    my ($a, $b) = @_;
    my $sa = "" . $a;
    my $sb = "" . $b;
    if ($sa lt $sb) { return 1; }
    return 0;
}
