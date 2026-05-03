use std::{fs, process::Command};

use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn check_command_reports_verified_functions() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("sample.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result > $x
sub foo {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int, Int) -> Int
# pre: $y >= 0
# post: $result >= $x
sub bar {
    my ($x, $y) = @_;
    return $x + $y;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "✔ foo: verified\n✔ bar: verified\n"
    );
}

#[test]
fn check_command_fails_for_malformed_annotation_block() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("broken.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result >= $x

sub foo {
    my ($x) = @_;
    return $x;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("invalid sub declaration").eval(&stderr));
}

#[test]
fn check_command_reports_zero_annotated_functions_without_list() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("plain.pl");
    fs::write(
        &file,
        r#"
sub foo {
    return 1;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Found 0 annotated functions\n"
    );
}

#[test]
fn check_command_reports_counterexample_output() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("counterexample.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result > $x
sub foo {
    my ($x) = @_;
    return $x;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✘ foo: counterexample found"));
    assert!(stdout.contains("Function foo failed:"));
    assert!(stdout.contains("x ="));
}

#[test]
fn check_command_supports_elsif_and_safe_division() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("expanded.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result >= 0
sub foo {
    my ($x) = @_;
    if ($x < 0) {
        return 0 - $x;
    } elsif ($x == 0) {
        return 1;
    } else {
        return $x;
    }
}

# sig: (Int, Int) -> Int
# pre: $y != 0
# post: $result == $x / $y
sub bar {
    my ($x, $y) = @_;
    return $x / $y;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "✔ foo: verified\n✔ bar: verified\n"
    );
}

#[test]
fn check_command_rejects_uninitialized_and_unsafe_division() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("invalid.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result >= $x
sub foo {
    my ($x) = @_;
    my $y;
    return $y;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("uninitialized variable").eval(&stderr));
}

#[test]
fn check_command_supports_strings_and_bounded_string_ops() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("strings.pl");
    fs::write(
        &file,
        r#"
# sig: (Str, Str) -> Str
# post: length($result) == length($x) + length($y)
sub concat_len {
    my ($x, $y) = @_;
    return $x . $y;
}

# sig: (Str, Str) -> Int
# post: $result == 0
sub whole_index {
    my ($x, $y) = @_;
    return index($x, $x);
}

# sig: (Str) -> Str
# post: $result eq substr($x, 0, length($x))
sub whole_substr {
    my ($x) = @_;
    return substr($x, 0, length($x));
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ concat_len: verified"));
    assert!(stdout.contains("✔ whole_index: verified"));
    assert!(stdout.contains("✔ whole_substr: verified"));
}

#[test]
fn check_command_reports_string_counterexample_and_unsafe_substr() {
    let tempdir = tempdir().unwrap();
    let counterexample = tempdir.path().join("string_counterexample.pl");
    fs::write(
        &counterexample,
        r#"
# sig: (Str) -> Str
# post: $result eq "fixed"
sub foo {
    my ($x) = @_;
    return $x;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&counterexample)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✘ foo: counterexample found"));
    assert!(stdout.contains("x = \""));

    let invalid = tempdir.path().join("unsafe_substr.pl");
    fs::write(
        &invalid,
        r#"
# sig: (Str) -> Str
# post: $result eq $x
sub bad {
    my ($x) = @_;
    return substr($x, 1, 1);
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&invalid)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("substr").eval(&stderr));
}

#[test]
fn check_command_supports_modulo_and_discards_zero_divisor_paths() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("modulo.pl");
    fs::write(
        &file,
        r#"
# sig: (Int, Int) -> Int
# post: $result == $x % $y
sub mod_ok {
    my ($x, $y) = @_;
    return $x % $y;
}

# sig: (Int, Int) -> Int
# post: $result == 1
sub div_pruned {
    my ($x, $y) = @_;
    if ($y == 0) {
        return $x / $y;
    }
    return 1;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ mod_ok: verified"));
    assert!(stdout.contains("✔ div_pruned: verified"));

    let invalid = tempdir.path().join("modulo_invalid.pl");
    fs::write(
        &invalid,
        r#"
# sig: (Int) -> Int
# post: $result == 0
sub bad {
    my ($x) = @_;
    return $x % 0;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&invalid)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("no valid execution paths").eval(&stderr));
}

#[test]
fn check_command_supports_arrays_and_hashes() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("collections.pl");
    fs::write(
        &file,
        r#"
# sig: (Array<Int>, Int, Int) -> Int
# post: $result == $v
sub array_store {
    my ($arr, $i, $v) = @_;
    $arr[$i] = $v;
    return $arr[$i];
}

# sig: (Hash<Str, Str>, Str, Str) -> Str
# post: $result eq $v
sub hash_store {
    my ($h, $k, $v) = @_;
    $h{$k} = $v;
    return $h{$k};
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ array_store: verified"));
    assert!(stdout.contains("✔ hash_store: verified"));
}

#[test]
fn check_command_rejects_collection_type_errors() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("collection_error.pl");
    fs::write(
        &file,
        r#"
# sig: (Array<Int>, Str) -> Int
# post: $result >= 0
sub bad {
    my ($arr, $k) = @_;
    return $arr[$k];
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("array index").eval(&stderr));
}

#[test]
fn check_command_supports_intra_file_calls_and_rejects_recursion() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("calls.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 1
sub use_inc {
    my ($x) = @_;
    return inc($x);
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ inc: verified"));
    assert!(stdout.contains("✔ use_inc: verified"));

    let recursive = tempdir.path().join("recursive.pl");
    fs::write(
        &recursive,
        r#"
# sig: (Int) -> Int
# post: $result >= $x
sub loop {
    my ($x) = @_;
    return loop($x);
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&recursive)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("recursive call graph").eval(&stderr));
}

#[test]
fn check_command_supports_bounded_loops_and_reports_exhaustion() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("loops.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 5
# post: $result == 0
sub countdown {
    my ($x) = @_;
    while ($x > 0) {
        $x = $x - 1;
    }
    return $x;
}

# sig: (Int) -> Int
# post: $result == $x + 3
sub counted_for {
    my ($x) = @_;
    my $i;
    for ($i = 0; $i < 3; $i = $i + 1) {
        $x = $x + 1;
    }
    return $x;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ countdown: verified"));
    assert!(stdout.contains("✔ counted_for: verified"));

    let invalid = tempdir.path().join("loop_bound.pl");
    fs::write(
        &invalid,
        r#"
# sig: (Int) -> Int
# post: $result == 0
sub spin {
    my ($x) = @_;
    while ($x >= 0) {
        $x = $x + 1;
    }
    return 0;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&invalid)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("loop unroll bound").eval(&stderr));
}

#[test]
fn check_command_reports_path_limit_exceeded() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("too_many_paths.pl");
    fs::write(
        &file,
        r#"
# sig: (Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) -> Int
# post: $result >= 0
sub too_many_paths {
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
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(predicate::str::contains("maximum number of symbolic paths").eval(&stderr));
}

#[test]
fn check_command_supports_calls_in_arbitrary_expression_positions() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("call_positions.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# post: $result == $x + 1
sub inc {
    my ($x) = @_;
    return $x + 1;
}

# sig: (Int) -> Int
# post: $result == $x + 2
sub call_in_binary {
    my ($x) = @_;
    my $z = inc($x) + 1;
    return $z;
}

# sig: (Int) -> Int
# post: $result == $x + 2
sub nested_calls {
    my ($x) = @_;
    return inc(inc($x));
}

# sig: (Int) -> Int
# pre: $x >= 0
# post: $result >= 1
sub call_in_condition {
    my ($x) = @_;
    if (inc($x) > 5) {
        return inc($x);
    }
    return 1;
}

# sig: (Int, Int) -> Int
# post: $result == $x + $y + 2
sub multiple_calls_in_expr {
    my ($x, $y) = @_;
    my $z = inc($x) + inc($y);
    return $z;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ inc: verified"));
    assert!(stdout.contains("✔ call_in_binary: verified"));
    assert!(stdout.contains("✔ nested_calls: verified"));
    assert!(stdout.contains("✔ call_in_condition: verified"));
    assert!(stdout.contains("✔ multiple_calls_in_expr: verified"));
}

#[test]
fn check_foreach_loop_support() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("foreach.pl");
    fs::write(
        &file,
        r#"
# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 1 && scalar(@arr) <= 5
# post: $result >= 0
sub sum_positive {
    my ($arr) = @_;
    my $sum = 0;
    foreach my $x (@arr) {
        if ($x > 0) {
            $sum = $sum + $x;
        }
    }
    return $sum;
}

# sig: (Array<Int>) -> Int
# pre: scalar(@arr) >= 0 && scalar(@arr) <= 4
# post: $result >= 0 && $result <= 4
sub count_positive {
    my ($arr) = @_;
    my $count = 0;
    foreach my $val (@arr) {
        if ($val > 0) {
            $count = $count + 1;
        }
    }
    return $count;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ sum_positive: verified"));
    assert!(stdout.contains("✔ count_positive: verified"));
}

#[test]
fn check_array_literal_initialization() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("array_init.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# pre: $x >= 1 && $x <= 3
# post: $result >= 10 && $result <= 30
sub lookup_table {
    my ($x) = @_;
    my @table = (10, 20, 30);
    my $idx = $x - 1;
    return $table[$idx];
}

# sig: (Int) -> Int
# pre: $n >= 0 && $n <= 3
# post: $result >= 0
sub sum_first_n {
    my ($n) = @_;
    my @vals = (5, 10, 15, 20);
    my $sum = 0;
    my $i = 0;
    for ($i = 0; $i < $n; $i = $i + 1) {
        $sum = $sum + $vals[$i];
    }
    return $sum;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ lookup_table: verified"));
    assert!(stdout.contains("✔ sum_first_n: verified"));
}

#[test]
fn check_regex_match_desugaring() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("regex.pl");
    fs::write(
        &file,
        r#"
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub starts_with_hello {
    my ($s) = @_;
    if ($s =~ /^hello/) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub contains_test {
    my ($s) = @_;
    if ($s =~ /test/) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub not_ending_with_x {
    my ($s) = @_;
    if ($s !~ /x$/) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 10
# post: $result >= 0 && $result <= 1
sub exact_match {
    my ($s) = @_;
    if ($s =~ /^ok$/) {
        return 1;
    }
    return 0;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ starts_with_hello: verified"));
    assert!(stdout.contains("✔ contains_test: verified"));
    assert!(stdout.contains("✔ not_ending_with_x: verified"));
    assert!(stdout.contains("✔ exact_match: verified"));
}

#[test]
fn check_string_ordering_fixed() {
    // Verify that string ordering (lt/gt/le/ge) uses real Z3 lexicographic
    // comparison rather than the old vacuous-true encoding.
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("str_order.pl");
    fs::write(
        &file,
        r#"
# "a" lt "b" is always true lexicographically
# sig: (Int) -> Int
# post: $result == 1
sub a_lt_b {
    my ($dummy) = @_;
    if ("a" lt "b") {
        return 1;
    }
    return 0;
}

# "b" lt "a" is always false — with the old always-true bug this would
# incorrectly verify the postcondition $result == 1
# sig: (Int) -> Int
# post: $result == 0
sub b_not_lt_a {
    my ($dummy) = @_;
    if ("b" lt "a") {
        return 1;
    }
    return 0;
}

# le is reflexive: any string is le itself
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub le_reflexive {
    my ($s) = @_;
    if ($s le $s) {
        return 1;
    }
    return 0;
}

# If a lt b then NOT (b lt a)
# sig: (Str, Str) -> Int
# pre: length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5
# post: $result >= 0 && $result <= 1
sub lt_gt_consistent {
    my ($a, $b) = @_;
    if ($a lt $b) {
        if ($b gt $a) {
            return 1;
        }
        return 0;
    }
    return 0;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ a_lt_b: verified"));
    assert!(stdout.contains("✔ b_not_lt_a: verified"));
    assert!(stdout.contains("✔ le_reflexive: verified"));
    assert!(stdout.contains("✔ lt_gt_consistent: verified"));
}

#[test]
fn check_64bit_bitvectors() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("bv64.pl");
    fs::write(
        &file,
        r#"
# sig: (Int) -> Int
# pre: $x >= 0 && $x <= 255
# post: $result >= 0
sub shift_left_large {
    my ($x) = @_;
    my $result = $x << 33;
    return $result;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0 && $result <= 255
sub bitwise_and_vals {
    my ($a, $b) = @_;
    return $a & $b;
}

# sig: (Int, Int) -> Int
# pre: $a >= 0 && $a <= 255 && $b >= 0 && $b <= 255
# post: $result >= 0
sub bitwise_or_vals {
    my ($a, $b) = @_;
    return $a | $b;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ shift_left_large: verified"));
    assert!(stdout.contains("✔ bitwise_and_vals: verified"));
    assert!(stdout.contains("✔ bitwise_or_vals: verified"));
}

#[test]
fn check_sound_reverse_encoding() {
    let tempdir = tempdir().unwrap();
    let file = tempdir.path().join("reverse_sound.pl");
    fs::write(
        &file,
        r#"
# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub reverse_preserves_length {
    my ($s) = @_;
    my $r = reverse($s);
    if (length($r) == length($s)) {
        return 1;
    }
    return 0;
}

# sig: (Str) -> Int
# pre: length($s) >= 1 && length($s) <= 5
# post: $result == 1
sub double_reverse_identity {
    my ($s) = @_;
    my $r = reverse(reverse($s));
    if ($r eq $s) {
        return 1;
    }
    return 0;
}
"#,
    )
    .unwrap();

    let output = Command::new(cargo_bin("perlchecker"))
        .arg("check")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✔ reverse_preserves_length: verified"));
    assert!(stdout.contains("✔ double_reverse_identity: verified"));
}
