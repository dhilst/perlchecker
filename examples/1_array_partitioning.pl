# =============================================================
# Domain: Array Partitioning / Statistics (pure, verified)
# =============================================================
# Functions for analyzing bounded integer arrays: counting
# elements below a threshold, range-checking, and summing.
#
# Realistic use case: input validation, histogram bin counting,
# partition logic for sorting, bounded aggregation.
#
# BOUNDARY PUSH: The final function `auto_sum` attempts to use
# scalar(@arr) to obtain array length at runtime, eliminating
# the need to pass length as a separate parameter. This is
# currently unsupported.
# =============================================================

# --- Count elements less than a pivot in a bounded array ---
# sig: (Array<I64>, I64, I64) -> I64
# pre: $len >= 0 && $len <= 5 && $pivot >= 0
# post: $result >= 0 && $result <= $len
sub count_less_than {
    my ($arr, $len, $pivot) = @_;
    my $count = 0;
    my $i;
    for ($i = 0; $i < $len; $i = $i + 1) {
        if ($arr[$i] < $pivot) {
            $count = $count + 1;
        }
    }
    return $count;
}

# --- Check if all elements are in [lo, hi] ---
# sig: (Array<I64>, I64, I64, I64) -> I64
# pre: $len >= 0 && $len <= 5 && $lo <= $hi
# post: $result == 0 || $result == 1
sub all_in_range {
    my ($arr, $len, $lo, $hi) = @_;
    my $i;
    for ($i = 0; $i < $len; $i = $i + 1) {
        if ($arr[$i] < $lo) {
            return 0;
        }
        if ($arr[$i] > $hi) {
            return 0;
        }
    }
    return 1;
}

# --- Sum of array elements (bounded length) ---
# sig: (Array<I64>, I64) -> I64
# pre: $len >= 0 && $len <= 5
# post: $result >= 0 || $result < 0
sub array_sum {
    my ($arr, $len) = @_;
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $len; $i = $i + 1) {
        $sum = $sum + $arr[$i];
    }
    return $sum;
}

# --- BOUNDARY PUSH: scalar(@arr) for array length ---
# This function should sum all elements without needing
# an explicit length parameter. Currently FAILS to parse.
# sig: (Array<I64>) -> I64
# post: $result >= 0 || $result < 0
sub auto_sum {
    my ($arr) = @_;
    my $len = scalar(@arr);
    my $sum = 0;
    my $i;
    for ($i = 0; $i < $len; $i = $i + 1) {
        $sum = $sum + $arr[$i];
    }
    return $sum;
}
