use std::{cell::RefCell, collections::BTreeMap, str::FromStr, sync::atomic::{AtomicUsize, Ordering}};

use thiserror::Error;
use tracing::debug;
use z3::{
    Context,
    Params,
    Sort,
    Solver,
    ast::{Ast, Array, BV, Bool, Int, Real, String as Z3String},
};

use crate::{
    ast::Type,
    limits::DEFAULT_SOLVER_TIMEOUT_MS,
    symexec::{
        ArrayIntExpr, ArrayStrExpr, BoolExpr, CmpOp, HashIntExpr, HashStrExpr, IntExpr,
        ModelValue, StrExpr,
    },
};

static REVERSE_COUNTER: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static REVERSE_AXIOMS: RefCell<Vec<Bool>> = RefCell::new(Vec::new());
}

pub const MAX_STR_LEN: i64 = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelVar {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Error)]
pub enum SmtError {
    #[error("solver returned unknown for function `{function}`: {reason}")]
    Unknown { function: String, reason: String },
}

pub fn is_satisfiable(function: &str, condition: &BoolExpr) -> std::result::Result<bool, SmtError> {
    is_satisfiable_with_timeout(function, condition, DEFAULT_SOLVER_TIMEOUT_MS)
}

pub fn is_satisfiable_with_timeout(function: &str, condition: &BoolExpr, timeout_ms: u32) -> std::result::Result<bool, SmtError> {
    let solver = Solver::new();
    apply_solver_timeout(&solver, timeout_ms);
    assert_string_bounds(&solver, condition);
    reset_reverse_axioms();
    solver.assert(&encode_semantic_safety(condition));
    solver.assert(&encode_bool(condition));
    assert_reverse_axioms(&solver);
    let result = solver.check();
    debug!(function, ?result, "checked satisfiability");

    Ok(match result {
        z3::SatResult::Sat => true,
        z3::SatResult::Unsat => false,
        z3::SatResult::Unknown => {
            let reason = solver
                .get_reason_unknown()
                .unwrap_or_else(|| "unknown".to_string());
            return Err(SmtError::Unknown {
                function: function.to_string(),
                reason,
            });
        }
    })
}

pub fn find_model(
    function: &str,
    condition: &BoolExpr,
    variables: &[ModelVar],
) -> std::result::Result<Option<BTreeMap<String, ModelValue>>, SmtError> {
    find_model_with_timeout(function, condition, variables, DEFAULT_SOLVER_TIMEOUT_MS)
}

pub fn find_model_with_timeout(
    function: &str,
    condition: &BoolExpr,
    variables: &[ModelVar],
    timeout_ms: u32,
) -> std::result::Result<Option<BTreeMap<String, ModelValue>>, SmtError> {
    let solver = Solver::new();
    apply_solver_timeout(&solver, timeout_ms);
    assert_string_bounds(&solver, condition);
    reset_reverse_axioms();
    solver.assert(&encode_semantic_safety(condition));
    solver.assert(&encode_bool(condition));
    assert_reverse_axioms(&solver);
    let result = solver.check();
    debug!(function, ?result, "checked counterexample query");

    match result {
        z3::SatResult::Unsat => Ok(None),
        z3::SatResult::Unknown => Err(SmtError::Unknown {
            function: function.to_string(),
            reason: solver
                .get_reason_unknown()
                .unwrap_or_else(|| "unknown".to_string()),
        }),
        z3::SatResult::Sat => {
            let model = solver
                .get_model()
                .expect("model must exist for satisfiable query");
            let mut assignments = BTreeMap::new();
            for variable in variables {
                let value = match variable.ty {
                    Type::I64 => {
                        let symbol = BV::new_const(variable.name.clone(), 64);
                        match model
                            .eval(&symbol, true)
                            .and_then(|value| value.as_u64())
                        {
                            Some(value) => ModelValue::Int(value as i64),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    Type::Str => {
                        let symbol = Z3String::new_const(variable.name.clone());
                        match model
                            .eval(&symbol, true)
                            .and_then(|value| value.as_string())
                        {
                            Some(value) => ModelValue::Str(value),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    Type::ArrayI64 => {
                        let symbol = Array::new_const(variable.name.clone(), &Sort::bitvector(64), &Sort::bitvector(64));
                        match model.eval(&symbol, true) {
                            Some(value) => ModelValue::Collection(value.to_string()),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    Type::ArrayStr => {
                        let symbol =
                            Array::new_const(variable.name.clone(), &Sort::bitvector(64), &Sort::string());
                        match model.eval(&symbol, true) {
                            Some(value) => ModelValue::Collection(value.to_string()),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    Type::HashI64 => {
                        let symbol =
                            Array::new_const(variable.name.clone(), &Sort::string(), &Sort::bitvector(64));
                        match model.eval(&symbol, true) {
                            Some(value) => ModelValue::Collection(value.to_string()),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    Type::HashStr => {
                        let symbol = Array::new_const(
                            variable.name.clone(),
                            &Sort::string(),
                            &Sort::string(),
                        );
                        match model.eval(&symbol, true) {
                            Some(value) => ModelValue::Collection(value.to_string()),
                            None => ModelValue::Unconstrained,
                        }
                    }
                    // References are desugared before SMT; these should never appear.
                    Type::RefI64 | Type::RefStr
                    | Type::RefArrayI64 | Type::RefArrayStr
                    | Type::RefHashI64 | Type::RefHashStr => {
                        ModelValue::Unconstrained
                    }
                };
                assignments.insert(variable.name.clone(), value);
            }
            Ok(Some(assignments))
        }
    }
}

fn reset_reverse_axioms() {
    REVERSE_AXIOMS.with(|axioms| axioms.borrow_mut().clear());
}

fn assert_reverse_axioms(solver: &Solver) {
    REVERSE_AXIOMS.with(|axioms| {
        for axiom in axioms.borrow().iter() {
            solver.assert(axiom);
        }
    });
}

fn assert_string_bounds(solver: &Solver, condition: &BoolExpr) {
    let user_bounds = extract_length_bounds_from_bool(condition);
    // Soundness: the bound must be at least as large as the longest string
    // constant in the formula.  Otherwise the solver cannot construct inputs
    // that match those constants, causing false "verified" results.
    let min_bound = max_string_const_len_bool(condition).max(MAX_STR_LEN);
    for variable in collect_string_vars_from_bool(condition) {
        let bound = user_bounds
            .get(&variable)
            .copied()
            .map(|ub| ub.max(min_bound))
            .unwrap_or(min_bound);
        let symbol = Z3String::new_const(variable);
        solver.assert(&symbol.length().le(bound));
    }
}

/// Extract explicit upper bounds on string lengths from the condition.
/// Looks for patterns like `length(Var(name)) <= N` (and symmetric forms)
/// within conjunctions to find user-specified length constraints.
fn extract_length_bounds_from_bool(expr: &BoolExpr) -> BTreeMap<String, i64> {
    let mut bounds = BTreeMap::new();
    extract_length_bounds_inner(expr, &mut bounds);
    bounds
}

fn extract_length_bounds_inner(expr: &BoolExpr, bounds: &mut BTreeMap<String, i64>) {
    match expr {
        BoolExpr::And(left, right) => {
            extract_length_bounds_inner(left, bounds);
            extract_length_bounds_inner(right, bounds);
        }
        BoolExpr::IntCmp(op, left, right) => {
            // Check for: Length(Var(name)) <op> Const(n)
            if let Some((name, bound)) = extract_length_upper_bound(op, left, right) {
                let entry = bounds.entry(name).or_insert(0);
                if bound > *entry {
                    *entry = bound;
                }
            }
            // Check symmetric: Const(n) <op> Length(Var(name))
            if let Some(flipped) = flip_cmp_op(op) {
                if let Some((name, bound)) = extract_length_upper_bound(&flipped, right, left) {
                    let entry = bounds.entry(name).or_insert(0);
                    if bound > *entry {
                        *entry = bound;
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_length_upper_bound(op: &CmpOp, left: &IntExpr, right: &IntExpr) -> Option<(String, i64)> {
    // Pattern: Length(Var(name)) <= Const(n) or Length(Var(name)) < Const(n)
    if let IntExpr::Length(str_expr) = left {
        if let StrExpr::Var(name) = str_expr.as_ref() {
            if let IntExpr::Const(n) = right {
                match op {
                    CmpOp::Le => return Some((name.clone(), *n)),
                    CmpOp::Lt => return Some((name.clone(), *n - 1)),
                    CmpOp::Eq => return Some((name.clone(), *n)),
                    _ => {}
                }
            }
        }
    }
    None
}

fn flip_cmp_op(op: &CmpOp) -> Option<CmpOp> {
    match op {
        CmpOp::Lt => Some(CmpOp::Gt),
        CmpOp::Le => Some(CmpOp::Ge),
        CmpOp::Gt => Some(CmpOp::Lt),
        CmpOp::Ge => Some(CmpOp::Le),
        CmpOp::Eq => Some(CmpOp::Eq),
        CmpOp::Ne => Some(CmpOp::Ne),
    }
}

fn apply_solver_timeout(solver: &Solver, timeout_ms: u32) {
    let mut params = Params::new();
    params.set_u32("timeout", timeout_ms);
    solver.set_params(&params);
}

fn bv_const(value: i64) -> BV {
    BV::from_i64(value, 64)
}

fn int_to_bv(i: &Int) -> BV {
    BV::from_int(i, 64)
}

fn int_to_bv_via_axiom(i: &Int, nonneg: bool) -> BV {
    let n = REVERSE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let fresh = BV::new_const(format!("__i2bv_{n}"), 64);
    let axiom = fresh.to_int(true).eq(i);
    REVERSE_AXIOMS.with(|axioms| {
        let mut ax = axioms.borrow_mut();
        ax.push(axiom);
        if nonneg {
            ax.push(fresh.bvsge(&bv_const(0)));
        }
    });
    fresh
}

fn encode_int(expr: &IntExpr) -> BV {
    match expr {
        IntExpr::Const(value) => bv_const(*value),
        IntExpr::Var(name) => BV::new_const(name.clone(), 64),
        IntExpr::Add(left, right) => encode_int(left).bvadd(&encode_int(right)),
        IntExpr::Sub(left, right) => encode_int(left).bvsub(&encode_int(right)),
        IntExpr::Mul(left, right) => encode_int(left).bvmul(&encode_int(right)),
        IntExpr::Pow(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            let exp_is_zero = Ast::eq(&r, &bv_const(0));
            let l_int = l.to_int(true);
            let r_int = r.to_int(true);
            let real_result = l_int.power(&r_int);
            let zero_real = Real::from_rational(0, 1);
            let is_nonneg = real_result.ge(&zero_real);
            let floor_val = real_result.to_int();
            let neg_floor = real_result.unary_minus().to_int();
            let normal_int = is_nonneg.ite(&floor_val, &neg_floor.unary_minus());
            let normal_bv = int_to_bv(&normal_int);
            exp_is_zero.ite(&bv_const(1), &normal_bv)
        }
        IntExpr::Div(left, right) => encode_int(left).bvsdiv(&encode_int(right)),
        IntExpr::Mod(left, right) => {
            // Perl's % uses floor-modulo (result has sign of divisor).
            // Z3's bvsmod implements exactly this semantics.
            encode_int(left).bvsmod(&encode_int(right))
        }
        IntExpr::BitAnd(left, right) => encode_int(left).bvand(&encode_int(right)),
        IntExpr::BitOr(left, right) => encode_int(left).bvor(&encode_int(right)),
        IntExpr::BitXor(left, right) => encode_int(left).bvxor(&encode_int(right)),
        IntExpr::Shl(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            let zero = bv_const(0);
            let sixty_four = bv_const(64);
            let r_nonneg = r.bvsge(&zero);
            let r_abs = r_nonneg.ite(&r, &r.bvneg());
            let r_too_large = r_abs.bvsge(&sixty_four);
            let shl_result = l.bvshl(&r_abs);
            let shr_result = l.bvlshr(&r_abs);
            let normal_result = r_nonneg.ite(&shl_result, &shr_result);
            r_too_large.ite(&zero, &normal_result)
        }
        IntExpr::Shr(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            let zero = bv_const(0);
            let sixty_four = bv_const(64);
            let r_nonneg = r.bvsge(&zero);
            let r_abs = r_nonneg.ite(&r, &r.bvneg());
            let r_too_large = r_abs.bvsge(&sixty_four);
            let shr_result = l.bvlshr(&r_abs);
            let shl_result = l.bvshl(&r_abs);
            let normal_result = r_nonneg.ite(&shr_result, &shl_result);
            r_too_large.ite(&zero, &normal_result)
        }
        IntExpr::BitNot(value) => encode_int(value).bvnot(),
        IntExpr::Abs(value) => {
            let encoded = encode_int(value);
            let zero = bv_const(0);
            encoded.bvsge(&zero).ite(&encoded, &encoded.bvneg())
        }
        IntExpr::Ord(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_str(value);
            let is_empty = encoded.length().eq(Int::from_i64(0));
            let first_char = unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_seq_at(ctx.get_z3_context(), encoded.get_z3_ast(), Int::from_i64(0).get_z3_ast()).unwrap())
            };
            let code_point = unsafe {
                Int::wrap(ctx, z3_sys::Z3_mk_string_to_code(ctx.get_z3_context(), first_char.get_z3_ast()).unwrap())
            };
            let result_int = is_empty.ite(&Int::from_i64(0), &code_point);
            int_to_bv_via_axiom(&result_int, true)
        }
        IntExpr::StrToInt(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_str(value);
            let minus_sign = Z3String::from_str("-").expect("minus literal");
            let one = Int::from_i64(1);
            let zero = Int::from_i64(0);
            let neg_one = Int::from_i64(-1);
            let starts_with_minus = unsafe {
                Bool::wrap(ctx, z3_sys::Z3_mk_seq_prefix(
                    ctx.get_z3_context(),
                    minus_sign.get_z3_ast(),
                    encoded.get_z3_ast(),
                ).unwrap())
            };
            let n = REVERSE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let fresh_name = format!("__strtoint_fallback_{n}");
            let fallback = Int::new_const(fresh_name);
            let raw_pos_val = unsafe {
                Int::wrap(ctx, z3_sys::Z3_mk_str_to_int(
                    ctx.get_z3_context(),
                    encoded.get_z3_ast(),
                ).unwrap())
            };
            let is_empty = encoded.length().eq(Int::from_i64(0));
            let pos_val = raw_pos_val.eq(&neg_one).ite(
                &is_empty.ite(&zero, &fallback),
                &raw_pos_val,
            );
            let tail = encoded.substr(one, encoded.length());
            let raw_tail_val = unsafe {
                Int::wrap(ctx, z3_sys::Z3_mk_str_to_int(
                    ctx.get_z3_context(),
                    tail.get_z3_ast(),
                ).unwrap())
            };
            let tail_is_empty = tail.length().eq(Int::from_i64(0));
            let neg_val = raw_tail_val.eq(&neg_one).ite(
                &tail_is_empty.ite(&zero, &fallback),
                &raw_tail_val.unary_minus(),
            );
            let result_int = starts_with_minus.ite(&neg_val, &pos_val);
            int_to_bv_via_axiom(&result_int, false)
        }
        IntExpr::Contains(haystack, needle) => {
            let ctx = &Context::thread_local();
            let h = encode_str(haystack);
            let n = encode_str(needle);
            let contains_bool = unsafe {
                Bool::wrap(ctx, z3_sys::Z3_mk_seq_contains(ctx.get_z3_context(), h.get_z3_ast(), n.get_z3_ast()).unwrap())
            };
            contains_bool.ite(&bv_const(1), &bv_const(0))
        }
        IntExpr::StartsWith(string, prefix) => {
            let ctx = &Context::thread_local();
            let s = encode_str(string);
            let p = encode_str(prefix);
            let prefix_bool = unsafe {
                Bool::wrap(ctx, z3_sys::Z3_mk_seq_prefix(ctx.get_z3_context(), p.get_z3_ast(), s.get_z3_ast()).unwrap())
            };
            prefix_bool.ite(&bv_const(1), &bv_const(0))
        }
        IntExpr::EndsWith(string, suffix) => {
            let ctx = &Context::thread_local();
            let s = encode_str(string);
            let sfx = encode_str(suffix);
            let suffix_bool = unsafe {
                Bool::wrap(ctx, z3_sys::Z3_mk_seq_suffix(ctx.get_z3_context(), sfx.get_z3_ast(), s.get_z3_ast()).unwrap())
            };
            suffix_bool.ite(&bv_const(1), &bv_const(0))
        }
        IntExpr::Ite(cond, then_int, else_int) => {
            let cond_bool = encode_bool(cond);
            let then_encoded = encode_int(then_int);
            let else_encoded = encode_int(else_int);
            cond_bool.ite(&then_encoded, &else_encoded)
        }
        IntExpr::Length(value) => int_to_bv_via_axiom(&encode_str(value).length(), true),
        IntExpr::Index(haystack, needle, start) => {
            let result_int = encode_index_of(&encode_str(haystack), &encode_str(needle), &encode_int(start).to_int(true));
            int_to_bv_via_axiom(&result_int, false)
        }
        IntExpr::Chomp(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_str(value);
            let newline = Z3String::from_str("\n").expect("newline literal");
            let has_newline = unsafe {
                Bool::wrap(ctx, z3_sys::Z3_mk_seq_suffix(ctx.get_z3_context(), newline.get_z3_ast(), encoded.get_z3_ast()).unwrap())
            };
            has_newline.ite(&bv_const(1), &bv_const(0))
        }
        IntExpr::ArraySelect(array, index) => encode_array_int(array)
            .select(&encode_int(index))
            .as_bv()
            .expect("array select should produce BV"),
        IntExpr::HashSelect(hash, key) => encode_hash_int(hash)
            .select(&encode_str(key))
            .as_bv()
            .expect("hash select should produce BV"),
    }
}

fn encode_str(expr: &StrExpr) -> Z3String {
    match expr {
        StrExpr::Const(value) => {
            Z3String::from_str(value).expect("string literals must not contain NUL")
        }
        StrExpr::Var(name) => Z3String::new_const(name.clone()),
        StrExpr::Concat(left, right) => Z3String::concat(&[encode_str(left), encode_str(right)]),
        StrExpr::Substr(value, start, len) => {
            let encoded_value = encode_str(value);
            let encoded_start = encode_int(start).to_int(true);
            let encoded_len = encode_int(len).to_int(true);
            let zero = Int::from_i64(0);
            let effective_start = encoded_start.ge(&zero).ite(
                &encoded_start,
                &Int::add(&[&encoded_value.length(), &encoded_start]),
            );
            encoded_value.substr(effective_start, encoded_len)
        }
        StrExpr::Chr(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_int(value).to_int(true);
            let replacement = Int::from_i64(65533);
            let max_code = Int::from_i64(0x7FFFFFFF);
            let zero = Int::from_i64(0);
            let in_range = Bool::and(&[&encoded.ge(&zero), &encoded.le(&max_code)]);
            let clamped = in_range.ite(&encoded, &replacement);
            unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_string_from_code(ctx.get_z3_context(), clamped.get_z3_ast()).unwrap())
            }
        }
        StrExpr::FromInt(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_int(value).to_int(true);
            let zero = Int::from_i64(0);
            let is_nonneg = encoded.ge(&zero);
            let pos_str = unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_int_to_str(ctx.get_z3_context(), encoded.get_z3_ast()).unwrap())
            };
            let neg_val = encoded.unary_minus();
            let neg_digits = unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_int_to_str(ctx.get_z3_context(), neg_val.get_z3_ast()).unwrap())
            };
            let minus_sign = Z3String::from_str("-").expect("minus literal");
            let neg_str = Z3String::concat(&[minus_sign, neg_digits]);
            is_nonneg.ite(&pos_str, &neg_str)
        }
        StrExpr::Reverse(value) => {
            let n = REVERSE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let fresh_name = format!("__reverse_{n}");
            let fresh_var = Z3String::new_const(fresh_name);
            let encoded_inner = encode_str(value);
            let len_axiom = fresh_var.length().eq(&encoded_inner.length());
            REVERSE_AXIOMS.with(|axioms| axioms.borrow_mut().push(len_axiom));
            fresh_var
        }
        StrExpr::Replace(string, from, to) => {
            let ctx = &Context::thread_local();
            let s = encode_str(string);
            let f = encode_str(from);
            let t = encode_str(to);
            let placeholder_chars: String = std::iter::repeat('\u{F8FF}').take((MAX_STR_LEN + 1) as usize).collect();
            let placeholder = Z3String::from_str(&placeholder_chars).expect("placeholder literal");
            let mut current = s;
            for _ in 0..2 * MAX_STR_LEN {
                current = unsafe {
                    let args = [current.get_z3_ast(), f.get_z3_ast(), placeholder.get_z3_ast()];
                    Z3String::wrap(ctx, z3_sys::Z3_mk_seq_replace(ctx.get_z3_context(), args[0], args[1], args[2]).unwrap())
                };
            }
            for _ in 0..2 * MAX_STR_LEN {
                current = unsafe {
                    let args = [current.get_z3_ast(), placeholder.get_z3_ast(), t.get_z3_ast()];
                    Z3String::wrap(ctx, z3_sys::Z3_mk_seq_replace(ctx.get_z3_context(), args[0], args[1], args[2]).unwrap())
                };
            }
            current
        }
        StrExpr::CharAt(string, index) => {
            let ctx = &Context::thread_local();
            let s = encode_str(string);
            let i = encode_int(index).to_int(true);
            let zero = Int::from_i64(0);
            let effective_i = i.ge(&zero).ite(&i, &Int::add(&[&s.length(), &i]));
            unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_seq_at(ctx.get_z3_context(), s.get_z3_ast(), effective_i.get_z3_ast()).unwrap())
            }
        }
        StrExpr::Ite(cond, then_str, else_str) => {
            let cond_bool = encode_bool(cond);
            let then_encoded = encode_str(then_str);
            let else_encoded = encode_str(else_str);
            cond_bool.ite(&then_encoded, &else_encoded)
        }
        StrExpr::ArraySelect(array, index) => encode_array_str(array)
            .select(&encode_int(index))
            .as_string()
            .expect("array select should produce String"),
        StrExpr::HashSelect(hash, key) => encode_hash_str(hash)
            .select(&encode_str(key))
            .as_string()
            .expect("hash select should produce String"),
    }
}

fn encode_array_int(expr: &ArrayIntExpr) -> Array {
    match expr {
        ArrayIntExpr::Var(name) => Array::new_const(name.clone(), &Sort::bitvector(64), &Sort::bitvector(64)),
        ArrayIntExpr::Store(base, index, value) => {
            encode_array_int(base).store(&encode_int(index), &encode_int(value))
        }
    }
}

fn encode_array_str(expr: &ArrayStrExpr) -> Array {
    match expr {
        ArrayStrExpr::Var(name) => Array::new_const(name.clone(), &Sort::bitvector(64), &Sort::string()),
        ArrayStrExpr::Store(base, index, value) => {
            encode_array_str(base).store(&encode_int(index), &encode_str(value))
        }
    }
}

fn encode_hash_int(expr: &HashIntExpr) -> Array {
    match expr {
        HashIntExpr::Var(name) => Array::new_const(name.clone(), &Sort::string(), &Sort::bitvector(64)),
        HashIntExpr::Store(base, key, value) => {
            encode_hash_int(base).store(&encode_str(key), &encode_int(value))
        }
    }
}

fn encode_hash_str(expr: &HashStrExpr) -> Array {
    match expr {
        HashStrExpr::Var(name) => Array::new_const(name.clone(), &Sort::string(), &Sort::string()),
        HashStrExpr::Store(base, key, value) => {
            encode_hash_str(base).store(&encode_str(key), &encode_str(value))
        }
    }
}

fn encode_index_of(haystack: &Z3String, needle: &Z3String, start: &Int) -> Int {
    // Use Z3's native str.indexof which correctly handles arbitrary-length
    // strings.  The previous hand-rolled loop only searched positions
    // 0..=MAX_STR_LEN, which was unsound for strings produced by
    // concatenation (whose length can exceed MAX_STR_LEN).
    //
    // Perl treats negative start positions as 0; Z3's str.indexof returns
    // -1 for negative offsets.  Clamp start to max(start, 0).
    //
    // Empty-needle fix: In Perl, index($s, "", $pos) returns
    // min($pos, length($s)) — the empty string is always "found".
    // Z3's str.indexof(s, "", pos) returns -1 when pos > length(s),
    // which diverges from Perl.  Guard: when needle is empty, return
    // min(clamped_start, length(haystack)).
    let ctx = &Context::thread_local();
    let zero = Int::from_i64(0);
    let clamped_start = start.ge(0).ite(start, &zero);
    let z3_result = unsafe {
        Int::wrap(ctx, z3_sys::Z3_mk_seq_index(
            ctx.get_z3_context(),
            haystack.get_z3_ast(),
            needle.get_z3_ast(),
            clamped_start.get_z3_ast(),
        ).unwrap())
    };
    // Detect empty needle: length(needle) == 0
    let needle_is_empty = needle.length().eq(Int::from_i64(0));
    // Perl result for empty needle: min(clamped_start, length(haystack))
    let hay_len = haystack.length();
    let start_le_len = clamped_start.le(&hay_len);
    let empty_needle_result = start_le_len.ite(&clamped_start, &hay_len);
    // Use the Perl-correct result when needle is empty
    needle_is_empty.ite(&empty_needle_result, &z3_result)
}

fn encode_bool(expr: &BoolExpr) -> Bool {
    match expr {
        BoolExpr::Const(value) => Bool::from_bool(*value),
        BoolExpr::Not(expr) => encode_bool(expr).not(),
        BoolExpr::And(left, right) => Bool::and(&[&encode_bool(left), &encode_bool(right)]),
        BoolExpr::Or(left, right) => Bool::or(&[&encode_bool(left), &encode_bool(right)]),
        BoolExpr::Overflow(exprs) => {
            if exprs.is_empty() {
                return Bool::from_bool(false);
            }
            let safety_parts: Vec<Bool> = exprs.iter().map(|e| encode_int_overflow_safety(e)).collect();
            let refs: Vec<&Bool> = safety_parts.iter().collect();
            let all_safe = Bool::and(&refs);
            all_safe.not()
        }
        BoolExpr::IntCmp(op, left, right) => {
            let left = encode_int(left);
            let right = encode_int(right);
            match op {
                crate::symexec::CmpOp::Lt => left.bvslt(&right),
                crate::symexec::CmpOp::Le => left.bvsle(&right),
                crate::symexec::CmpOp::Gt => left.bvsgt(&right),
                crate::symexec::CmpOp::Ge => left.bvsge(&right),
                crate::symexec::CmpOp::Eq => Ast::eq(&left, &right),
                crate::symexec::CmpOp::Ne => Ast::eq(&left, &right).not(),
            }
        }
        BoolExpr::StrEq(left, right) => encode_str(left).eq(encode_str(right)),
        BoolExpr::StrCmp(op, left, right) => {
            let ctx = &Context::thread_local();
            let left_encoded = encode_str(left);
            let right_encoded = encode_str(right);
            match op {
                crate::symexec::CmpOp::Lt => unsafe {
                    Bool::wrap(ctx, z3_sys::Z3_mk_str_lt(
                        ctx.get_z3_context(),
                        left_encoded.get_z3_ast(),
                        right_encoded.get_z3_ast(),
                    ).unwrap())
                },
                crate::symexec::CmpOp::Le => unsafe {
                    Bool::wrap(ctx, z3_sys::Z3_mk_str_le(
                        ctx.get_z3_context(),
                        left_encoded.get_z3_ast(),
                        right_encoded.get_z3_ast(),
                    ).unwrap())
                },
                crate::symexec::CmpOp::Gt => unsafe {
                    // a gt b  ≡  b lt a
                    Bool::wrap(ctx, z3_sys::Z3_mk_str_lt(
                        ctx.get_z3_context(),
                        right_encoded.get_z3_ast(),
                        left_encoded.get_z3_ast(),
                    ).unwrap())
                },
                crate::symexec::CmpOp::Ge => unsafe {
                    // a ge b  ≡  b le a
                    Bool::wrap(ctx, z3_sys::Z3_mk_str_le(
                        ctx.get_z3_context(),
                        right_encoded.get_z3_ast(),
                        left_encoded.get_z3_ast(),
                    ).unwrap())
                },
                crate::symexec::CmpOp::Eq => left_encoded.eq(&right_encoded),
                crate::symexec::CmpOp::Ne => left_encoded.eq(&right_encoded).not(),
            }
        }
    }
}

fn encode_bool_semantic_safety(expr: &BoolExpr) -> Bool {
    match expr {
        BoolExpr::Const(_) | BoolExpr::Overflow(_) => Bool::from_bool(true),
        BoolExpr::Not(expr) => encode_bool_semantic_safety(expr),
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            Bool::and(&[&encode_bool_semantic_safety(left), &encode_bool_semantic_safety(right)])
        }
        BoolExpr::IntCmp(_, left, right) => {
            Bool::and(&[&encode_int_semantic_safety(left), &encode_int_semantic_safety(right)])
        }
        BoolExpr::StrEq(left, right) | BoolExpr::StrCmp(_, left, right) => {
            Bool::and(&[&encode_str_semantic_safety(left), &encode_str_semantic_safety(right)])
        }
    }
}

fn encode_bool_overflow_safety(expr: &BoolExpr) -> Bool {
    match expr {
        BoolExpr::Const(_) | BoolExpr::Overflow(_) => Bool::from_bool(true),
        BoolExpr::Not(expr) => encode_bool_overflow_safety(expr),
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            Bool::and(&[&encode_bool_overflow_safety(left), &encode_bool_overflow_safety(right)])
        }
        BoolExpr::IntCmp(_, left, right) => {
            Bool::and(&[&encode_int_overflow_safety(left), &encode_int_overflow_safety(right)])
        }
        BoolExpr::StrEq(left, right) | BoolExpr::StrCmp(_, left, right) => {
            Bool::and(&[&encode_str_overflow_safety(left), &encode_str_overflow_safety(right)])
        }
    }
}

fn encode_int_semantic_safety(expr: &IntExpr) -> Bool {
    match expr {
        IntExpr::Const(_) | IntExpr::Var(_) => Bool::from_bool(true),
        IntExpr::Add(left, right) | IntExpr::Sub(left, right) | IntExpr::Mul(left, right) => {
            Bool::and(&[&encode_int_semantic_safety(left), &encode_int_semantic_safety(right)])
        }
        IntExpr::BitAnd(left, right)
        | IntExpr::BitOr(left, right)
        | IntExpr::BitXor(left, right) => {
            Bool::and(&[&encode_int_semantic_safety(left), &encode_int_semantic_safety(right)])
        }
        IntExpr::Shl(left, right) | IntExpr::Shr(left, right) => {
            let l = encode_int(left);
            Bool::and(&[
                &encode_int_semantic_safety(left),
                &encode_int_semantic_safety(right),
                &l.bvsge(&bv_const(0)),
            ])
        }
        IntExpr::Pow(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            let exp_nonneg = r.bvsge(&bv_const(0));
            let l_int = l.to_int(true);
            let r_int = r.to_int(true);
            let real_result = l_int.power(&r_int);
            let min_val = Real::from_rational(i64::MIN, 1);
            let max_val = Real::from_rational(i64::MAX, 1);
            let fits = Bool::and(&[&real_result.ge(&min_val), &real_result.le(&max_val)]);
            Bool::and(&[
                &encode_int_semantic_safety(left),
                &encode_int_semantic_safety(right),
                &exp_nonneg,
                &fits,
            ])
        }
        IntExpr::Div(left, right) => {
            let r = encode_int(right);
            let r_nonzero = Ast::eq(&r, &bv_const(0)).not();
            Bool::and(&[
                &encode_int_semantic_safety(left),
                &encode_int_semantic_safety(right),
                &r_nonzero,
            ])
        }
        IntExpr::Mod(left, right) => Bool::and(&[
            &encode_int_semantic_safety(left),
            &encode_int_semantic_safety(right),
            &Ast::eq(&encode_int(right), &bv_const(0)).not(),
        ]),
        IntExpr::BitNot(value) => encode_int_semantic_safety(value),
        IntExpr::Abs(value) => encode_int_semantic_safety(value),
        IntExpr::StrToInt(value) => encode_str_semantic_safety(value),
        IntExpr::Ord(value) => encode_str_semantic_safety(value),
        IntExpr::Contains(haystack, needle) => {
            Bool::and(&[&encode_str_semantic_safety(haystack), &encode_str_semantic_safety(needle)])
        }
        IntExpr::StartsWith(string, prefix) => {
            Bool::and(&[&encode_str_semantic_safety(string), &encode_str_semantic_safety(prefix)])
        }
        IntExpr::EndsWith(string, suffix) => {
            Bool::and(&[&encode_str_semantic_safety(string), &encode_str_semantic_safety(suffix)])
        }
        IntExpr::Chomp(value) => encode_str_semantic_safety(value),
        IntExpr::Ite(cond, then_int, else_int) => {
            let cond_encoded = encode_bool(cond);
            let then_safe = Bool::or(&[&cond_encoded.not(), &encode_int_semantic_safety(then_int)]);
            let else_safe = Bool::or(&[&cond_encoded, &encode_int_semantic_safety(else_int)]);
            Bool::and(&[&encode_bool_semantic_safety(cond), &then_safe, &else_safe])
        }
        IntExpr::Length(value) => encode_str_semantic_safety(value),
        IntExpr::Index(haystack, needle, start) => {
            Bool::and(&[
                &encode_str_semantic_safety(haystack),
                &encode_str_semantic_safety(needle),
                &encode_int_semantic_safety(start),
            ])
        }
        IntExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_int_semantic_safety(array),
            &encode_int_semantic_safety(index),
        ]),
        IntExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_int_semantic_safety(hash),
            &encode_str_semantic_safety(key),
        ]),
    }
}

fn encode_int_overflow_safety(expr: &IntExpr) -> Bool {
    match expr {
        IntExpr::Const(_) | IntExpr::Var(_) => Bool::from_bool(true),
        IntExpr::Add(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            Bool::and(&[
                &encode_int_overflow_safety(left),
                &encode_int_overflow_safety(right),
                &l.bvadd_no_overflow(&r, true),
                &l.bvadd_no_underflow(&r),
            ])
        }
        IntExpr::Sub(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            Bool::and(&[
                &encode_int_overflow_safety(left),
                &encode_int_overflow_safety(right),
                &l.bvsub_no_overflow(&r),
                &l.bvsub_no_underflow(&r, true),
            ])
        }
        IntExpr::Mul(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            Bool::and(&[
                &encode_int_overflow_safety(left),
                &encode_int_overflow_safety(right),
                &l.bvmul_no_overflow(&r, true),
                &l.bvmul_no_underflow(&r),
            ])
        }
        IntExpr::Div(left, right) => {
            let l = encode_int(left);
            let r = encode_int(right);
            let no_overflow = l.bvsdiv_no_overflow(&r);
            Bool::and(&[
                &encode_int_overflow_safety(left),
                &encode_int_overflow_safety(right),
                &no_overflow,
            ])
        }
        IntExpr::Mod(left, right) => Bool::and(&[
            &encode_int_overflow_safety(left),
            &encode_int_overflow_safety(right),
        ]),
        IntExpr::Pow(left, right) => {
            Bool::and(&[
                &encode_int_overflow_safety(left),
                &encode_int_overflow_safety(right),
            ])
        }
        IntExpr::BitAnd(left, right)
        | IntExpr::BitOr(left, right)
        | IntExpr::BitXor(left, right) => {
            Bool::and(&[&encode_int_overflow_safety(left), &encode_int_overflow_safety(right)])
        }
        IntExpr::Shl(left, right) | IntExpr::Shr(left, right) => {
            Bool::and(&[&encode_int_overflow_safety(left), &encode_int_overflow_safety(right)])
        }
        IntExpr::BitNot(value) => encode_int_overflow_safety(value),
        IntExpr::Abs(value) => {
            let encoded = encode_int(value);
            Bool::and(&[&encode_int_overflow_safety(value), &encoded.bvneg_no_overflow()])
        }
        IntExpr::StrToInt(value) => encode_str_overflow_safety(value),
        IntExpr::Ord(value) => encode_str_overflow_safety(value),
        IntExpr::Contains(haystack, needle) => {
            Bool::and(&[&encode_str_overflow_safety(haystack), &encode_str_overflow_safety(needle)])
        }
        IntExpr::StartsWith(string, prefix) => {
            Bool::and(&[&encode_str_overflow_safety(string), &encode_str_overflow_safety(prefix)])
        }
        IntExpr::EndsWith(string, suffix) => {
            Bool::and(&[&encode_str_overflow_safety(string), &encode_str_overflow_safety(suffix)])
        }
        IntExpr::Chomp(value) => encode_str_overflow_safety(value),
        IntExpr::Ite(cond, then_int, else_int) => {
            let cond_encoded = encode_bool(cond);
            let then_safe = Bool::or(&[&cond_encoded.not(), &encode_int_overflow_safety(then_int)]);
            let else_safe = Bool::or(&[&cond_encoded, &encode_int_overflow_safety(else_int)]);
            Bool::and(&[&encode_bool_overflow_safety(cond), &then_safe, &else_safe])
        }
        IntExpr::Length(value) => encode_str_overflow_safety(value),
        IntExpr::Index(haystack, needle, start) => {
            Bool::and(&[
                &encode_str_overflow_safety(haystack),
                &encode_str_overflow_safety(needle),
                &encode_int_overflow_safety(start),
            ])
        }
        IntExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_int_overflow_safety(array),
            &encode_int_overflow_safety(index),
        ]),
        IntExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_int_overflow_safety(hash),
            &encode_str_overflow_safety(key),
        ]),
    }
}

fn encode_str_semantic_safety(expr: &StrExpr) -> Bool {
    match expr {
        StrExpr::Const(_) | StrExpr::Var(_) => Bool::from_bool(true),
        StrExpr::Concat(left, right) => {
            Bool::and(&[&encode_str_semantic_safety(left), &encode_str_semantic_safety(right)])
        }
        StrExpr::Substr(value, start, len) => Bool::and(&[
            &encode_str_semantic_safety(value),
            &encode_int_semantic_safety(start),
            &encode_int_semantic_safety(len),
        ]),
        StrExpr::Chr(value) => encode_int_semantic_safety(value),
        StrExpr::FromInt(value) => encode_int_semantic_safety(value),
        StrExpr::Reverse(value) => encode_str_semantic_safety(value),
        StrExpr::Replace(string, from, to) => {
            let from_encoded = encode_str(from);
            let from_nonempty = from_encoded.length().ge(Int::from_i64(1));
            Bool::and(&[
                &encode_str_semantic_safety(string),
                &encode_str_semantic_safety(from),
                &encode_str_semantic_safety(to),
                &from_nonempty,
            ])
        }
        StrExpr::CharAt(string, index) => Bool::and(&[
            &encode_str_semantic_safety(string),
            &encode_int_semantic_safety(index),
        ]),
        StrExpr::Ite(cond, then_str, else_str) => {
            let cond_encoded = encode_bool(cond);
            let then_safe = Bool::or(&[&cond_encoded.not(), &encode_str_semantic_safety(then_str)]);
            let else_safe = Bool::or(&[&cond_encoded, &encode_str_semantic_safety(else_str)]);
            Bool::and(&[&encode_bool_semantic_safety(cond), &then_safe, &else_safe])
        }
        StrExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_str_semantic_safety(array),
            &encode_int_semantic_safety(index),
        ]),
        StrExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_str_semantic_safety(hash),
            &encode_str_semantic_safety(key),
        ]),
    }
}

fn encode_str_overflow_safety(expr: &StrExpr) -> Bool {
    match expr {
        StrExpr::Const(_) | StrExpr::Var(_) => Bool::from_bool(true),
        StrExpr::Concat(left, right) => {
            Bool::and(&[&encode_str_overflow_safety(left), &encode_str_overflow_safety(right)])
        }
        StrExpr::Substr(value, start, len) => Bool::and(&[
            &encode_str_overflow_safety(value),
            &encode_int_overflow_safety(start),
            &encode_int_overflow_safety(len),
        ]),
        StrExpr::Chr(value) => encode_int_overflow_safety(value),
        StrExpr::FromInt(value) => encode_int_overflow_safety(value),
        StrExpr::Reverse(value) => encode_str_overflow_safety(value),
        StrExpr::Replace(string, from, to) => Bool::and(&[
            &encode_str_overflow_safety(string),
            &encode_str_overflow_safety(from),
            &encode_str_overflow_safety(to),
        ]),
        StrExpr::CharAt(string, index) => Bool::and(&[
            &encode_str_overflow_safety(string),
            &encode_int_overflow_safety(index),
        ]),
        StrExpr::Ite(cond, then_str, else_str) => {
            let cond_encoded = encode_bool(cond);
            let then_safe = Bool::or(&[&cond_encoded.not(), &encode_str_overflow_safety(then_str)]);
            let else_safe = Bool::or(&[&cond_encoded, &encode_str_overflow_safety(else_str)]);
            Bool::and(&[&encode_bool_overflow_safety(cond), &then_safe, &else_safe])
        }
        StrExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_str_overflow_safety(array),
            &encode_int_overflow_safety(index),
        ]),
        StrExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_str_overflow_safety(hash),
            &encode_str_overflow_safety(key),
        ]),
    }
}

fn encode_array_int_semantic_safety(expr: &ArrayIntExpr) -> Bool {
    match expr {
        ArrayIntExpr::Var(_) => Bool::from_bool(true),
        ArrayIntExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_int_semantic_safety(base),
            &encode_int_semantic_safety(index),
            &encode_int_semantic_safety(value),
        ]),
    }
}

fn encode_array_int_overflow_safety(expr: &ArrayIntExpr) -> Bool {
    match expr {
        ArrayIntExpr::Var(_) => Bool::from_bool(true),
        ArrayIntExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_int_overflow_safety(base),
            &encode_int_overflow_safety(index),
            &encode_int_overflow_safety(value),
        ]),
    }
}

fn encode_array_str_semantic_safety(expr: &ArrayStrExpr) -> Bool {
    match expr {
        ArrayStrExpr::Var(_) => Bool::from_bool(true),
        ArrayStrExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_str_semantic_safety(base),
            &encode_int_semantic_safety(index),
            &encode_str_semantic_safety(value),
        ]),
    }
}

fn encode_array_str_overflow_safety(expr: &ArrayStrExpr) -> Bool {
    match expr {
        ArrayStrExpr::Var(_) => Bool::from_bool(true),
        ArrayStrExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_str_overflow_safety(base),
            &encode_int_overflow_safety(index),
            &encode_str_overflow_safety(value),
        ]),
    }
}

fn encode_hash_int_semantic_safety(expr: &HashIntExpr) -> Bool {
    match expr {
        HashIntExpr::Var(_) => Bool::from_bool(true),
        HashIntExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_int_semantic_safety(base),
            &encode_str_semantic_safety(key),
            &encode_int_semantic_safety(value),
        ]),
    }
}

fn encode_hash_int_overflow_safety(expr: &HashIntExpr) -> Bool {
    match expr {
        HashIntExpr::Var(_) => Bool::from_bool(true),
        HashIntExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_int_overflow_safety(base),
            &encode_str_overflow_safety(key),
            &encode_int_overflow_safety(value),
        ]),
    }
}

fn encode_hash_str_semantic_safety(expr: &HashStrExpr) -> Bool {
    match expr {
        HashStrExpr::Var(_) => Bool::from_bool(true),
        HashStrExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_str_semantic_safety(base),
            &encode_str_semantic_safety(key),
            &encode_str_semantic_safety(value),
        ]),
    }
}

fn encode_hash_str_overflow_safety(expr: &HashStrExpr) -> Bool {
    match expr {
        HashStrExpr::Var(_) => Bool::from_bool(true),
        HashStrExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_str_overflow_safety(base),
            &encode_str_overflow_safety(key),
            &encode_str_overflow_safety(value),
        ]),
    }
}

fn encode_semantic_safety(expr: &BoolExpr) -> Bool {
    encode_bool_semantic_safety(expr)
}

pub fn encode_overflow_safety(expr: &BoolExpr) -> Bool {
    encode_bool_overflow_safety(expr)
}

fn collect_string_vars_from_bool(expr: &BoolExpr) -> Vec<String> {
    let mut vars = Vec::new();
    collect_string_vars_from_bool_inner(expr, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn collect_string_vars_from_bool_inner(expr: &BoolExpr, vars: &mut Vec<String>) {
    match expr {
        BoolExpr::Const(_) => {}
        BoolExpr::Overflow(exprs) => {
            for e in exprs {
                collect_string_vars_from_int(e, vars);
            }
        }
        BoolExpr::Not(expr) => collect_string_vars_from_bool_inner(expr, vars),
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            collect_string_vars_from_bool_inner(left, vars);
            collect_string_vars_from_bool_inner(right, vars);
        }
        BoolExpr::IntCmp(_, left, right) => {
            collect_string_vars_from_int(left, vars);
            collect_string_vars_from_int(right, vars);
        }
        BoolExpr::StrEq(left, right) | BoolExpr::StrCmp(_, left, right) => {
            collect_string_vars_from_str(left, vars);
            collect_string_vars_from_str(right, vars);
        }
    }
}

fn collect_string_vars_from_int(expr: &IntExpr, vars: &mut Vec<String>) {
    match expr {
        IntExpr::Const(_) | IntExpr::Var(_) => {}
        IntExpr::Add(left, right)
        | IntExpr::Sub(left, right)
        | IntExpr::Mul(left, right)
        | IntExpr::Div(left, right)
        | IntExpr::Mod(left, right)
        | IntExpr::Pow(left, right)
        | IntExpr::BitAnd(left, right)
        | IntExpr::BitOr(left, right)
        | IntExpr::BitXor(left, right)
        | IntExpr::Shl(left, right)
        | IntExpr::Shr(left, right) => {
            collect_string_vars_from_int(left, vars);
            collect_string_vars_from_int(right, vars);
        }
        IntExpr::BitNot(value) => collect_string_vars_from_int(value, vars),
        IntExpr::Abs(value) => collect_string_vars_from_int(value, vars),
        IntExpr::StrToInt(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Contains(haystack, needle) => {
            collect_string_vars_from_str(haystack, vars);
            collect_string_vars_from_str(needle, vars);
        }
        IntExpr::StartsWith(string, prefix) => {
            collect_string_vars_from_str(string, vars);
            collect_string_vars_from_str(prefix, vars);
        }
        IntExpr::EndsWith(string, suffix) => {
            collect_string_vars_from_str(string, vars);
            collect_string_vars_from_str(suffix, vars);
        }
        IntExpr::Ord(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Chomp(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Ite(cond, then_int, else_int) => {
            collect_string_vars_from_bool_inner(cond, vars);
            collect_string_vars_from_int(then_int, vars);
            collect_string_vars_from_int(else_int, vars);
        }
        IntExpr::Length(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Index(haystack, needle, start) => {
            collect_string_vars_from_str(haystack, vars);
            collect_string_vars_from_str(needle, vars);
            collect_string_vars_from_int(start, vars);
        }
        IntExpr::ArraySelect(_, index) => {
            collect_string_vars_from_int(index, vars);
        }
        IntExpr::HashSelect(_, key) => {
            collect_string_vars_from_str(key, vars);
        }
    }
}

fn collect_string_vars_from_str(expr: &StrExpr, vars: &mut Vec<String>) {
    match expr {
        StrExpr::Const(_) => {}
        StrExpr::Var(name) => vars.push(name.clone()),
        StrExpr::Concat(left, right) => {
            collect_string_vars_from_str(left, vars);
            collect_string_vars_from_str(right, vars);
        }
        StrExpr::Substr(value, start, len) => {
            collect_string_vars_from_str(value, vars);
            collect_string_vars_from_int(start, vars);
            collect_string_vars_from_int(len, vars);
        }
        StrExpr::Chr(value) => collect_string_vars_from_int(value, vars),
        StrExpr::FromInt(value) => collect_string_vars_from_int(value, vars),
        StrExpr::Reverse(value) => collect_string_vars_from_str(value, vars),
        StrExpr::Replace(string, from, to) => {
            collect_string_vars_from_str(string, vars);
            collect_string_vars_from_str(from, vars);
            collect_string_vars_from_str(to, vars);
        }
        StrExpr::CharAt(string, index) => {
            collect_string_vars_from_str(string, vars);
            collect_string_vars_from_int(index, vars);
        }
        StrExpr::Ite(cond, then_str, else_str) => {
            collect_string_vars_from_bool_inner(cond, vars);
            collect_string_vars_from_str(then_str, vars);
            collect_string_vars_from_str(else_str, vars);
        }
        StrExpr::ArraySelect(_, index) => collect_string_vars_from_int(index, vars),
        StrExpr::HashSelect(_, key) => collect_string_vars_from_str(key, vars),
    }
}

/// Find the maximum length of any string constant appearing in a BoolExpr tree.
/// Used to ensure variable bounds are at least as large as any constant they
/// might need to equal, preventing false "verified" results.
fn max_string_const_len_bool(expr: &BoolExpr) -> i64 {
    match expr {
        BoolExpr::Const(_) | BoolExpr::Overflow(_) => 0,
        BoolExpr::Not(inner) => max_string_const_len_bool(inner),
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            max_string_const_len_bool(left).max(max_string_const_len_bool(right))
        }
        BoolExpr::IntCmp(_, left, right) => {
            max_string_const_len_int(left).max(max_string_const_len_int(right))
        }
        BoolExpr::StrEq(left, right) | BoolExpr::StrCmp(_, left, right) => {
            max_string_const_len_str(left).max(max_string_const_len_str(right))
        }
    }
}

fn max_string_const_len_int(expr: &IntExpr) -> i64 {
    match expr {
        IntExpr::Const(_) | IntExpr::Var(_) => 0,
        IntExpr::Add(l, r)
        | IntExpr::Sub(l, r)
        | IntExpr::Mul(l, r)
        | IntExpr::Div(l, r)
        | IntExpr::Mod(l, r)
        | IntExpr::Pow(l, r)
        | IntExpr::BitAnd(l, r)
        | IntExpr::BitOr(l, r)
        | IntExpr::BitXor(l, r)
        | IntExpr::Shl(l, r)
        | IntExpr::Shr(l, r) => max_string_const_len_int(l).max(max_string_const_len_int(r)),
        IntExpr::BitNot(v) | IntExpr::Abs(v) => max_string_const_len_int(v),
        IntExpr::StrToInt(v) => max_string_const_len_str(v),
        IntExpr::Contains(h, n) => max_string_const_len_str(h).max(max_string_const_len_str(n)),
        IntExpr::StartsWith(s, p) => max_string_const_len_str(s).max(max_string_const_len_str(p)),
        IntExpr::EndsWith(s, sfx) => max_string_const_len_str(s).max(max_string_const_len_str(sfx)),
        IntExpr::Ord(v) | IntExpr::Chomp(v) => max_string_const_len_str(v),
        IntExpr::Ite(c, t, e) => {
            max_string_const_len_bool(c)
                .max(max_string_const_len_int(t))
                .max(max_string_const_len_int(e))
        }
        IntExpr::Length(v) => max_string_const_len_str(v),
        IntExpr::Index(h, n, s) => {
            max_string_const_len_str(h)
                .max(max_string_const_len_str(n))
                .max(max_string_const_len_int(s))
        }
        IntExpr::ArraySelect(_, idx) => max_string_const_len_int(idx),
        IntExpr::HashSelect(_, key) => max_string_const_len_str(key),
    }
}

fn max_string_const_len_str(expr: &StrExpr) -> i64 {
    match expr {
        StrExpr::Const(s) => s.len() as i64,
        StrExpr::Var(_) => 0,
        StrExpr::Concat(l, r) => max_string_const_len_str(l).max(max_string_const_len_str(r)),
        StrExpr::Substr(v, s, len) => {
            max_string_const_len_str(v)
                .max(max_string_const_len_int(s))
                .max(max_string_const_len_int(len))
        }
        StrExpr::Chr(v) | StrExpr::FromInt(v) => max_string_const_len_int(v),
        StrExpr::Reverse(v) => max_string_const_len_str(v),
        StrExpr::Replace(s, from, to) => {
            max_string_const_len_str(s)
                .max(max_string_const_len_str(from))
                .max(max_string_const_len_str(to))
        }
        StrExpr::CharAt(s, idx) => {
            max_string_const_len_str(s).max(max_string_const_len_int(idx))
        }
        StrExpr::Ite(c, t, e) => {
            max_string_const_len_bool(c)
                .max(max_string_const_len_str(t))
                .max(max_string_const_len_str(e))
        }
        StrExpr::ArraySelect(_, idx) => max_string_const_len_int(idx),
        StrExpr::HashSelect(_, key) => max_string_const_len_str(key),
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_STR_LEN, ModelVar, find_model, is_satisfiable};
    use crate::{
        ast::Type,
        symexec::{ArrayIntExpr, BoolExpr, CmpOp, IntExpr, ModelValue, StrExpr},
    };

    #[test]
    fn detects_unsatisfiable_constraint() {
        let condition = BoolExpr::And(
            Box::new(BoolExpr::IntCmp(
                CmpOp::Gt,
                Box::new(IntExpr::Var("x".to_string())),
                Box::new(IntExpr::Const(0)),
            )),
            Box::new(BoolExpr::IntCmp(
                CmpOp::Le,
                Box::new(IntExpr::Var("x".to_string())),
                Box::new(IntExpr::Const(0)),
            )),
        );

        assert!(!is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn finds_string_model_with_bound() {
        let condition = BoolExpr::StrEq(
            Box::new(StrExpr::Var("x".to_string())),
            Box::new(StrExpr::Const("hello".to_string())),
        );

        let model = find_model(
            "foo",
            &condition,
            &[ModelVar {
                name: "x".to_string(),
                ty: Type::Str,
            }],
        )
        .unwrap()
        .unwrap();

        assert_eq!(model["x"], ModelValue::Str("hello".to_string()));
        assert!("hello".len() as i64 <= MAX_STR_LEN);
    }

    #[test]
    fn supports_indexof_semantics() {
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::Index(
                Box::new(StrExpr::Const("hello".to_string())),
                Box::new(StrExpr::Const("ll".to_string())),
                Box::new(IntExpr::Const(0)),
            )),
            Box::new(IntExpr::Const(2)),
        );

        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn array_store_select_identity() {
        // store(arr, i, v)[i] == v should always hold (theory of arrays)
        let stored = ArrayIntExpr::Store(
            Box::new(ArrayIntExpr::Var("arr".to_string())),
            Box::new(IntExpr::Var("i".to_string())),
            Box::new(IntExpr::Var("v".to_string())),
        );
        let selected = IntExpr::ArraySelect(Box::new(stored), Box::new(IntExpr::Var("i".to_string())));
        // NOT(selected == v) should be UNSAT
        let negation = BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(selected),
            Box::new(IntExpr::Var("v".to_string())),
        );
        assert!(!is_satisfiable("foo", &negation).unwrap(), "array store-then-select should be identity");
    }

    #[test]
    fn prunes_division_by_zero_via_safety_constraints() {
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::Div(
                Box::new(IntExpr::Const(4)),
                Box::new(IntExpr::Const(0)),
            )),
            Box::new(IntExpr::Const(0)),
        );

        assert!(!is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn supports_modulo_semantics() {
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::Mod(
                Box::new(IntExpr::Const(7)),
                Box::new(IntExpr::Const(3)),
            )),
            Box::new(IntExpr::Const(1)),
        );

        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn perl_modulo_matches_all_sign_combinations() {
        // Verify bvsmod matches Perl's % for all sign combinations
        let cases: &[(i64, i64, i64)] = &[
            // (a, b, expected_perl_mod)
            (7, 3, 1),
            (-7, 3, 2),
            (7, -3, -2),
            (-7, -3, -1),
            (10, 3, 1),
            (-10, 3, 2),
            (10, -3, -2),
            (-10, -3, -1),
            (1, 5, 1),
            (-1, 5, 4),
            (1, -5, -4),
            (-1, -5, -1),
        ];

        for &(a, b, perl_expected) in cases {
            let condition = BoolExpr::IntCmp(
                CmpOp::Eq,
                Box::new(IntExpr::Mod(
                    Box::new(IntExpr::Const(a)),
                    Box::new(IntExpr::Const(b)),
                )),
                Box::new(IntExpr::Const(perl_expected)),
            );
            assert!(
                is_satisfiable("foo", &condition).unwrap(),
                "bvsmod({}, {}) should equal {} (Perl's %)",
                a,
                b,
                perl_expected
            );
        }
    }

    #[test]
    fn strtoint_pure_digits_correct() {
        // int("42") == 42 should be satisfiable
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("42".to_string())))),
            Box::new(IntExpr::Const(42)),
        );
        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn strtoint_negative_correct() {
        // int("-7") == -7 should be satisfiable
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("-7".to_string())))),
            Box::new(IntExpr::Const(-7)),
        );
        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn strtoint_empty_returns_zero() {
        // int("") == 0 should be satisfiable
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn strtoint_bare_minus_returns_zero() {
        // int("-") == 0 should be satisfiable
        let condition = BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("-".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &condition).unwrap());
    }

    #[test]
    fn strtoint_whitespace_not_falsely_zero() {
        // int("  42") == 0 should NOT be provably true.
        // The encoding uses an unconstrained fallback for non-pure-digit strings,
        // so int("  42") != 0 is satisfiable (the fallback could be non-zero).
        let not_zero = BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("  42".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &not_zero).unwrap(),
            "int(\"  42\") != 0 should be satisfiable (old bug: encoding falsely forced 0)");
    }

    #[test]
    fn strtoint_trailing_chars_not_falsely_zero() {
        // int("42abc") != 0 should be satisfiable
        let not_zero = BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("42abc".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &not_zero).unwrap(),
            "int(\"42abc\") != 0 should be satisfiable (old bug: encoding falsely forced 0)");
    }

    #[test]
    fn strtoint_decimal_not_falsely_zero() {
        // int("3.14") != 0 should be satisfiable
        let not_zero = BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const("3.14".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &not_zero).unwrap(),
            "int(\"3.14\") != 0 should be satisfiable (old bug: encoding falsely forced 0)");
    }

    #[test]
    fn strtoint_neg_whitespace_not_falsely_zero() {
        // int(" -42") != 0 should be satisfiable
        let not_zero = BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(IntExpr::StrToInt(Box::new(StrExpr::Const(" -42".to_string())))),
            Box::new(IntExpr::Const(0)),
        );
        assert!(is_satisfiable("foo", &not_zero).unwrap(),
            "int(\" -42\") != 0 should be satisfiable (old bug: encoding falsely forced 0)");
    }

    #[test]
    fn replace_empty_pattern_pruned() {
        // replace("abc", "", "x") should be pruned by safety constraints
        // because the pattern is empty.
        let condition = BoolExpr::StrEq(
            Box::new(StrExpr::Replace(
                Box::new(StrExpr::Const("abc".to_string())),
                Box::new(StrExpr::Const("".to_string())),
                Box::new(StrExpr::Const("x".to_string())),
            )),
            Box::new(StrExpr::Const("abc".to_string())),
        );
        assert!(!is_satisfiable("foo", &condition).unwrap(),
            "replace with empty pattern should be pruned by safety constraints");
    }

    #[test]
    fn replace_nonempty_pattern_works() {
        // replace("abc", "b", "x") == "axc" should be satisfiable
        let condition = BoolExpr::StrEq(
            Box::new(StrExpr::Replace(
                Box::new(StrExpr::Const("abc".to_string())),
                Box::new(StrExpr::Const("b".to_string())),
                Box::new(StrExpr::Const("x".to_string())),
            )),
            Box::new(StrExpr::Const("axc".to_string())),
        );
        assert!(is_satisfiable("foo", &condition).unwrap());
    }
}
