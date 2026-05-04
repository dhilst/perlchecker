# =============================================================
# Round 161: Soundness audit -- string eq/ne with constants > MAX_STR_LEN
# =============================================================
# Bug: assert_string_bounds() bounded all string variables to MAX_STR_LEN
# (32 chars) WITHOUT considering string constants in the formula.  When a
# function body or postcondition contained a constant longer than 32 chars,
# the solver could never construct a variable value matching that constant,
# causing spurious "verified" results.
#
# Fix: compute the maximum string constant length in the formula and use
# it as a lower bound for the variable length constraint.

# --- Exploit: a 33-char constant made ne trivially true ---
# Before the fix, the checker said "verified" because the variable $s
# was bounded to 32 chars and could never equal the 33-char target.
# sig: (Str) -> Int
# pre: length($s) >= 1
# post: $result == 1
sub ne_long_const_unsound {
    my ($s) = @_;
    my $target = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    return ($s ne $target) ? 1 : 0;
}

# --- Correct: when length is explicitly bounded below the constant ---
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 32
# post: $result == 1
sub ne_long_const_with_bound {
    my ($s) = @_;
    my $target = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    return ($s ne $target) ? 1 : 0;
}

# --- Correct: eq with a constant within MAX_STR_LEN ---
# sig: (Str) -> Int
# pre: $s eq "hello"
# post: $result == 1
sub eq_short_const {
    my ($s) = @_;
    return ($s eq "hello") ? 1 : 0;
}

# --- Correct: ne with empty string when nonempty ---
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result == 1
sub ne_empty_nonempty {
    my ($s) = @_;
    return ($s ne "") ? 1 : 0;
}

# --- Perl tests ---
use lib "$ENV{HOME}/perl5/lib/perl5";
use Test::More tests => 4;

# ne_long_const_unsound: $s can equal the target in Perl
my $s1 = "a" x 33;
my $target = "a" x 33;
my $r1 = ($s1 ne $target) ? 1 : 0;
is($r1, 0, "ne_long_const_unsound: counterexample s='a'x33 gives 0, not 1");

# ne_long_const_with_bound: if $s is bounded to 32 chars it really can't match
my $s2 = "a" x 32;
my $r2 = ($s2 ne $target) ? 1 : 0;
is($r2, 1, "ne_long_const_with_bound: 32-char string ne 33-char constant");

# eq_short_const
my $r3 = ("hello" eq "hello") ? 1 : 0;
is($r3, 1, "eq_short_const: matching constant");

# ne_empty_nonempty
my $r4 = ("x" ne "") ? 1 : 0;
is($r4, 1, "ne_empty_nonempty: nonempty ne empty");
