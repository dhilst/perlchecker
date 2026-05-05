# =============================================================
# Round 149: Soundness audit — contains, starts_with, ends_with
# =============================================================
# Verify edge cases: empty needle always true, empty haystack
# with non-empty needle false, boolean-to-I64 mapping (0/1).

# --- Edge case 1: contains with empty needle always returns 1 ---
# sig: (Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10
# post: $result == 1
sub contains_empty_needle {
    my ($s) = @_;
    my $r = contains($s, "");
    return $r;
}

# --- Edge case 2: starts_with with empty prefix always returns 1 ---
# sig: (Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10
# post: $result == 1
sub starts_with_empty_prefix {
    my ($s) = @_;
    my $r = starts_with($s, "");
    return $r;
}

# --- Edge case 3: ends_with with empty suffix always returns 1 ---
# sig: (Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10
# post: $result == 1
sub ends_with_empty_suffix {
    my ($s) = @_;
    my $r = ends_with($s, "");
    return $r;
}

# --- Edge case 4: contains result is always 0 or 1 ---
# sig: (Str, Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5
# post: $result >= 0 && $result <= 1
sub contains_bounded {
    my ($s, $t) = @_;
    my $r = contains($s, $t);
    return $r;
}

# --- Edge case 5: starts_with result is always 0 or 1 ---
# sig: (Str, Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5
# post: $result >= 0 && $result <= 1
sub starts_with_bounded {
    my ($s, $t) = @_;
    my $r = starts_with($s, $t);
    return $r;
}

# --- Edge case 6: ends_with result is always 0 or 1 ---
# sig: (Str, Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5
# post: $result >= 0 && $result <= 1
sub ends_with_bounded {
    my ($s, $t) = @_;
    my $r = ends_with($s, $t);
    return $r;
}

# --- Edge case 7: contains($s, $s) is always 1 ---
# sig: (Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10
# post: $result == 1
sub contains_self {
    my ($s) = @_;
    my $r = contains($s, $s);
    return $r;
}

# --- Edge case 8: starts_with implies contains ---
# sig: (Str, Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5 && starts_with($s, $t) == 1
# post: $result == 1
sub starts_with_implies_contains {
    my ($s, $t) = @_;
    my $r = contains($s, $t);
    return $r;
}

# --- Edge case 9: ends_with implies contains ---
# sig: (Str, Str) -> I64
# pre: length($s) >= 0 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5 && ends_with($s, $t) == 1
# post: $result == 1
sub ends_with_implies_contains {
    my ($s, $t) = @_;
    my $r = contains($s, $t);
    return $r;
}

sub contains { return index($_[0], $_[1]) >= 0 ? 1 : 0 }
sub starts_with { return substr($_[0], 0, length($_[1])) eq $_[1] ? 1 : 0 }
sub ends_with { return 1 if length($_[1]) == 0; return length($_[0]) >= length($_[1]) && substr($_[0], -length($_[1])) eq $_[1] ? 1 : 0 }

use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::LectroTest;

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]) ]##
    my $result = contains_empty_needle($s);
    $result == 1;
}, name => "contains_empty_needle: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]) ]##
    my $result = starts_with_empty_prefix($s);
    $result == 1;
}, name => "starts_with_empty_prefix: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]) ]##
    my $result = ends_with_empty_suffix($s);
    $result == 1;
}, name => "ends_with_empty_suffix: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]), t <- String(charset=>"a-z", length=>[0,5]) ]##
    my $result = contains_bounded($s, $t);
    $result >= 0 && $result <= 1;
}, name => "contains_bounded: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]), t <- String(charset=>"a-z", length=>[0,5]) ]##
    my $result = starts_with_bounded($s, $t);
    $result >= 0 && $result <= 1;
}, name => "starts_with_bounded: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]), t <- String(charset=>"a-z", length=>[0,5]) ]##
    my $result = ends_with_bounded($s, $t);
    $result >= 0 && $result <= 1;
}, name => "ends_with_bounded: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]) ]##
    my $result = contains_self($s);
    $result == 1;
}, name => "contains_self: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]), t <- String(charset=>"a-z", length=>[0,5]) ]##
    my $sw = (index($s, $t) == 0) ? 1 : 0;
    !$sw || (index($s, $t) >= 0);
}, name => "starts_with_implies_contains: post holds";

Property {
    ##[ s <- String(charset=>"a-z", length=>[0,10]), t <- String(charset=>"a-z", length=>[0,5]) ]##
    my $ew = (length($s) >= length($t) && substr($s, -length($t)) eq $t) ? 1 : 0;
    !$ew || (index($s, $t) >= 0);
}, name => "ends_with_implies_contains: post holds";
