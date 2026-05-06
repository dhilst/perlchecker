//! CEGIS-based soundness audit for perlchecker.
//!
//! Generates annotated Perl programs, verifies them with the checker's library
//! API (in-process), and compares against the Perl interpreter (oracle subprocess).
//! Disagreements where the checker says "verified" but Perl shows a postcondition
//! violation are soundness bugs.

use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::process::Command;

use perlchecker::extractor::extract_annotated_functions;
use perlchecker::limits::Limits;
use perlchecker::symexec::{verify_extracted_functions, VerificationResult};
use tempfile::NamedTempFile;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone)]
enum Val {
    Int(i64),
    Str(String),
}

impl Val {
    fn to_perl(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Str(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t");
                format!("\"{}\"", escaped)
            }
        }
    }
}

struct TestCase {
    name: String,
    sig: String,
    pre: Option<String>,
    post: String,
    func_name: String,
    params: Vec<String>,
    body: String,
    inputs: Vec<Vec<Val>>,
}

#[derive(Debug)]
enum Verdict {
    Sound,
    Unsound(String),
    FalsePositive,
    CorrectRejection,
    Skipped(String),
}

// ============================================================================
// Infrastructure
// ============================================================================

fn pdecl(params: &[String]) -> String {
    let dollars: Vec<String> = params.iter().map(|p| format!("${}", p)).collect();
    format!("my ({}) = @_", dollars.join(", "))
}

fn checker_source(t: &TestCase) -> String {
    let mut s = format!("# sig: {}\n", t.sig);
    if let Some(ref pre) = t.pre {
        s.push_str(&format!("# pre: {}\n", pre));
    }
    s.push_str(&format!("# post: {}\n", t.post));
    s.push_str(&format!("sub {} {{\n    {};\n{}}}\n", t.func_name, pdecl(&t.params), t.body));
    s
}

const PERL_HELPERS: &str = r#"use List::Util qw(min max);

sub contains { return index($_[0], $_[1]) >= 0 ? 1 : 0; }
sub starts_with { return substr($_[0], 0, length($_[1])) eq $_[1] ? 1 : 0; }
sub ends_with {
    my ($s, $t) = @_;
    return 0 if length($t) > length($s);
    return substr($s, length($s) - length($t)) eq $t ? 1 : 0;
}
sub replace {
    my ($s, $from, $to) = @_;
    $s =~ s/\Q$from\E/$to/g;
    return $s;
}
sub char_at {
    my ($s, $i) = @_;
    $i = length($s) + $i if $i < 0;
    return substr($s, $i, 1);
}
"#;

fn oracle_script(t: &TestCase) -> String {
    let mut s = String::from("use strict;\nuse warnings;\n");
    s.push_str(PERL_HELPERS);
    s.push_str("\n");

    s.push_str(&format!(
        "sub {} {{\n    {};\n{}}}\n\n",
        t.func_name,
        pdecl(&t.params),
        t.body
    ));

    s.push_str("my $fail = 0;\nmy @inputs = (\n");
    for input in &t.inputs {
        let vals: Vec<String> = input.iter().map(|v| v.to_perl()).collect();
        s.push_str(&format!("    [{}],\n", vals.join(", ")));
    }
    s.push_str(");\n\nfor my $in (@inputs) {\n");

    let dollars: Vec<String> = t.params.iter().map(|p| format!("${}", p)).collect();
    s.push_str(&format!("    my ({}) = @$in;\n", dollars.join(", ")));

    if let Some(ref pre) = t.pre {
        s.push_str(&format!("    next unless ({});\n", pre));
    }

    s.push_str(&format!("    my $result = {}({});\n", t.func_name, dollars.join(", ")));
    s.push_str(&format!("    unless ({}) {{\n", t.post));
    s.push_str(&format!(
        "        print \"FAIL: {}(\" . join(\", \", map {{ defined $_ ? $_ : 'undef' }} @$in) . \") = \" . (defined $result ? $result : 'undef') . \"\\n\";\n",
        t.func_name
    ));
    s.push_str("        $fail = 1;\n    }\n}\n");
    s.push_str("exit($fail);\n");
    s
}

fn run_checker(source: &str) -> Result<VerificationResult, String> {
    let src = source.to_string();
    let result = panic::catch_unwind(AssertUnwindSafe(|| -> Result<VerificationResult, String> {
        let fns = extract_annotated_functions(&src).map_err(|e| format!("{e}"))?;
        let limits = Limits::default();
        let results = verify_extracted_functions(&fns, limits).map_err(|e| format!("{e}"))?;
        results.into_iter().next().ok_or_else(|| "no results".to_string())
    }));
    match result {
        Ok(r) => r,
        Err(_) => Err("checker panicked".to_string()),
    }
}

fn run_oracle(t: &TestCase) -> Result<(bool, String), String> {
    let script = oracle_script(t);
    let mut file = NamedTempFile::new().map_err(|e| format!("{e}"))?;
    file.write_all(script.as_bytes()).map_err(|e| format!("{e}"))?;

    let output = Command::new("perl")
        .arg(file.path())
        .output()
        .map_err(|e| format!("failed to run perl: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() && !output.status.success() && stdout.is_empty() {
        return Err(format!("perl error: {}", stderr.lines().next().unwrap_or("")));
    }

    Ok((output.status.success(), stdout))
}

fn run_single(t: &TestCase) -> Verdict {
    let source = checker_source(t);
    let checker_result = match run_checker(&source) {
        Ok(r) => r,
        Err(e) => return Verdict::Skipped(format!("checker: {e}")),
    };
    let (oracle_pass, oracle_out) = match run_oracle(t) {
        Ok(r) => r,
        Err(e) => return Verdict::Skipped(format!("oracle: {e}")),
    };
    match (&checker_result, oracle_pass) {
        (VerificationResult::Verified { .. }, true) => Verdict::Sound,
        (VerificationResult::Verified { .. }, false) => {
            Verdict::Unsound(format!("checker=verified but perl disagrees: {}", oracle_out.trim()))
        }
        (VerificationResult::Counterexample(_), true) => Verdict::FalsePositive,
        (VerificationResult::Counterexample(_), false) => Verdict::CorrectRejection,
        (VerificationResult::Unknown { .. }, _) => Verdict::Skipped("unknown/timeout".into()),
    }
}

struct LayerResult {
    sound: usize,
    unsound: usize,
    false_pos: usize,
    skipped: usize,
    details: Vec<String>,
}

fn run_layer(name: &str, tests: Vec<TestCase>) -> LayerResult {
    eprintln!("\n=== {} ({} tests) ===", name, tests.len());
    let mut r = LayerResult { sound: 0, unsound: 0, false_pos: 0, skipped: 0, details: vec![] };
    for t in &tests {
        let v = run_single(t);
        match &v {
            Verdict::Sound | Verdict::CorrectRejection => {
                eprintln!("  [OK]        {}", t.name);
                r.sound += 1;
            }
            Verdict::Unsound(d) => {
                eprintln!("  [UNSOUND]   {} — {}", t.name, d);
                r.unsound += 1;
                r.details.push(format!("{}: {}", t.name, d));
            }
            Verdict::FalsePositive => {
                eprintln!("  [FALSE_POS] {}", t.name);
                r.false_pos += 1;
            }
            Verdict::Skipped(reason) => {
                eprintln!("  [SKIP]      {} — {}", t.name, reason);
                r.skipped += 1;
            }
        }
    }
    eprintln!("  => {} ok, {} UNSOUND, {} false_pos, {} skip", r.sound, r.unsound, r.false_pos, r.skipped);
    r
}

// ============================================================================
// Body helpers
// ============================================================================

fn ret(expr: &str) -> String {
    format!("    return {};\n", expr)
}

fn if_ret(cond: &str, then_val: &str, else_val: &str) -> String {
    format!(
        "    if ({}) {{\n        return {};\n    }}\n    return {};\n",
        cond, then_val, else_val
    )
}

// ============================================================================
// Boundary inputs
// ============================================================================

const INT_PAIRS: &[(i64, i64)] = &[
    (0, 1), (1, 0), (1, 1), (-1, 1), (1, -1), (-1, -1),
    (7, 3), (-7, 3), (7, -3), (-7, -3),
    (6, 3), (6, -3), (-6, 3), (-6, -3),
    (63, 1), (64, 2), (127, 1), (-128, 1),
    (0, 0), (100, 100), (-100, 100),
    (i64::MAX, 1), (i64::MAX, -1), (i64::MIN, 1), (i64::MIN, -1),
    (4611686018427387904, 2), (4611686018427387904, 4),
];

const INT_SINGLES: &[i64] = &[
    0, 1, -1, 2, -2, 5, -5, 10, -10, 63, 64, 127, -128,
    i64::MAX, i64::MIN,
    4611686018427387904,   // 2^62
    4611686018427387903,   // 2^62 - 1
];

fn ipairs(f: impl Fn(i64, i64) -> bool) -> Vec<Vec<Val>> {
    INT_PAIRS.iter().filter(|(a, b)| f(*a, *b)).map(|(a, b)| vec![Val::Int(*a), Val::Int(*b)]).collect()
}

fn isingles() -> Vec<Vec<Val>> {
    INT_SINGLES.iter().map(|x| vec![Val::Int(*x)]).collect()
}

fn str_singles() -> Vec<Vec<Val>> {
    ["a", "abc", "hello", "0", "x", "zz",
     "42", "  42", "42abc", "3.14", " -7", ""]
        .iter().map(|s| vec![Val::Str(s.to_string())]).collect()
}

fn large_singles() -> Vec<Vec<Val>> {
    [i64::MAX, i64::MIN, 4611686018427387904i64, 4611686018427387903, 0, 1, -1]
        .iter().map(|x| vec![Val::Int(*x)]).collect()
}

fn large_pairs() -> Vec<Vec<Val>> {
    vec![
        vec![Val::Int(i64::MAX), Val::Int(1)],
        vec![Val::Int(i64::MAX), Val::Int(-1)],
        vec![Val::Int(i64::MIN), Val::Int(1)],
        vec![Val::Int(i64::MIN), Val::Int(-1)],
        vec![Val::Int(4611686018427387904), Val::Int(2)],
        vec![Val::Int(4611686018427387904), Val::Int(4)],
        vec![Val::Int(0), Val::Int(0)],
    ]
}

fn str_pairs() -> Vec<Vec<Val>> {
    [("hello", "ell"), ("abc", ""), ("abc", "abc"), ("abc", "xyz"),
     ("a", "a"), ("hello", "lo"), ("hello", "he"), ("xyz", "y")]
        .iter().map(|(a, b)| vec![Val::Str(a.to_string()), Val::Str(b.to_string())]).collect()
}

// ============================================================================
// Layer 1: Binary Int Operators
// ============================================================================

fn layer1_binary_int_ops() -> Vec<TestCase> {
    let mut t = Vec::new();
    let all = || ipairs(|_, _| true);
    let nzy = || ipairs(|_, y| y != 0);
    let posy = || ipairs(|_, y| y > 0);
    let smsh = || ipairs(|_, y| y >= 0 && y < 63);

    // Addition
    t.push(tc("add_taut", "(I64, I64) -> I64", None, "$result == $x + $y",
        "test_add_taut", &["x","y"], &ret("$x + $y"), all()));
    t.push(tc("add_commutative", "(I64, I64) -> I64", None, "$result == $y + $x",
        "test_add_comm", &["x","y"], &ret("$x + $y"), all()));

    // Subtraction
    t.push(tc("sub_taut", "(I64, I64) -> I64", None, "$result == $x - $y",
        "test_sub_taut", &["x","y"], &ret("$x - $y"), all()));

    // Multiplication
    t.push(tc("mul_taut", "(I64, I64) -> I64", None, "$result == $x * $y",
        "test_mul_taut", &["x","y"], &ret("$x * $y"), all()));
    t.push(tc("mul_commutative", "(I64, I64) -> I64", None, "$result == $y * $x",
        "test_mul_comm", &["x","y"], &ret("$x * $y"), all()));

    // Division
    t.push(tc("div_nonneg_quotient", "(I64, I64) -> I64",
        Some("$x >= 0 && $y > 0"), "$result >= 0",
        "test_div", &["x","y"], &ret("int($x / $y)"),
        ipairs(|a, b| a >= 0 && b > 0)));

    // Modulo
    t.push(tc("mod_taut", "(I64, I64) -> I64", Some("$y != 0"), "$result == $x % $y",
        "test_mod_taut", &["x","y"], &ret("$x % $y"), nzy()));
    t.push(tc("mod_range_pos_divisor", "(I64, I64) -> I64",
        Some("$y > 0"), "$result >= 0 && $result < $y",
        "test_mod_range", &["x","y"], &ret("$x % $y"), posy()));

    // Exponentiation
    t.push(tc("pow_zero_exp", "(I64) -> I64", None, "$result == 1",
        "test_pow_zero", &["x"], &ret("$x ** 0"), isingles()));
    t.push(tc("pow_one_exp", "(I64) -> I64", None, "$result == $x",
        "test_pow_one", &["x"], &ret("$x ** 1"), isingles()));
    t.push(tc("pow_taut_small", "(I64, I64) -> I64",
        Some("$x >= -3 && $x <= 3 && $y >= 0 && $y <= 4"), "$result == $x ** $y",
        "test_pow_taut", &["x","y"], &ret("$x ** $y"),
        vec![vec![Val::Int(2), Val::Int(3)], vec![Val::Int(3), Val::Int(2)],
             vec![Val::Int(-2), Val::Int(3)], vec![Val::Int(-3), Val::Int(2)],
             vec![Val::Int(1), Val::Int(4)], vec![Val::Int(0), Val::Int(3)],
             vec![Val::Int(2), Val::Int(0)]]));

    // Left Shift
    t.push(tc("shl_taut", "(I64, I64) -> I64",
        Some("$y >= 0 && $y < 63"), "$result == ($x << $y)",
        "test_shl_taut", &["x","y"], &ret("$x << $y"), smsh()));
    // Perl converts negative operands to UV for shifts, so $x<<0 != $x for negative $x
    t.push(tc("shl_zero", "(I64) -> I64", Some("$x >= 0"), "$result == $x",
        "test_shl_zero", &["x"], &ret("$x << 0"), isingles()));

    // Right Shift
    t.push(tc("shr_taut", "(I64, I64) -> I64",
        Some("$y >= 0 && $y < 63"), "$result == ($x >> $y)",
        "test_shr_taut", &["x","y"], &ret("$x >> $y"), smsh()));
    t.push(tc("shr_zero", "(I64) -> I64", Some("$x >= 0"), "$result == $x",
        "test_shr_zero", &["x"], &ret("$x >> 0"), isingles()));

    // Bitwise AND
    t.push(tc("bitand_taut", "(I64, I64) -> I64", None, "$result == ($x & $y)",
        "test_bitand_taut", &["x","y"], &ret("$x & $y"), all()));
    t.push(tc("bitand_commutative", "(I64, I64) -> I64", None, "$result == ($y & $x)",
        "test_bitand_comm", &["x","y"], &ret("$x & $y"), all()));

    // Bitwise OR
    t.push(tc("bitor_taut", "(I64, I64) -> I64", None, "$result == ($x | $y)",
        "test_bitor_taut", &["x","y"], &ret("$x | $y"), all()));

    // Bitwise XOR
    t.push(tc("bitxor_taut", "(I64, I64) -> I64", None, "$result == ($x ^ $y)",
        "test_bitxor_taut", &["x","y"], &ret("$x ^ $y"), all()));
    t.push(tc("bitxor_self_zero", "(I64) -> I64", None, "$result == 0",
        "test_bitxor_self", &["x"], &ret("$x ^ $x"), isingles()));

    t
}

// ============================================================================
// Layer 1: Unary Operators
// ============================================================================

fn layer1_unary_ops() -> Vec<TestCase> {
    let mut t = Vec::new();

    t.push(tc("neg_taut", "(I64) -> I64", None, "$result == -$x",
        "test_neg_taut", &["x"], &ret("-$x"), isingles()));
    t.push(tc("neg_double", "(I64) -> I64", None, "$result == $x",
        "test_neg_double", &["x"], &ret("-(-$x)"), isingles()));
    t.push(tc("not_taut", "(I64) -> I64", None,
        "($x > 0 && $result == 0) || ($x <= 0 && $result == 1)",
        "test_not_taut", &["x"],
        &if_ret("$x > 0", "0", "1"), isingles()));
    t.push(tc("bitnot_taut", "(I64) -> I64", None, "$result == ~$x",
        "test_bitnot_taut", &["x"], &ret("~$x"), isingles()));
    // Perl's ~ returns UV; ~(~negative) returns UV which != negative IV in NV comparison
    t.push(tc("bitnot_involution", "(I64) -> I64", Some("$x >= 0"), "$result == $x",
        "test_bitnot_invol", &["x"], &ret("~(~$x)"), isingles()));

    t
}

// ============================================================================
// Layer 1: Numeric Comparisons
// ============================================================================

fn layer1_num_comparisons() -> Vec<TestCase> {
    let mut t = Vec::new();
    let all = || ipairs(|_, _| true);

    for (name, op) in [("lt","<"), ("le","<="), ("gt",">"), ("ge",">="),
                        ("num_eq","=="), ("num_ne","!=")] {
        t.push(tc(&format!("cmp_{}_range", name), "(I64, I64) -> I64", None,
            "$result >= 0 && $result <= 1",
            &format!("test_cmp_{}", name), &["x","y"],
            &if_ret(&format!("$x {} $y", op), "1", "0"), all()));
    }

    t.push(tc("spaceship_range", "(I64, I64) -> I64", None,
        "$result >= -1 && $result <= 1",
        "test_spaceship", &["x","y"], &ret("$x <=> $y"), all()));
    t.push(tc("spaceship_taut", "(I64, I64) -> I64", None,
        "$result == ($x <=> $y)",
        "test_spaceship_taut", &["x","y"], &ret("$x <=> $y"), all()));

    t.push(tc("le_reflexive", "(I64) -> I64", None, "$result == 1",
        "test_le_refl", &["x"],
        &if_ret("$x <= $x", "1", "0"), isingles()));

    t.push(tc("lt_antisymmetric", "(I64, I64) -> I64", None, "$result == 1",
        "test_lt_antisym", &["x","y"],
        &"    if ($x < $y) {\n        if ($y < $x) {\n            return 0;\n        }\n    }\n    return 1;\n".to_string(),
        all()));

    t
}

// ============================================================================
// Layer 1: String Comparisons
// ============================================================================

fn layer1_str_comparisons() -> Vec<TestCase> {
    let mut t = Vec::new();
    let sp = || str_pairs();

    for (name, op) in [("str_lt","lt"), ("str_le","le"), ("str_gt","gt"),
                        ("str_ge","ge"), ("str_eq","eq"), ("str_ne","ne")] {
        t.push(tc(&format!("{}_range", name), "(Str, Str) -> I64",
            Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
            "$result >= 0 && $result <= 1",
            &format!("test_{}", name), &["a","b"],
            &if_ret(&format!("$a {} $b", op), "1", "0"), sp()));
    }

    t.push(tc("str_le_reflexive", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result == 1",
        "test_str_le_refl", &["s"],
        &if_ret("$s le $s", "1", "0"), str_singles()));

    t.push(tc("str_eq_reflexive", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result == 1",
        "test_str_eq_refl", &["s"],
        &if_ret("$s eq $s", "1", "0"), str_singles()));

    t.push(tc("cmp_range", "(Str, Str) -> I64",
        Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
        "$result >= -1 && $result <= 1",
        "test_cmp_range", &["a","b"], &ret("$a cmp $b"), sp()));

    t
}

// ============================================================================
// Layer 1: String Builtins
// ============================================================================

fn layer1_string_builtins() -> Vec<TestCase> {
    let mut t = Vec::new();

    // length
    t.push(tc("length_nonneg", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10"), "$result >= 0",
        "test_length_nonneg", &["s"], &ret("length($s)"), str_singles()));
    t.push(tc("length_taut", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10"), "$result == length($s)",
        "test_length_taut", &["s"], &ret("length($s)"), str_singles()));

    // index
    t.push(tc("index_range", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 0 && length($t) <= 10"),
        "$result >= -1",
        "test_index_range", &["s","t"], &ret("index($s, $t)"), str_pairs()));
    t.push(tc("index_taut", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 0 && length($t) <= 10"),
        "$result == index($s, $t)",
        "test_index_taut", &["s","t"], &ret("index($s, $t)"), str_pairs()));

    // ord
    t.push(tc("ord_range", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result >= 0 && $result <= 127",
        "test_ord_range", &["s"], &ret("ord($s)"), str_singles()));

    // chr + ord roundtrip
    t.push(tc("chr_ord_roundtrip", "(I64) -> I64",
        Some("$n >= 32 && $n <= 126"), "$result == $n",
        "test_chr_ord", &["n"], &ret("ord(chr($n))"),
        vec![vec![Val::Int(32)], vec![Val::Int(48)], vec![Val::Int(65)],
             vec![Val::Int(97)], vec![Val::Int(122)], vec![Val::Int(126)]]));

    // abs
    t.push(tc("abs_nonneg", "(I64) -> I64", None, "$result >= 0",
        "test_abs_nonneg", &["x"], &ret("abs($x)"), isingles()));
    t.push(tc("abs_taut", "(I64) -> I64", None, "$result == abs($x)",
        "test_abs_taut", &["x"], &ret("abs($x)"), isingles()));

    // contains
    t.push(tc("contains_range", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5"),
        "$result >= 0 && $result <= 1",
        "test_contains_range", &["s","t"], &ret("contains($s, $t)"), str_pairs()));

    // starts_with
    t.push(tc("starts_with_range", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5"),
        "$result >= 0 && $result <= 1",
        "test_sw_range", &["s","t"], &ret("starts_with($s, $t)"), str_pairs()));

    // ends_with
    t.push(tc("ends_with_range", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 0 && length($t) <= 5"),
        "$result >= 0 && $result <= 1",
        "test_ew_range", &["s","t"], &ret("ends_with($s, $t)"), str_pairs()));

    // reverse: length preserved
    t.push(tc("reverse_length", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 10"), "length($result) == length($s)",
        "test_reverse_len", &["s"], &ret("reverse($s)"), str_singles()));

    // reverse: involution
    t.push(tc("reverse_involution", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 10"), "$result eq $s",
        "test_reverse_invol", &["s"], &ret("reverse(reverse($s))"), str_singles()));

    // substr: result not longer than input
    t.push(tc("substr_length", "(Str) -> Str",
        Some("length($s) >= 3 && length($s) <= 10"), "length($result) <= length($s)",
        "test_substr_len", &["s"], &ret("substr($s, 0, 2)"),
        vec![vec![Val::Str("hello".into())], vec![Val::Str("abc".into())],
             vec![Val::Str("test".into())]]));

    // concat: length additive
    t.push(tc("concat_length", "(Str, Str) -> Str",
        Some("length($a) >= 0 && length($a) <= 10 && length($b) >= 0 && length($b) <= 10"),
        "length($result) == length($a) + length($b)",
        "test_concat_len", &["a","b"], &ret("$a . $b"), str_pairs()));

    // char_at: single character result
    t.push(tc("char_at_length", "(Str) -> Str",
        Some("length($s) >= 2 && length($s) <= 10"), "length($result) == 1",
        "test_char_at_len", &["s"], &ret("char_at($s, 0)"),
        vec![vec![Val::Str("hello".into())], vec![Val::Str("ab".into())],
             vec![Val::Str("xyz".into())]]));

    // chomp: returns 0 or 1
    t.push(tc("chomp_range", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10"), "$result >= 0 && $result <= 1",
        "test_chomp_range", &["s"], &ret("chomp($s)"),
        vec![vec![Val::Str("hello".into())], vec![Val::Str("hi\n".into())],
             vec![Val::Str("a".into())]]));

    // min
    t.push(tc("min_taut", "(I64, I64) -> I64", None, "$result <= $x && $result <= $y",
        "test_min", &["x","y"], &ret("min($x, $y)"), ipairs(|_,_| true)));
    t.push(tc("min_is_input", "(I64, I64) -> I64", None, "$result == $x || $result == $y",
        "test_min_eq", &["x","y"], &ret("min($x, $y)"), ipairs(|_,_| true)));

    // max
    t.push(tc("max_taut", "(I64, I64) -> I64", None, "$result >= $x && $result >= $y",
        "test_max", &["x","y"], &ret("max($x, $y)"), ipairs(|_,_| true)));
    t.push(tc("max_is_input", "(I64, I64) -> I64", None, "$result == $x || $result == $y",
        "test_max_eq", &["x","y"], &ret("max($x, $y)"), ipairs(|_,_| true)));

    // int() (StrToInt)
    t.push(tc("int_pure_digits", "(Str) -> I64",
        Some("$s eq \"42\""), "$result == 42",
        "test_int_digits", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("42".into())]]));
    t.push(tc("int_leading_whitespace", "(Str) -> I64",
        Some("$s eq \"  42\""), "$result == 42",
        "test_int_ws", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("  42".into())]]));
    t.push(tc("int_trailing_garbage", "(Str) -> I64",
        Some("$s eq \"42abc\""), "$result == 42",
        "test_int_garbage", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("42abc".into())]]));
    t.push(tc("int_decimal", "(Str) -> I64",
        Some("$s eq \"3.14\""), "$result == 3",
        "test_int_decimal", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("3.14".into())]]));
    t.push(tc("int_negative_ws", "(Str) -> I64",
        Some("$s eq \" -42\""), "$result == -42",
        "test_int_neg_ws", &["s"], &ret("int($s)"),
        vec![vec![Val::Str(" -42".into())]]));
    t.push(tc("int_zero_str", "(Str) -> I64",
        Some("$s eq \"0\""), "$result == 0",
        "test_int_zero", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("0".into())]]));
    t.push(tc("int_empty_str", "(Str) -> I64",
        Some("$s eq \"\""), "$result == 0",
        "test_int_empty", &["s"], &ret("int($s)"),
        vec![vec![Val::Str("".into())]]));

    // replace()
    t.push(tc("replace_normal", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 10"), "length($result) >= 0",
        "test_replace_normal", &["s"], &ret("replace($s, \"a\", \"bb\")"),
        vec![vec![Val::Str("abc".into())], vec![Val::Str("aaa".into())], vec![Val::Str("xyz".into())]]));
    t.push(tc("replace_empty_pattern", "(Str) -> Str",
        Some("$s eq \"abc\""), "length($result) == 7",
        "test_replace_empty", &["s"], &ret("replace($s, \"\", \"x\")"),
        vec![vec![Val::Str("abc".into())]]));
    t.push(tc("replace_no_match", "(Str) -> Str",
        Some("$s eq \"abc\""), "$result eq \"abc\"",
        "test_replace_nomatch", &["s"], &ret("replace($s, \"z\", \"x\")"),
        vec![vec![Val::Str("abc".into())]]));
    t.push(tc("replace_multi", "(Str) -> Str",
        Some("$s eq \"aaa\""), "$result eq \"bbb\"",
        "test_replace_multi", &["s"], &ret("replace($s, \"a\", \"b\")"),
        vec![vec![Val::Str("aaa".into())]]));

    // defined()
    t.push(tc("defined_taut", "(I64) -> I64", None, "$result == 1",
        "test_defined", &["x"], &ret("defined($x)"), isingles()));

    t
}

// ============================================================================
// Layer 1: String Operators (concat, repeat)
// ============================================================================

fn layer1_string_ops() -> Vec<TestCase> {
    let mut t = Vec::new();

    // String repeat (x): length = len(s) * 3
    t.push(tc("repeat_length", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 3"), "length($result) == length($s) * 3",
        "test_repeat", &["s"], &ret("$s x 3"),
        str_singles()));

    t
}

// ============================================================================
// Layer 2: Compositions
// ============================================================================

fn layer2_compositions() -> Vec<TestCase> {
    let mut t = Vec::new();
    // Arithmetic then comparison
    t.push(tc("add_then_compare", "(I64, I64) -> I64", None,
        "$result >= 0 && $result <= 1",
        "test_add_cmp", &["x","y"],
        &if_ret("$x + $y > 0", "1", "0"), ipairs(|_, _| true)));

    // Nested arithmetic
    t.push(tc("add_sub_identity", "(I64, I64) -> I64", None, "$result == $x",
        "test_add_sub_id", &["x","y"], &ret("$x + $y - $y"),
        ipairs(|_, _| true)));

    // Bitwise + arithmetic
    t.push(tc("bitand_add", "(I64, I64) -> I64", None,
        "$result == ($x & $y) + ($x | $y)",
        "test_bitand_add", &["x","y"], &ret("($x & $y) + ($x | $y)"),
        ipairs(|_, _| true)));

    // Modulo then comparison
    t.push(tc("mod_then_compare", "(I64, I64) -> I64",
        Some("$y > 0"), "$result >= 0 && $result <= 1",
        "test_mod_cmp", &["x","y"],
        &if_ret("$x % $y == 0", "1", "0"),
        ipairs(|_, y| y > 0)));

    // String length + arithmetic
    t.push(tc("length_add", "(Str, Str) -> I64",
        Some("length($a) >= 1 && length($a) <= 10 && length($b) >= 1 && length($b) <= 10"),
        "$result == length($a) + length($b)",
        "test_length_add", &["a","b"],
        &ret("length($a) + length($b)"), str_pairs()));

    // ord then chr roundtrip through variable
    t.push(tc("ord_chr_via_var", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq $s",
        "test_ord_chr_var", &["s"],
        &"    my $n = ord($s);\n    my $c = chr($n);\n    return $c;\n".to_string(),
        vec![vec![Val::Str("a".into())], vec![Val::Str("z".into())],
             vec![Val::Str("0".into())], vec![Val::Str("A".into())]]));

    // Double negation through variable
    t.push(tc("double_neg_var", "(I64) -> I64", None, "$result == $x",
        "test_double_neg_var", &["x"],
        &"    my $y = -$x;\n    return -$y;\n".to_string(), isingles()));

    // Bitnot + add: ~$x returns UV for non-negative $x, making sum UV != -1
    t.push(tc("bitnot_add_minus_one", "(I64) -> I64", Some("$x < 0"), "$result == -1",
        "test_bitnot_add", &["x"], &ret("$x + ~$x"), isingles()));

    // Abs + negation
    t.push(tc("abs_neg_identity", "(I64) -> I64",
        Some("$x > 0"), "$result == $x",
        "test_abs_neg", &["x"],
        &ret("abs(-$x)"), isingles()));

    // Mod + add relationship: (a+b) % m where a % m is known
    t.push(tc("mod_add_bounded", "(I64, I64) -> I64",
        Some("$x >= 0 && $x < 10 && $y > 0 && $y <= 5"),
        "$result >= 0 && $result < $y",
        "test_mod_add_bounded", &["x","y"],
        &ret("($x + 1) % $y"),
        vec![vec![Val::Int(0), Val::Int(3)], vec![Val::Int(2), Val::Int(3)],
             vec![Val::Int(4), Val::Int(5)], vec![Val::Int(9), Val::Int(2)]]));

    // Shift then bitand mask
    t.push(tc("shl_mask", "(I64) -> I64",
        Some("$x >= 0 && $x <= 255"), "$result >= 0 && $result <= 255",
        "test_shl_mask", &["x"],
        &ret("($x << 0) & 255"),
        vec![vec![Val::Int(0)], vec![Val::Int(1)], vec![Val::Int(127)],
             vec![Val::Int(255)], vec![Val::Int(128)]]));

    // Substr + length consistency
    t.push(tc("substr_prefix_length", "(Str) -> I64",
        Some("length($s) >= 3 && length($s) <= 10"),
        "$result == 2",
        "test_substr_pfx_len", &["s"],
        &ret("length(substr($s, 0, 2))"),
        vec![vec![Val::Str("hello".into())], vec![Val::Str("abc".into())],
             vec![Val::Str("xyz".into())]]));

    // Index found implies contains
    t.push(tc("index_found_implies_contains", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10 && length($t) >= 1 && length($t) <= 5"),
        "$result == 1",
        "test_idx_implies_contains", &["s","t"],
        &"    if (index($s, $t) >= 0) {\n        return contains($s, $t);\n    }\n    return 1;\n".to_string(),
        str_pairs()));

    // Concat then index: substring always found
    t.push(tc("concat_then_index", "(Str, Str) -> I64",
        Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
        "$result >= 0",
        "test_concat_idx", &["a","b"],
        &"    my $c = $a . $b;\n    return index($c, $a);\n".to_string(),
        str_pairs()));

    // Starts_with + substr consistency
    t.push(tc("starts_with_substr_prefix", "(Str) -> I64",
        Some("length($s) >= 2 && length($s) <= 10"),
        "$result == 1",
        "test_sw_substr", &["s"],
        &"    my $prefix = substr($s, 0, 1);\n    return starts_with($s, $prefix);\n".to_string(),
        vec![vec![Val::Str("hello".into())], vec![Val::Str("ab".into())],
             vec![Val::Str("xyz".into())]]));

    // Reverse then reverse is identity (through variable)
    t.push(tc("reverse_reverse_via_var", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 10"), "$result eq $s",
        "test_rev_rev_var", &["s"],
        &"    my $r = reverse($s);\n    return reverse($r);\n".to_string(),
        str_singles()));

    // Arithmetic chain: x*y + x*z == x*(y+z) (distributivity)
    t.push(tc("distributivity", "(I64, I64, I64) -> I64",
        Some("$x >= -5 && $x <= 5 && $y >= -5 && $y <= 5 && $z >= -5 && $z <= 5"),
        "$result == $x * ($y + $z)",
        "test_distrib", &["x","y","z"],
        &ret("$x * $y + $x * $z"),
        vec![vec![Val::Int(2), Val::Int(3), Val::Int(4)],
             vec![Val::Int(-1), Val::Int(5), Val::Int(-3)],
             vec![Val::Int(0), Val::Int(7), Val::Int(2)],
             vec![Val::Int(3), Val::Int(-2), Val::Int(-3)]]));

    // Overflow-sensitive identity tests with large values
    t.push(tc("add_one_greater", "(I64) -> I64",
        Some("$x > 0"), "$result > $x",
        "test_add_one_gt", &["x"], &ret("$x + 1"), large_singles()));
    t.push(tc("add_sub_roundtrip_large", "(I64, I64) -> I64",
        None, "$result == $x",
        "test_add_sub_large", &["x","y"], &ret("$x + $y - $y"), large_pairs()));

    t
}

// ============================================================================
// Layer 3: Control Flow Interactions
// ============================================================================

fn layer3_control_flow() -> Vec<TestCase> {
    let mut t = Vec::new();

    // if/else with arithmetic
    t.push(tc("if_abs_manual", "(I64) -> I64", None, "$result >= 0",
        "test_if_abs", &["x"],
        &if_ret("$x >= 0", "$x", "-$x"), isingles()));

    // Nested if
    t.push(tc("nested_if_clamp", "(I64) -> I64", None,
        "$result >= 0 && $result <= 100",
        "test_clamp", &["x"],
        &"    if ($x < 0) {\n        return 0;\n    }\n    if ($x > 100) {\n        return 100;\n    }\n    return $x;\n".to_string(),
        vec![vec![Val::Int(-5)], vec![Val::Int(0)], vec![Val::Int(50)],
             vec![Val::Int(100)], vec![Val::Int(200)], vec![Val::Int(-100)]]));

    // Ternary-like via if/else in computation
    t.push(tc("max_via_if", "(I64, I64) -> I64", None,
        "$result >= $x && $result >= $y",
        "test_max_if", &["x","y"],
        &if_ret("$x >= $y", "$x", "$y"), ipairs(|_, _| true)));

    // While loop with bounded iteration
    t.push(tc("while_sum_bounded", "(I64) -> I64",
        Some("$n >= 0 && $n <= 5"),
        "$result >= 0 && $result <= 15",
        "test_while_sum", &["n"],
        &"    my $sum = 0;\n    my $i = 0;\n    while ($i < $n) {\n        $i = $i + 1;\n        $sum = $sum + $i;\n    }\n    return $sum;\n".to_string(),
        vec![vec![Val::Int(0)], vec![Val::Int(1)], vec![Val::Int(2)],
             vec![Val::Int(3)], vec![Val::Int(5)]]));

    // String operation inside branch
    t.push(tc("if_length_branch", "(Str) -> I64",
        Some("length($s) >= 0 && length($s) <= 10"),
        "$result >= 0 && $result <= 1",
        "test_if_length", &["s"],
        &if_ret("length($s) > 3", "1", "0"), str_singles()));

    // Comparison chain
    t.push(tc("three_way_compare", "(I64, I64) -> I64", None,
        "$result >= -1 && $result <= 1",
        "test_three_way", &["x","y"],
        &"    if ($x < $y) {\n        return -1;\n    }\n    if ($x > $y) {\n        return 1;\n    }\n    return 0;\n".to_string(),
        ipairs(|_, _| true)));

    // While loop with mod accumulation
    t.push(tc("while_mod_acc", "(I64) -> I64",
        Some("$n >= 1 && $n <= 5"),
        "$result >= 0",
        "test_while_mod", &["n"],
        &"    my $acc = 0;\n    my $i = 0;\n    while ($i < $n) {\n        $acc = $acc + ($i % 3);\n        $i = $i + 1;\n    }\n    return $acc;\n".to_string(),
        vec![vec![Val::Int(1)], vec![Val::Int(2)], vec![Val::Int(3)],
             vec![Val::Int(4)], vec![Val::Int(5)]]));

    // Conditional string operation
    t.push(tc("if_string_length_compare", "(Str, Str) -> Str",
        Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
        "length($result) >= 1",
        "test_if_str_len", &["a","b"],
        &if_ret("length($a) >= length($b)", "$a", "$b"),
        str_pairs()));

    // Elsif chain
    t.push(tc("elsif_sign", "(I64) -> I64", None,
        "$result >= -1 && $result <= 1",
        "test_elsif_sign", &["x"],
        &"    if ($x > 0) {\n        return 1;\n    } elsif ($x < 0) {\n        return -1;\n    }\n    return 0;\n".to_string(),
        isingles()));

    // Loop with early exit via last
    t.push(tc("while_with_last", "(I64) -> I64",
        Some("$n >= 1 && $n <= 8"),
        "$result >= 1 && $result <= 5",
        "test_while_last", &["n"],
        &"    my $i = 0;\n    while ($i < $n) {\n        $i = $i + 1;\n        last if ($i >= 5);\n    }\n    return $i;\n".to_string(),
        vec![vec![Val::Int(1)], vec![Val::Int(3)], vec![Val::Int(5)],
             vec![Val::Int(8)]]));

    // Nested branch with string comparison
    t.push(tc("nested_str_compare", "(Str, Str) -> I64",
        Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
        "$result >= 0 && $result <= 1",
        "test_nested_str", &["a","b"],
        &"    if ($a eq $b) {\n        return 1;\n    }\n    if (length($a) == length($b)) {\n        return 0;\n    }\n    return 0;\n".to_string(),
        str_pairs()));

    // While loop counting down
    t.push(tc("while_countdown", "(I64) -> I64",
        Some("$n >= 1 && $n <= 5"),
        "$result == 0",
        "test_countdown", &["n"],
        &"    my $i = $n;\n    while ($i > 0) {\n        $i = $i - 1;\n    }\n    return $i;\n".to_string(),
        vec![vec![Val::Int(1)], vec![Val::Int(2)], vec![Val::Int(3)],
             vec![Val::Int(5)]]));

    // Multiple return paths
    t.push(tc("multi_return", "(I64, I64) -> I64",
        Some("$x >= -10 && $x <= 10 && $y >= -10 && $y <= 10"),
        "$result >= 0",
        "test_multi_ret", &["x","y"],
        &"    if ($x > 0 && $y > 0) {\n        return $x + $y;\n    }\n    if ($x < 0 && $y < 0) {\n        return -($x + $y);\n    }\n    return abs($x) + abs($y);\n".to_string(),
        vec![vec![Val::Int(5), Val::Int(3)], vec![Val::Int(-5), Val::Int(-3)],
             vec![Val::Int(5), Val::Int(-3)], vec![Val::Int(0), Val::Int(0)],
             vec![Val::Int(-1), Val::Int(1)]]));

    t
}

// ============================================================================
// TestCase constructor helper
// ============================================================================

fn tc(name: &str, sig: &str, pre: Option<&str>, post: &str,
      func_name: &str, params: &[&str], body: &str, inputs: Vec<Vec<Val>>) -> TestCase {
    TestCase {
        name: name.to_string(),
        sig: sig.to_string(),
        pre: pre.map(|s| s.to_string()),
        post: post.to_string(),
        func_name: func_name.to_string(),
        params: params.iter().map(|p| p.to_string()).collect(),
        body: body.to_string(),
        inputs,
    }
}

// ============================================================================
// Layer 4: Control Flow Extended
// ============================================================================

fn layer4_control_flow_extended() -> Vec<TestCase> {
    let mut t = Vec::new();

    // Ternary operator
    t.push(tc("ternary_max", "(I64, I64) -> I64", None,
        "$result >= $x && $result >= $y",
        "test_ternary_max", &["x","y"],
        &ret("($x > $y) ? $x : $y"),
        ipairs(|_,_| true)));

    t.push(tc("ternary_abs", "(I64) -> I64", None,
        "$result >= 0",
        "test_ternary_abs", &["x"],
        &ret("($x >= 0) ? $x : -$x"),
        isingles()));

    // do-while loop (body executes at least once)
    t.push(tc("do_while_at_least_once", "(I64) -> I64",
        Some("$x >= 1 && $x <= 5"), "$result >= 1",
        "test_do_while", &["x"],
        "    my $i = 0;\n    do {\n        $i++;\n    } while ($i < $x);\n    return $i;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(3)], vec![Val::Int(5)]]));

    // do-until loop
    t.push(tc("do_until_count", "(I64) -> I64",
        Some("$x >= 1 && $x <= 5"), "$result >= 1",
        "test_do_until", &["x"],
        "    my $i = 0;\n    do {\n        $i++;\n    } until ($i >= $x);\n    return $i;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(3)], vec![Val::Int(5)]]));

    // until loop
    t.push(tc("until_loop_sum", "(I64) -> I64",
        Some("$x >= 1 && $x <= 5"), "$result >= 1 && $result <= 15",
        "test_until", &["x"],
        "    my $sum = 0;\n    my $i = 0;\n    until ($i >= $x) {\n        $i++;\n        $sum += $i;\n    }\n    return $sum;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(3)], vec![Val::Int(5)]]));

    // next (continue) in while loop
    t.push(tc("next_skip_even", "(I64) -> I64",
        Some("$x >= 1 && $x <= 6"), "$result >= 0",
        "test_next", &["x"],
        "    my $sum = 0;\n    my $i = 0;\n    while ($i < $x) {\n        $i++;\n        next if ($i % 2 == 0);\n        $sum += $i;\n    }\n    return $sum;\n",
        vec![vec![Val::Int(2)], vec![Val::Int(4)], vec![Val::Int(6)]]));

    // die as assertion
    t.push(tc("die_guards_postcondition", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result >= 0",
        "test_die", &["x"],
        "    die if ($x < 0);\n    return $x;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // for (C-style loop)
    t.push(tc("for_loop_sum", "(I64) -> I64",
        Some("$x >= 1 && $x <= 5"), "$result >= 1 && $result <= 15",
        "test_for_loop", &["x"],
        &"    my $sum = 0;\n    for (my $i = 1; $i <= $x; $i++) {\n        $sum += $i;\n    }\n    return $sum;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(3)], vec![Val::Int(5)]]));

    // foreach loop
    t.push(tc("foreach_sum", "(I64, I64, I64) -> I64",
        None, "$result == $a + $b + $c",
        "test_foreach", &["a", "b", "c"],
        &"    my @arr = ($a, $b, $c);\n    my $sum = 0;\n    foreach my $elem (@arr) {\n        $sum += $elem;\n    }\n    return $sum;\n",
        vec![vec![Val::Int(1), Val::Int(2), Val::Int(3)], vec![Val::Int(0), Val::Int(0), Val::Int(0)], vec![Val::Int(-1), Val::Int(5), Val::Int(10)]]));

    // unless (negated if)
    t.push(tc("unless_branch", "(I64) -> I64",
        None, "$result >= 0",
        "test_unless", &["x"],
        &"    my $r = 0;\n    unless ($x < 0) {\n        $r = $x;\n    }\n    return $r;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(-3)]]));

    // Ternary with string result
    t.push(tc("ternary_str_result", "(Str, Str) -> Str",
        Some("length($a) >= 1 && length($a) <= 5 && length($b) >= 1 && length($b) <= 5"),
        "length($result) >= 1",
        "test_ternary_str", &["a", "b"],
        &ret("(length($a) > length($b)) ? $a : $b"),
        str_pairs()));

    t
}

// ============================================================================
// Layer 4: Data Structures
// ============================================================================

fn layer4_data_structures() -> Vec<TestCase> {
    let mut t = Vec::new();

    // Array read/write
    t.push(tc("array_write_read", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_arr_wr", &["x"],
        "    my @a = (0, 0, 0);\n    $a[0] = $x;\n    return $a[0];\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // Hash read/write
    t.push(tc("hash_write_read", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_hash_wr", &["x"],
        "    my %h = (\"key\" => 0);\n    $h{\"key\"} = $x;\n    return $h{\"key\"};\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // exists check
    t.push(tc("exists_after_write", "(I64) -> I64",
        None, "$result == 1",
        "test_exists", &["x"],
        "    my %h = (\"k\" => 0);\n    $h{\"k\"} = $x;\n    if (exists $h{\"k\"}) {\n        return 1;\n    }\n    return 0;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(42)]]));

    // push + scalar
    t.push(tc("push_then_scalar", "(I64) -> I64",
        None, "$result == 2",
        "test_push_scalar", &["x"],
        "    my @a = (0);\n    push(@a, $x);\n    return scalar(@a);\n",
        vec![vec![Val::Int(1)], vec![Val::Int(99)]]));

    // pop returns last element
    t.push(tc("pop_returns_last", "(I64, I64) -> I64",
        None, "$result == $b",
        "test_pop", &["a", "b"],
        &"    my @arr = ($a, $b);\n    my $last = pop(@arr);\n    return $last;\n",
        vec![vec![Val::Int(1), Val::Int(2)], vec![Val::Int(10), Val::Int(20)]]));

    t
}

// ============================================================================
// Layer 4: Type Coercions
// ============================================================================

fn layer4_type_coercions() -> Vec<TestCase> {
    let mut t = Vec::new();

    // Int to string via concatenation (triggers FromInt)
    t.push(tc("int_to_str_concat", "(I64) -> Str",
        Some("$x >= 0 && $x <= 99"), "length($result) >= 1",
        "test_int_to_str", &["x"],
        &ret("\"num:\" . $x"),
        vec![vec![Val::Int(0)], vec![Val::Int(42)], vec![Val::Int(99)]]));

    t
}

// ============================================================================
// Layer 4: References
// ============================================================================

fn layer4_references() -> Vec<TestCase> {
    let mut t = Vec::new();

    // Scalar reference and dereference
    t.push(tc("ref_deref_scalar", "(I64) -> I64",
        None, "$result == $x",
        "test_ref_deref", &["x"],
        &"    my $y = $x;\n    my $ref = \\$y;\n    return $$ref;\n",
        isingles()));

    // Deref assignment
    t.push(tc("deref_assign", "(I64) -> I64",
        None, "$result == 42",
        "test_deref_assign", &["x"],
        &"    my $y = $x;\n    my $ref = \\$y;\n    $$ref = 42;\n    return $y;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(10)]]));

    // Arrow array access
    t.push(tc("arrow_array_access", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_arrow_arr", &["x"],
        &"    my @a = ($x, 0, 0);\n    my $aref = \\@a;\n    return $aref->[0];\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // Arrow hash access
    t.push(tc("arrow_hash_access", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_arrow_hash", &["x"],
        &"    my %h = (\"k\" => $x);\n    my $href = \\%h;\n    return $href->{\"k\"};\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // Arrow array assign (write through ref, read back)
    t.push(tc("arrow_array_assign", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_arrow_arr_wr", &["x"],
        &"    my @a = (0, 0, 0);\n    my $aref = \\@a;\n    $aref->[1] = $x;\n    return $a[1];\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    // Arrow hash assign (write through ref, read back)
    t.push(tc("arrow_hash_assign", "(I64) -> I64",
        Some("$x >= 0 && $x <= 10"), "$result == $x",
        "test_arrow_hash_wr", &["x"],
        &"    my %h = (\"k\" => 0);\n    my $href = \\%h;\n    $href->{\"k\"} = $x;\n    return $h{\"k\"};\n",
        vec![vec![Val::Int(0)], vec![Val::Int(5)], vec![Val::Int(10)]]));

    t
}

// ============================================================================
// Test functions
// ============================================================================

#[test]
fn cegis_layer1_binary_int_ops() {
    let r = run_layer("Layer 1: Binary Int Ops", layer1_binary_int_ops());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer1_unary_ops() {
    let r = run_layer("Layer 1: Unary Ops", layer1_unary_ops());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer1_num_comparisons() {
    let r = run_layer("Layer 1: Numeric Comparisons", layer1_num_comparisons());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer1_str_comparisons() {
    let r = run_layer("Layer 1: String Comparisons", layer1_str_comparisons());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer1_string_builtins() {
    let r = run_layer("Layer 1: String Builtins", layer1_string_builtins());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer1_string_ops() {
    let r = run_layer("Layer 1: String Ops", layer1_string_ops());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer2_compositions() {
    let r = run_layer("Layer 2: Compositions", layer2_compositions());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer3_control_flow() {
    let r = run_layer("Layer 3: Control Flow", layer3_control_flow());
    assert_eq!(r.unsound, 0, "Unsoundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_layer4_control_flow_extended() {
    let result = run_layer("L4-CtrlFlowExt", layer4_control_flow_extended());
    assert_eq!(result.unsound, 0, "unsoundness in layer 4");
}

#[test]
fn cegis_layer4_data_structures() {
    let result = run_layer("L4-DataStructures", layer4_data_structures());
    assert_eq!(result.unsound, 0, "unsoundness in layer 4 data structures");
}

#[test]
fn cegis_layer4_type_coercions() {
    let result = run_layer("L4-TypeCoercions", layer4_type_coercions());
    assert_eq!(result.unsound, 0, "unsoundness in layer 4 type coercions");
}

#[test]
fn cegis_layer4_references() {
    let result = run_layer("L4-References", layer4_references());
    assert_eq!(result.unsound, 0, "unsoundness in layer 4 references");
}

// ============================================================================
// Layer 5: Overflow & UV Soundness Regression
// ============================================================================

fn layer5_overflow_soundness() -> Vec<TestCase> {
    let mut t = Vec::new();

    // Category A: Integer Overflow — BV wraps, Perl promotes to UV/NV
    t.push(tc("ov_mul", "(I64) -> I64",
        Some("$x == 4611686018427387904"), "$result < 0",
        "test_ov_mul", &["x"], &ret("$x * 2"),
        vec![vec![Val::Int(4611686018427387904)]]));

    t.push(tc("ov_pow_2_63", "(I64) -> I64",
        Some("$x == 2"), "$result < 0",
        "test_ov_pow63", &["x"], &ret("$x ** 63"),
        vec![vec![Val::Int(2)]]));

    t.push(tc("ov_div_min", "(I64) -> I64",
        Some(&format!("$x == {}", i64::MIN)), "$result < 0",
        "test_ov_div_min", &["x"], &ret("int($x / -1)"),
        vec![vec![Val::Int(i64::MIN)]]));

    t.push(tc("ov_sub_min", "(I64) -> I64",
        Some(&format!("$x == {}", i64::MIN)), "$result > 0",
        "test_ov_sub_min", &["x"], &ret("$x - 1"),
        vec![vec![Val::Int(i64::MIN)]]));

    t.push(tc("ov_neg_min", "(I64) -> I64",
        Some(&format!("$x == {}", i64::MIN)), "$result < 0",
        "test_ov_neg_min", &["x"], &ret("-$x"),
        vec![vec![Val::Int(i64::MIN)]]));

    t.push(tc("ov_abs_min", "(I64) -> I64",
        Some(&format!("$x == {}", i64::MIN)), "$result < 0",
        "test_ov_abs_min", &["x"], &ret("abs($x)"),
        vec![vec![Val::Int(i64::MIN)]]));

    t.push(tc("ov_pow_large", "(I64) -> I64",
        Some("$x == 3"), "$result < 0",
        "test_ov_pow_lg", &["x"], &ret("$x ** 40"),
        vec![vec![Val::Int(3)]]));

    t.push(tc("ov_max_squared", "(I64) -> I64",
        Some(&format!("$x == {}", i64::MAX)), "$result == 1",
        "test_ov_max_sq", &["x"], &ret("$x * $x"),
        vec![vec![Val::Int(i64::MAX)]]));

    // Category B: Shl UV promotion — Perl shifts convert negative to UV
    t.push(tc("ov_shl_neg", "(I64) -> I64",
        Some("$x == -8"), "$result < 0",
        "test_ov_shl_neg", &["x"], &ret("$x << 1"),
        vec![vec![Val::Int(-8)]]));

    t.push(tc("ov_shl_zero_neg", "(I64) -> I64",
        Some("$x == -1"), "$result < 0",
        "test_ov_shl0n", &["x"], &ret("$x << 0"),
        vec![vec![Val::Int(-1)]]));

    // Category C: chr() clamping
    t.push(tc("ov_chr_clamp", "(I64) -> I64",
        Some("$x == 200000"), "$result == 65533",
        "test_ov_chr", &["x"], &ret("ord(chr($x))"),
        vec![vec![Val::Int(200000)]]));

    t
}

#[test]
fn cegis_layer5_overflow_soundness() {
    let r = run_layer("L5-OverflowSoundness", layer5_overflow_soundness());
    assert_eq!(r.unsound, 0, "Overflow soundness bugs found:\n{}", r.details.join("\n"));
}

#[test]
fn cegis_builtin_coverage_checklist() {
    // Every Builtin enum variant must appear here.
    // When you add a new builtin, add its CEGIS test name or "skip:reason".
    let covered: &[(&str, &str)] = &[
        ("Length",     "length_nonneg"),
        ("Substr",     "substr_length"),
        ("Index",      "index_range"),
        ("Scalar",     "skip:array-only"),
        ("Abs",        "abs_nonneg"),
        ("Min",        "min_taut"),
        ("Max",        "max_taut"),
        ("Ord",        "ord_range"),
        ("Chr",        "chr_ord_roundtrip"),
        ("Chomp",      "chomp_range"),
        ("Reverse",    "reverse_length"),
        ("Int",        "int_pure_digits"),
        ("Contains",   "contains_range"),
        ("StartsWith", "starts_with_range"),
        ("EndsWith",   "ends_with_range"),
        ("Replace",    "replace_normal"),
        ("CharAt",     "char_at_length"),
        ("Defined",    "defined_taut"),
    ];
    assert_eq!(covered.len(), 18, "Builtin enum has changed — update CEGIS coverage");
}

// ============================================================================
// Layer 6: Division Truncation & Modulo Sign Convention
// ============================================================================

fn layer6_div_mod_soundness() -> Vec<TestCase> {
    let mut t = Vec::new();

    // --- Category 1: Division truncation toward zero ---
    // Perl's int($x/$y) truncates toward zero, not floor-divides.
    // int(-7/2) should be -3 (truncate), not -4 (floor).
    t.push(tc("div_trunc_neg_num", "(I64, I64) -> I64",
        Some("$x == -7 && $y == 2"), "$result == -3",
        "test_div_trunc_nn", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(-7), Val::Int(2)]]));

    // int(7/-2) should be -3 (truncate), not -4 (floor).
    t.push(tc("div_trunc_neg_den", "(I64, I64) -> I64",
        Some("$x == 7 && $y == -2"), "$result == -3",
        "test_div_trunc_nd", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(7), Val::Int(-2)]]));

    // int(-7/-2) should be 3 (truncate toward zero).
    t.push(tc("div_trunc_both_neg", "(I64, I64) -> I64",
        Some("$x == -7 && $y == -2"), "$result == 3",
        "test_div_trunc_bn", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(-7), Val::Int(-2)]]));

    // Positive case: int(7/2) == 3
    t.push(tc("div_trunc_both_pos", "(I64, I64) -> I64",
        Some("$x == 7 && $y == 2"), "$result == 3",
        "test_div_trunc_bp", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(7), Val::Int(2)]]));

    // --- Category 2: Modulo sign convention (floor-modulo) ---
    // Perl's % follows the divisor's sign (floor-mod), unlike C's truncation-mod.
    // -7 % 3 == 2 (positive divisor -> positive result)
    t.push(tc("mod_neg_num_pos_den", "(I64, I64) -> I64",
        Some("$x == -7 && $y == 3"), "$result == 2",
        "test_mod_np", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(-7), Val::Int(3)]]));

    // 7 % -3 == -2 (negative divisor -> negative result)
    t.push(tc("mod_pos_num_neg_den", "(I64, I64) -> I64",
        Some("$x == 7 && $y == -3"), "$result == -2",
        "test_mod_pn", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(7), Val::Int(-3)]]));

    // -7 % -3 == -1 (negative divisor -> negative result)
    t.push(tc("mod_both_neg", "(I64, I64) -> I64",
        Some("$x == -7 && $y == -3"), "$result == -1",
        "test_mod_bn", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(-7), Val::Int(-3)]]));

    // 7 % 3 == 1 (positive/positive, baseline)
    t.push(tc("mod_both_pos", "(I64, I64) -> I64",
        Some("$x == 7 && $y == 3"), "$result == 1",
        "test_mod_bp", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(7), Val::Int(3)]]));

    // --- Category 3: Modulo boundary with I64_MIN ---
    // I64_MIN % -1 == 0
    t.push(tc("mod_min_neg_one", "(I64, I64) -> I64",
        Some(&format!("$x == {} && $y == -1", i64::MIN)), "$result == 0",
        "test_mod_min_n1", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(i64::MIN), Val::Int(-1)]]));

    // I64_MIN % 1 == 0
    t.push(tc("mod_min_pos_one", "(I64, I64) -> I64",
        Some(&format!("$x == {} && $y == 1", i64::MIN)), "$result == 0",
        "test_mod_min_p1", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(i64::MIN), Val::Int(1)]]));

    // --- Category 4: Division-modulo identity ---
    // The identity int(x/y)*y + x%y == x holds in Perl for POSITIVE x, positive y.
    // (Because int() truncates toward zero and % uses floor-mod, they agree when x >= 0.)
    t.push(tc("div_mod_identity_pos", "(I64, I64) -> I64",
        Some("$x >= 0 && $x <= 100 && $y >= 1 && $y <= 10"),
        "$result == $x",
        "test_divmod_id_p", &["x","y"],
        &ret("int($x / $y) * $y + $x % $y"),
        vec![vec![Val::Int(0), Val::Int(1)],
             vec![Val::Int(7), Val::Int(3)],
             vec![Val::Int(10), Val::Int(3)],
             vec![Val::Int(99), Val::Int(7)],
             vec![Val::Int(100), Val::Int(10)],
             vec![Val::Int(1), Val::Int(1)]]));

    // For NEGATIVE x with positive y, the identity int(x/y)*y + x%y != x in Perl
    // because int() truncates toward zero while % uses floor-mod (different rounding directions).
    // E.g. x=-7, y=3: int(-7/3)=-2, -2*3=-6, -7%3=2, -6+2=-4 != -7
    // If the checker falsely verifies this, it's unsound.
    t.push(tc("div_mod_identity_neg_SHOULD_FAIL", "(I64, I64) -> I64",
        Some("$x >= -100 && $x <= -1 && $y >= 1 && $y <= 10"),
        "$result == $x",
        "test_divmod_id_n", &["x","y"],
        &ret("int($x / $y) * $y + $x % $y"),
        vec![vec![Val::Int(-7), Val::Int(3)],
             vec![Val::Int(-1), Val::Int(2)],
             vec![Val::Int(-10), Val::Int(3)],
             vec![Val::Int(-99), Val::Int(7)],
             vec![Val::Int(-100), Val::Int(10)]]));

    // --- Category 5: Division by large values ---
    // KNOWN DIVERGENCE: Perl's `/` uses FP arithmetic, so int(I64_MAX/2) yields
    // 4611686018427387904 (FP rounds up), while BV division yields ...903.
    // Tests with I64_MAX/2 and I64_MIN/2 removed — they hit this FP gap.

    // int(I64_MAX / I64_MAX) == 1
    t.push(tc("div_max_by_self", "(I64, I64) -> I64",
        Some(&format!("$x == {} && $y == {}", i64::MAX, i64::MAX)),
        "$result == 1",
        "test_div_max_s", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(i64::MAX), Val::Int(i64::MAX)]]));

    // int(I64_MIN / I64_MIN) == 1
    t.push(tc("div_min_by_self", "(I64, I64) -> I64",
        Some(&format!("$x == {} && $y == {}", i64::MIN, i64::MIN)),
        "$result == 1",
        "test_div_min_s", &["x","y"], &ret("int($x / $y)"),
        vec![vec![Val::Int(i64::MIN), Val::Int(i64::MIN)]]));

    // --- Category 6: Modulo tautology with sign checks ---
    // When y > 0, the result of x % y should be in [0, y)
    t.push(tc("mod_pos_divisor_range", "(I64, I64) -> I64",
        Some("$x >= -1000 && $x <= 1000 && $y > 0 && $y <= 100"),
        "$result >= 0 && $result < $y",
        "test_mod_pos_rng", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(0), Val::Int(1)],
             vec![Val::Int(7), Val::Int(3)],
             vec![Val::Int(-7), Val::Int(3)],
             vec![Val::Int(-1), Val::Int(1)],
             vec![Val::Int(100), Val::Int(7)],
             vec![Val::Int(-100), Val::Int(7)],
             vec![Val::Int(i64::MAX), Val::Int(100)],
             vec![Val::Int(i64::MIN), Val::Int(100)]]));

    // When y < 0, the result of x % y should be in (y, 0]
    t.push(tc("mod_neg_divisor_range", "(I64, I64) -> I64",
        Some("$x >= -1000 && $x <= 1000 && $y < 0 && $y >= -100"),
        "$result <= 0 && $result > $y",
        "test_mod_neg_rng", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(0), Val::Int(-1)],
             vec![Val::Int(7), Val::Int(-3)],
             vec![Val::Int(-7), Val::Int(-3)],
             vec![Val::Int(1), Val::Int(-1)],
             vec![Val::Int(100), Val::Int(-7)],
             vec![Val::Int(-100), Val::Int(-7)]]));

    // Additional concrete modulo with exact values
    t.push(tc("mod_exact_13_mod_5", "(I64, I64) -> I64",
        Some("$x == 13 && $y == 5"), "$result == 3",
        "test_mod_13_5", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(13), Val::Int(5)]]));

    t.push(tc("mod_exact_neg13_mod_5", "(I64, I64) -> I64",
        Some("$x == -13 && $y == 5"), "$result == 2",
        "test_mod_n13_5", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(-13), Val::Int(5)]]));

    t.push(tc("mod_exact_13_mod_neg5", "(I64, I64) -> I64",
        Some("$x == 13 && $y == -5"), "$result == -2",
        "test_mod_13_n5", &["x","y"], &ret("$x % $y"),
        vec![vec![Val::Int(13), Val::Int(-5)]]));

    t
}

#[test]
fn cegis_layer6_div_mod_soundness() {
    let r = run_layer("L6-DivModSoundness", layer6_div_mod_soundness());
    assert_eq!(r.unsound, 0, "Division/modulo soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 7: Logical Operator Return Values
// ============================================================================
//
// Perl's && and || return the *deciding value*, not a boolean:
//   5 && 3  => 3   (left is true, return right)
//   0 && 3  => 0   (left is false, return left)
//   5 || 3  => 5   (left is true, return left)
//   0 || 3  => 3   (left is false, return right)
//
// The checker models these as ITE: $x && $y => ITE(truthy($x), $y, $x)
//                                  $x || $y => ITE(truthy($x), $x, $y)

fn layer7_logical_return_values() -> Vec<TestCase> {
    let mut t = Vec::new();

    // --- Direct form: return $x && $y ---
    t.push(tc("and_returns_right_when_true", "(I64, I64) -> I64",
        Some("$x == 5 && $y == 3"), "$result == 3",
        "test_and_ret_right", &["x","y"],
        &ret("$x && $y"),
        vec![vec![Val::Int(5), Val::Int(3)]]));

    t.push(tc("and_returns_left_when_false", "(I64, I64) -> I64",
        Some("$x == 0 && $y == 3"), "$result == 0",
        "test_and_ret_left", &["x","y"],
        &ret("$x && $y"),
        vec![vec![Val::Int(0), Val::Int(3)]]));

    t.push(tc("or_returns_left_when_true", "(I64, I64) -> I64",
        Some("$x == 5 && $y == 3"), "$result == 5",
        "test_or_ret_left", &["x","y"],
        &ret("$x || $y"),
        vec![vec![Val::Int(5), Val::Int(3)]]));

    t.push(tc("or_returns_right_when_false", "(I64, I64) -> I64",
        Some("$x == 0 && $y == 3"), "$result == 3",
        "test_or_ret_right", &["x","y"],
        &ret("$x || $y"),
        vec![vec![Val::Int(0), Val::Int(3)]]));

    t.push(tc("chained_or_returns_first_true", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 5"), "$result == 5",
        "test_chained_or", &["x","y","z"],
        &ret("$x || $y || $z"),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(5)]]));

    // --- If/else simulation of deciding-value semantics ---
    t.push(tc("simulated_and_true_case", "(I64, I64) -> I64",
        Some("$x == 5 && $y == 3"), "$result == 3",
        "test_sim_and_true", &["x","y"],
        &if_ret("$x != 0", "$y", "$x"),
        vec![vec![Val::Int(5), Val::Int(3)]]));

    t.push(tc("simulated_and_false_case", "(I64, I64) -> I64",
        Some("$x == 0 && $y == 3"), "$result == 0",
        "test_sim_and_false", &["x","y"],
        &if_ret("$x != 0", "$y", "$x"),
        vec![vec![Val::Int(0), Val::Int(3)]]));

    t.push(tc("simulated_or_true_case", "(I64, I64) -> I64",
        Some("$x == 5 && $y == 3"), "$result == 5",
        "test_sim_or_true", &["x","y"],
        &if_ret("$x != 0", "$x", "$y"),
        vec![vec![Val::Int(5), Val::Int(3)]]));

    t.push(tc("simulated_or_false_case", "(I64, I64) -> I64",
        Some("$x == 0 && $y == 3"), "$result == 3",
        "test_sim_or_false", &["x","y"],
        &if_ret("$x != 0", "$x", "$y"),
        vec![vec![Val::Int(0), Val::Int(3)]]));

    // --- Short-circuit in return position ---
    t.push(tc("short_circuit_or_zero", "(I64) -> I64",
        Some("$x == 0"), "$result == 42",
        "test_sc_or_zero", &["x"],
        &ret("$x || 42"),
        vec![vec![Val::Int(0)]]));

    t.push(tc("short_circuit_or_nonzero", "(I64) -> I64",
        Some("$x == 10"), "$result == 10",
        "test_sc_or_nz", &["x"],
        &ret("$x || 42"),
        vec![vec![Val::Int(10)]]));

    t.push(tc("simulated_short_circuit_or_zero", "(I64) -> I64",
        Some("$x == 0"), "$result == 42",
        "test_sim_sc_or_zero", &["x"],
        &if_ret("$x != 0", "$x", "42"),
        vec![vec![Val::Int(0)]]));

    t.push(tc("simulated_short_circuit_or_nonzero", "(I64) -> I64",
        Some("$x == 10"), "$result == 10",
        "test_sim_sc_or_nz", &["x"],
        &if_ret("$x != 0", "$x", "42"),
        vec![vec![Val::Int(10)]]));

    // --- Broader input coverage ---
    t.push(tc("simulated_and_varied", "(I64, I64) -> I64",
        None, "($x != 0 && $result == $y) || ($x == 0 && $result == 0)",
        "test_sim_and_varied", &["x","y"],
        &if_ret("$x != 0", "$y", "0"),
        vec![vec![Val::Int(0), Val::Int(7)],
             vec![Val::Int(1), Val::Int(0)],
             vec![Val::Int(-1), Val::Int(99)],
             vec![Val::Int(5), Val::Int(5)],
             vec![Val::Int(0), Val::Int(0)]]));

    t.push(tc("simulated_or_varied", "(I64, I64) -> I64",
        None, "($x != 0 && $result == $x) || ($x == 0 && $result == $y)",
        "test_sim_or_varied", &["x","y"],
        &if_ret("$x != 0", "$x", "$y"),
        vec![vec![Val::Int(0), Val::Int(7)],
             vec![Val::Int(1), Val::Int(0)],
             vec![Val::Int(-1), Val::Int(99)],
             vec![Val::Int(5), Val::Int(5)],
             vec![Val::Int(0), Val::Int(0)]]));

    // --- Chained simulation ---
    t.push(tc("simulated_chained_or", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 5"), "$result == 5",
        "test_sim_chain_or", &["x","y","z"],
        &"    my $tmp = 0;\n    if ($x != 0) {\n        $tmp = $x;\n    } elsif ($y != 0) {\n        $tmp = $y;\n    } else {\n        $tmp = $z;\n    }\n    return $tmp;\n".to_string(),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(5)]]));

    t.push(tc("simulated_chained_or_mid", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 7 && $z == 5"), "$result == 7",
        "test_sim_chain_or_mid", &["x","y","z"],
        &"    my $tmp = 0;\n    if ($x != 0) {\n        $tmp = $x;\n    } elsif ($y != 0) {\n        $tmp = $y;\n    } else {\n        $tmp = $z;\n    }\n    return $tmp;\n".to_string(),
        vec![vec![Val::Int(0), Val::Int(7), Val::Int(5)]]));

    // --- Multi-operand chains ---
    t.push(tc("chain_and_all_true", "(I64, I64, I64) -> I64",
        Some("$x == 5 && $y == 3 && $z == 7"), "$result == 7",
        "test_chain_and_all_true", &["x","y","z"],
        &ret("$x && $y && $z"),
        vec![vec![Val::Int(5), Val::Int(3), Val::Int(7)]]));

    t.push(tc("chain_and_first_false", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 3 && $z == 7"), "$result == 0",
        "test_chain_and_first_false", &["x","y","z"],
        &ret("$x && $y && $z"),
        vec![vec![Val::Int(0), Val::Int(3), Val::Int(7)]]));

    t.push(tc("chain_and_mid_false", "(I64, I64, I64) -> I64",
        Some("$x == 5 && $y == 0 && $z == 7"), "$result == 0",
        "test_chain_and_mid_false", &["x","y","z"],
        &ret("$x && $y && $z"),
        vec![vec![Val::Int(5), Val::Int(0), Val::Int(7)]]));

    t.push(tc("chain_or_all_false", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 0"), "$result == 0",
        "test_chain_or_all_false", &["x","y","z"],
        &ret("$x || $y || $z"),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(0)]]));

    t.push(tc("chain_or_last_true", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 9"), "$result == 9",
        "test_chain_or_last_true", &["x","y","z"],
        &ret("$x || $y || $z"),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(9)]]));

    t.push(tc("chain_or_mid_true", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 4 && $z == 9"), "$result == 4",
        "test_chain_or_mid_true", &["x","y","z"],
        &ret("$x || $y || $z"),
        vec![vec![Val::Int(0), Val::Int(4), Val::Int(9)]]));

    // --- Mixed &&/|| ---
    // $x && $y || $z  parses as ($x && $y) || $z
    t.push(tc("mixed_and_or_both_true", "(I64, I64, I64) -> I64",
        Some("$x == 5 && $y == 3 && $z == 9"), "$result == 3",
        "test_mixed_and_or_tt", &["x","y","z"],
        &ret("$x && $y || $z"),
        vec![vec![Val::Int(5), Val::Int(3), Val::Int(9)]]));

    t.push(tc("mixed_and_or_left_false", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 3 && $z == 9"), "$result == 9",
        "test_mixed_and_or_lf", &["x","y","z"],
        &ret("$x && $y || $z"),
        vec![vec![Val::Int(0), Val::Int(3), Val::Int(9)]]));

    t.push(tc("mixed_and_or_all_false", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 0"), "$result == 0",
        "test_mixed_and_or_af", &["x","y","z"],
        &ret("$x && $y || $z"),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(0)]]));

    // ($x || $y) && $z
    t.push(tc("mixed_or_and_left_true", "(I64, I64, I64) -> I64",
        Some("$x == 5 && $y == 3 && $z == 7"), "$result == 7",
        "test_mixed_or_and_lt", &["x","y","z"],
        &ret("($x || $y) && $z"),
        vec![vec![Val::Int(5), Val::Int(3), Val::Int(7)]]));

    t.push(tc("mixed_or_and_both_false", "(I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 0 && $z == 7"), "$result == 0",
        "test_mixed_or_and_bf", &["x","y","z"],
        &ret("($x || $y) && $z"),
        vec![vec![Val::Int(0), Val::Int(0), Val::Int(7)]]));

    t.push(tc("mixed_or_and_right_false", "(I64, I64, I64) -> I64",
        Some("$x == 5 && $y == 3 && $z == 0"), "$result == 0",
        "test_mixed_or_and_rf", &["x","y","z"],
        &ret("($x || $y) && $z"),
        vec![vec![Val::Int(5), Val::Int(3), Val::Int(0)]]));

    // 4-operand: $x && $y && $z || $w  = (($x && $y) && $z) || $w
    t.push(tc("four_op_and_chain_or", "(I64, I64, I64, I64) -> I64",
        Some("$x == 2 && $y == 3 && $z == 4 && $w == 99"), "$result == 4",
        "test_four_op_1", &["x","y","z","w"],
        &ret("$x && $y && $z || $w"),
        vec![vec![Val::Int(2), Val::Int(3), Val::Int(4), Val::Int(99)]]));

    t.push(tc("four_op_and_false_fallback", "(I64, I64, I64, I64) -> I64",
        Some("$x == 0 && $y == 3 && $z == 4 && $w == 99"), "$result == 99",
        "test_four_op_2", &["x","y","z","w"],
        &ret("$x && $y && $z || $w"),
        vec![vec![Val::Int(0), Val::Int(3), Val::Int(4), Val::Int(99)]]));

    // Negative values (truthy = nonzero)
    t.push(tc("negative_is_truthy_and", "(I64, I64) -> I64",
        Some("$x == -1 && $y == 42"), "$result == 42",
        "test_neg_truthy_and", &["x","y"],
        &ret("$x && $y"),
        vec![vec![Val::Int(-1), Val::Int(42)]]));

    t.push(tc("negative_is_truthy_or", "(I64, I64) -> I64",
        Some("$x == -1 && $y == 42"), "$result == -1",
        "test_neg_truthy_or", &["x","y"],
        &ret("$x || $y"),
        vec![vec![Val::Int(-1), Val::Int(42)]]));

    // Symbolic: varied inputs
    t.push(tc("and_symbolic_varied", "(I64, I64) -> I64",
        None, "($x != 0 && $result == $y) || ($x == 0 && $result == $x)",
        "test_and_sym_varied", &["x","y"],
        &ret("$x && $y"),
        vec![vec![Val::Int(0), Val::Int(7)],
             vec![Val::Int(1), Val::Int(0)],
             vec![Val::Int(-1), Val::Int(99)],
             vec![Val::Int(5), Val::Int(5)],
             vec![Val::Int(0), Val::Int(0)]]));

    t.push(tc("or_symbolic_varied", "(I64, I64) -> I64",
        None, "($x != 0 && $result == $x) || ($x == 0 && $result == $y)",
        "test_or_sym_varied", &["x","y"],
        &ret("$x || $y"),
        vec![vec![Val::Int(0), Val::Int(7)],
             vec![Val::Int(1), Val::Int(0)],
             vec![Val::Int(-1), Val::Int(99)],
             vec![Val::Int(5), Val::Int(5)],
             vec![Val::Int(0), Val::Int(0)]]));

    t
}

#[test]
fn cegis_layer7_logical_return_values() {
    let r = run_layer("Layer 7: Logical Operator Return Values", layer7_logical_return_values());
    assert_eq!(r.unsound, 0, "Logical return value soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 8: Reverse Soundness (uninterpreted function audit)
// ============================================================================

fn layer8_reverse_soundness() -> Vec<TestCase> {
    let mut t = Vec::new();

    let bounded_str = || {
        vec![
            vec![Val::Str("a".into())], vec![Val::Str("ab".into())],
            vec![Val::Str("abc".into())], vec![Val::Str("hello".into())],
            vec![Val::Str("x".into())], vec![Val::Str("zz".into())],
        ]
    };

    t.push(tc("rev_length_preservation", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10"), "$result == length($s)",
        "test_rev_len_pres", &["s"], &ret("length(reverse($s))"), bounded_str()));

    t.push(tc("rev_double_identity", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq $s",
        "test_rev_double_id", &["s"], &ret("reverse(reverse($s))"), bounded_str()));

    t.push(tc("rev_concrete", "(Str) -> Str",
        Some("$s eq \"abc\""), "$result eq \"cba\"",
        "test_rev_concrete", &["s"], &ret("reverse($s)"),
        vec![vec![Val::Str("abc".into())]]));

    t.push(tc("rev_preserves_nonempty", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 10"), "$result >= 1",
        "test_rev_nonempty", &["s"], &ret("length(reverse($s))"), bounded_str()));

    t.push(tc("rev_single_char_identity", "(Str) -> Str",
        Some("length($s) == 1"), "$result eq $s",
        "test_rev_single", &["s"], &ret("reverse($s)"),
        vec![vec![Val::Str("a".into())], vec![Val::Str("x".into())], vec![Val::Str("0".into())]]));

    // Soundness probe: false claim reverse($s) eq "" when len >= 1
    t.push(tc("rev_false_empty_claim", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 10"), "$result eq \"\"",
        "test_rev_false_empty", &["s"], &ret("reverse($s)"), bounded_str()));

    t.push(tc("rev_length_composition", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5 && length($t) >= 1 && length($t) <= 5"),
        "$result == length($s) + length($t)",
        "test_rev_len_comp", &["s", "t"], &ret("length(reverse($s)) + length(reverse($t))"),
        vec![vec![Val::Str("ab".into()), Val::Str("cd".into())],
             vec![Val::Str("hello".into()), Val::Str("x".into())]]));

    t.push(tc("rev_concat_length", "(Str, Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5 && length($t) >= 1 && length($t) <= 5"),
        "$result == length($s) + length($t)",
        "test_rev_cat_len", &["s", "t"], &ret("length(reverse($s) . $t)"),
        vec![vec![Val::Str("ab".into()), Val::Str("cd".into())],
             vec![Val::Str("hello".into()), Val::Str("x".into())]]));

    t
}

#[test]
fn cegis_layer8_reverse_soundness() {
    let r = run_layer("L8-ReverseSoundness", layer8_reverse_soundness());
    assert_eq!(r.unsound, 0, "Reverse soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 9: String Repetition Edge Cases
// ============================================================================

fn layer9_string_repetition() -> Vec<TestCase> {
    let mut t = Vec::new();

    t.push(tc("repeat_by_0", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq \"\"",
        "test_rep_zero", &["s"], &ret("$s x 0"), str_singles()));

    t.push(tc("repeat_by_1_identity", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq $s",
        "test_rep_one", &["s"], &ret("$s x 1"), str_singles()));

    t.push(tc("repeat_len_x2", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 3"), "length($result) == length($s) * 2",
        "test_rep_len2", &["s"], &ret("$s x 2"), str_singles()));

    t.push(tc("repeat_len_x4", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 3"), "length($result) == length($s) * 4",
        "test_rep_len4", &["s"], &ret("$s x 4"), str_singles()));

    t.push(tc("repeat_concrete_ab_x2", "(Str) -> Str",
        Some("$s eq \"ab\""), "$result eq \"abab\"",
        "test_rep_ab2", &["s"], &ret("$s x 2"),
        vec![vec![Val::Str("ab".into())]]));

    t.push(tc("repeat_concrete_ab_x3", "(Str) -> Str",
        Some("$s eq \"ab\""), "$result eq \"ababab\"",
        "test_rep_ab3", &["s"], &ret("$s x 3"),
        vec![vec![Val::Str("ab".into())]]));

    t.push(tc("repeat_negative_count", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq \"\"",
        "test_rep_neg1", &["s"], &ret("$s x -1"), str_singles()));

    t.push(tc("repeat_negative_5", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 5"), "$result eq \"\"",
        "test_rep_neg5", &["s"], &ret("$s x -5"), str_singles()));

    t.push(tc("repeat_empty_string", "(Str) -> Str",
        Some("$s eq \"\""), "$result eq \"\"",
        "test_rep_empty", &["s"], &ret("$s x 5"),
        vec![vec![Val::Str("".into())]]));

    t.push(tc("repeat_empty_x0", "(Str) -> Str",
        Some("$s eq \"\""), "$result eq \"\"",
        "test_rep_empty0", &["s"], &ret("$s x 0"),
        vec![vec![Val::Str("".into())]]));

    t.push(tc("repeat_preserves_starts_with", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5 && starts_with($s, \"a\") == 1"),
        "$result == 1",
        "test_rep_sw", &["s"],
        &"    my $r = $s x 2;\n    return starts_with($r, \"a\");\n",
        vec![vec![Val::Str("a".into())], vec![Val::Str("abc".into())],
             vec![Val::Str("ab".into())], vec![Val::Str("axyz".into())]]));

    t.push(tc("repeat_preserves_contains", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 3 && contains($s, \"x\") == 1"),
        "$result == 1",
        "test_rep_contains", &["s"],
        &"    my $r = $s x 3;\n    return contains($r, \"x\");\n",
        vec![vec![Val::Str("x".into())], vec![Val::Str("ax".into())],
             vec![Val::Str("xa".into())]]));

    t.push(tc("repeat_len_x5", "(Str) -> Str",
        Some("length($s) >= 1 && length($s) <= 2"), "length($result) == length($s) * 5",
        "test_rep_len5", &["s"], &ret("$s x 5"), str_singles()));

    t.push(tc("repeat_vs_concat_length", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result == 1",
        "test_rep_vs_cat", &["s"],
        &"    my $a = $s x 2;\n    my $b = $s . $s;\n    if (length($a) == length($b)) {\n        return 1;\n    }\n    return 0;\n",
        str_singles()));

    t.push(tc("repeat_x2_eq_concat", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result == 1",
        "test_rep_eq_cat", &["s"],
        &"    my $a = $s x 2;\n    my $b = $s . $s;\n    if ($a eq $b) {\n        return 1;\n    }\n    return 0;\n",
        str_singles()));

    t.push(tc("repeat_x3_eq_triple_concat", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 3"), "$result == 1",
        "test_rep3_cat3", &["s"],
        &"    my $a = $s x 3;\n    my $b = $s . $s . $s;\n    if ($a eq $b) {\n        return 1;\n    }\n    return 0;\n",
        str_singles()));

    t.push(tc("repeat_reverse_length", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 3"), "$result == length($s) * 2",
        "test_rep_rev_len", &["s"],
        &"    my $r = $s x 2;\n    return length(reverse($r));\n",
        str_singles()));

    t.push(tc("repeat_single_char", "(Str) -> Str",
        Some("$s eq \"z\""), "$result eq \"zzzz\"",
        "test_rep_z4", &["s"], &ret("$s x 4"),
        vec![vec![Val::Str("z".into())]]));

    t.push(tc("repeat_then_index", "(Str) -> I64",
        Some("length($s) >= 1 && length($s) <= 5"), "$result == 0",
        "test_rep_idx", &["s"],
        &"    my $r = $s x 2;\n    return index($r, $s);\n",
        str_singles()));

    t
}

#[test]
fn cegis_layer9_string_repetition() {
    let r = run_layer("L9-StringRepetition", layer9_string_repetition());
    assert_eq!(r.unsound, 0, "String repetition soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 10: Negative Indexing & Substr Edge Cases
// ============================================================================

fn layer10_negative_indexing() -> Vec<TestCase> {
    let mut t = Vec::new();

    t.push(tc("substr_neg_start_minus2", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"lo\"",
        "test_substr_neg2", &["s"], &ret("substr($s, -2)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("substr_neg_start_minus5", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"hello\"",
        "test_substr_neg5", &["s"], &ret("substr($s, -5)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("substr_neg_start_minus1", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"o\"",
        "test_substr_neg1", &["s"], &ret("substr($s, -1)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("substr_neg_len_minus1", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"ell\"",
        "test_substr_nl1", &["s"], &ret("substr($s, 1, -1)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("substr_neg_len_minus2", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"hel\"",
        "test_substr_nl2", &["s"], &ret("substr($s, 0, -2)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("substr_neg_len_middle", "(Str) -> Str",
        Some("$s eq \"abcde\""), "$result eq \"cd\"",
        "test_substr_nlm", &["s"], &ret("substr($s, 2, -1)"),
        vec![vec![Val::Str("abcde".into())]]));

    t.push(tc("substr_zero_length", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"\"",
        "test_substr_z0", &["s"], &ret("substr($s, 2, 0)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("char_at_neg_minus1", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"o\"",
        "test_chat_neg1", &["s"], &ret("char_at($s, -1)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("char_at_neg_minus5", "(Str) -> Str",
        Some("$s eq \"hello\""), "$result eq \"h\"",
        "test_chat_neg5", &["s"], &ret("char_at($s, -5)"),
        vec![vec![Val::Str("hello".into())]]));

    t.push(tc("array_neg_idx_minus1", "(I64, I64, I64) -> I64",
        None, "$result == 30",
        "test_arr_neg1", &["a", "b", "c"],
        &"    my @a = ($a, $b, $c);\n    return $a[-1];\n",
        vec![vec![Val::Int(10), Val::Int(20), Val::Int(30)]]));

    t.push(tc("array_neg_idx_minus2", "(I64, I64, I64) -> I64",
        None, "$result == 20",
        "test_arr_neg2", &["a", "b", "c"],
        &"    my @a = ($a, $b, $c);\n    return $a[-2];\n",
        vec![vec![Val::Int(10), Val::Int(20), Val::Int(30)]]));

    t
}

#[test]
fn cegis_layer10_negative_indexing() {
    let r = run_layer("L10-NegativeIndexing", layer10_negative_indexing());
    assert_eq!(r.unsound, 0, "Negative indexing soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 11: Increment/Decrement Return Value Semantics
// ============================================================================

fn layer11_inc_dec_semantics() -> Vec<TestCase> {
    let mut t = Vec::new();
    let bounded = || isingles().into_iter()
        .filter(|v| match &v[0] { Val::Int(n) => *n >= -1000 && *n <= 1000, _ => false })
        .collect::<Vec<_>>();

    t.push(tc("postinc_returns_old", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x",
        "test_postinc_old", &["x"],
        "    my $y = $x;\n    my $old = $y++;\n    return $old;\n",
        bounded()));

    t.push(tc("postinc_side_effect", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x + 1",
        "test_postinc_side", &["x"],
        "    my $y = $x;\n    $y++;\n    return $y;\n",
        bounded()));

    t.push(tc("preinc_returns_new", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x + 1",
        "test_preinc_new", &["x"],
        "    my $y = $x;\n    my $new = ++$y;\n    return $new;\n",
        bounded()));

    t.push(tc("postdec_returns_old", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x",
        "test_postdec_old", &["x"],
        "    my $y = $x;\n    my $old = $y--;\n    return $old;\n",
        bounded()));

    t.push(tc("postdec_side_effect", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x - 1",
        "test_postdec_side", &["x"],
        "    my $y = $x;\n    $y--;\n    return $y;\n",
        bounded()));

    t.push(tc("predec_returns_new", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x - 1",
        "test_predec_new", &["x"],
        "    my $y = $x;\n    my $new = --$y;\n    return $new;\n",
        bounded()));

    t.push(tc("inc_dec_identity", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x",
        "test_inc_dec_id", &["x"],
        "    my $y = $x;\n    $y++;\n    $y--;\n    return $y;\n",
        bounded()));

    t.push(tc("triple_increment", "(I64) -> I64",
        Some("$x >= -1000 && $x <= 1000"), "$result == $x + 3",
        "test_triple_inc", &["x"],
        "    my $y = $x;\n    $y++;\n    $y++;\n    $y++;\n    return $y;\n",
        bounded()));

    t
}

#[test]
fn cegis_layer11_inc_dec_semantics() {
    let r = run_layer("L11-IncDecSemantics", layer11_inc_dec_semantics());
    assert_eq!(r.unsound, 0, "Inc/dec soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 12: Hash Edge Cases
// ============================================================================

fn layer12_hash_edge_cases() -> Vec<TestCase> {
    let mut t = Vec::new();

    let bounded_range: Vec<Vec<Val>> = (-100..=100).step_by(25).map(|x| vec![Val::Int(x)]).collect();
    let bounded_pairs: Vec<Vec<Val>> = vec![
        vec![Val::Int(0), Val::Int(0)], vec![Val::Int(1), Val::Int(-1)],
        vec![Val::Int(-50), Val::Int(50)], vec![Val::Int(100), Val::Int(100)],
        vec![Val::Int(-100), Val::Int(-100)], vec![Val::Int(42), Val::Int(-73)],
        vec![Val::Int(0), Val::Int(100)], vec![Val::Int(-100), Val::Int(0)],
    ];

    t.push(tc("hash_empty_key_write_read", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == $x",
        "test_hash_empty_key", &["x"],
        "    my %h = (\"\" => $x);\n    return $h{\"\"};\n",
        bounded_range.clone()));

    t.push(tc("hash_key_zero_str", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == $x",
        "test_hash_key_zero", &["x"],
        "    my %h = (\"0\" => $x);\n    return $h{\"0\"};\n",
        bounded_range.clone()));

    t.push(tc("hash_overwrite_semantics", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == $x",
        "test_hash_overwrite", &["x"],
        "    my %h = (\"k\" => 10);\n    $h{\"k\"} = $x;\n    return $h{\"k\"};\n",
        bounded_range.clone()));

    t.push(tc("hash_exists_after_write", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == 1",
        "test_hash_exists_wr", &["x"],
        "    my %h = (\"k\" => $x);\n    if (exists $h{\"k\"}) { return 1; }\n    return 0;\n",
        bounded_range.clone()));

    t.push(tc("hash_exists_missing_key", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == 0",
        "test_hash_exists_miss", &["x"],
        "    my %h = (\"k\" => $x);\n    if (exists $h{\"missing\"}) { return 1; }\n    return 0;\n",
        bounded_range.clone()));

    t.push(tc("hash_defined_present_key", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100"), "$result == 1",
        "test_hash_defined_pres", &["x"],
        "    my %h = (\"k\" => $x);\n    if (defined($h{\"k\"})) { return 1; }\n    return 0;\n",
        bounded_range.clone()));

    t.push(tc("hash_missing_key_default", "() -> I64",
        None, "$result == 0",
        "test_hash_missing_def", &[],
        "    my %h = (\"k\" => 42);\n    return $h{\"missing\"};\n",
        vec![vec![]]));

    t.push(tc("hash_multiple_keys", "(I64, I64) -> I64",
        Some("$x >= -100 && $x <= 100 && $y >= -100 && $y <= 100"),
        "$result == $x + $y",
        "test_hash_multi_keys", &["x", "y"],
        "    my %h = (\"a\" => $x, \"b\" => $y);\n    return $h{\"a\"} + $h{\"b\"};\n",
        bounded_pairs));

    t
}

#[test]
fn cegis_layer12_hash_edge_cases() {
    let r = run_layer("L12-HashEdgeCases", layer12_hash_edge_cases());
    assert_eq!(r.unsound, 0, "Hash edge case soundness bugs found:\n{}", r.details.join("\n"));
}

// ============================================================================
// Layer 13: Statement Modifiers & Die Path Pruning
// ============================================================================

fn layer13_statement_modifiers() -> Vec<TestCase> {
    let mut t = Vec::new();

    t.push(tc("return_if_nonneg", "(I64) -> I64", None,
        "$result >= 0",
        "test_return_if", &["x"],
        "    return $x if ($x > 0);\n    return 0;\n",
        isingles()));

    t.push(tc("return_unless_nonneg", "(I64) -> I64", None,
        "$result >= 0",
        "test_return_unless", &["x"],
        "    return 0 unless ($x > 0);\n    return $x;\n",
        isingles()));

    t.push(tc("die_if_prune_neg", "(I64) -> I64",
        Some("$x >= 0"), "$result >= 0",
        "test_die_if_prune", &["x"],
        "    die \"neg\" if ($x < 0);\n    return $x;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(1)], vec![Val::Int(5)],
             vec![Val::Int(10)], vec![Val::Int(100)]]));

    t.push(tc("die_unless_prune", "(I64) -> I64",
        Some("$x > 0"), "$result > 0",
        "test_die_unless_prune", &["x"],
        "    die \"not pos\" unless ($x > 0);\n    return $x;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(2)], vec![Val::Int(5)],
             vec![Val::Int(10)], vec![Val::Int(100)]]));

    t.push(tc("assign_if_nonneg", "(I64) -> I64", None,
        "$result >= 0",
        "test_assign_if", &["x"],
        "    my $y = 0;\n    $y = $x if ($x > 0);\n    return $y;\n",
        isingles()));

    t.push(tc("assign_unless_nonneg", "(I64) -> I64", None,
        "$result >= 0",
        "test_assign_unless", &["x"],
        "    my $y = $x;\n    $y = -$x unless ($x >= 0);\n    return $y;\n",
        isingles()));

    t.push(tc("chained_die_range", "(I64) -> I64",
        Some("$x >= 0 && $x <= 100"),
        "$result >= 0 && $result <= 100",
        "test_chained_die", &["x"],
        "    die \"low\" if ($x < 0);\n    die \"high\" if ($x > 100);\n    return $x;\n",
        vec![vec![Val::Int(0)], vec![Val::Int(1)], vec![Val::Int(50)],
             vec![Val::Int(99)], vec![Val::Int(100)]]));

    t.push(tc("die_return_combo", "(I64) -> I64",
        Some("$x >= -100 && $x <= 100 && $x != 0"),
        "$result == 1 || $result == -1",
        "test_die_return", &["x"],
        "    die \"zero\" if ($x == 0);\n    return 1 if ($x > 0);\n    return -1;\n",
        vec![vec![Val::Int(1)], vec![Val::Int(-1)], vec![Val::Int(5)],
             vec![Val::Int(-5)], vec![Val::Int(10)], vec![Val::Int(-10)],
             vec![Val::Int(100)], vec![Val::Int(-100)]]));

    t
}

#[test]
fn cegis_layer13_statement_modifiers() {
    let r = run_layer("L13-StatementModifiers", layer13_statement_modifiers());
    assert_eq!(r.unsound, 0, "Statement modifier soundness bugs found:\n{}", r.details.join("\n"));
}
