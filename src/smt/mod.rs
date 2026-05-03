use std::{collections::BTreeMap, str::FromStr};

use thiserror::Error;
use tracing::debug;
use z3::{
    Context,
    Params,
    Sort,
    Solver,
    ast::{Ast as _, Array, Bool, Int, String as Z3String},
};

use crate::{
    ast::Type,
    limits::DEFAULT_SOLVER_TIMEOUT_MS,
    symexec::{
        ArrayIntExpr, ArrayStrExpr, BoolExpr, HashIntExpr, HashStrExpr, IntExpr, ModelValue,
        StrExpr,
    },
};

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
    solver.assert(&encode_safety_constraints(condition));
    solver.assert(&encode_bool(condition));
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
    solver.assert(&encode_safety_constraints(condition));
    solver.assert(&encode_bool(condition));
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
                    Type::Int => {
                        let symbol = Int::new_const(variable.name.clone());
                        let value = model
                            .eval(&symbol, true)
                            .and_then(|value| value.as_i64())
                            .unwrap_or(0);
                        ModelValue::Int(value)
                    }
                    Type::Str => {
                        let symbol = Z3String::new_const(variable.name.clone());
                        let value = model
                            .eval(&symbol, true)
                            .and_then(|value| value.as_string())
                            .unwrap_or_default();
                        ModelValue::Str(value)
                    }
                    Type::ArrayInt => {
                        let symbol = Array::new_const(variable.name.clone(), &Sort::int(), &Sort::int());
                        let value = model
                            .eval(&symbol, true)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "<unknown>".to_string());
                        ModelValue::Collection(value)
                    }
                    Type::ArrayStr => {
                        let symbol =
                            Array::new_const(variable.name.clone(), &Sort::int(), &Sort::string());
                        let value = model
                            .eval(&symbol, true)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "<unknown>".to_string());
                        ModelValue::Collection(value)
                    }
                    Type::HashInt => {
                        let symbol =
                            Array::new_const(variable.name.clone(), &Sort::string(), &Sort::int());
                        let value = model
                            .eval(&symbol, true)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "<unknown>".to_string());
                        ModelValue::Collection(value)
                    }
                    Type::HashStr => {
                        let symbol = Array::new_const(
                            variable.name.clone(),
                            &Sort::string(),
                            &Sort::string(),
                        );
                        let value = model
                            .eval(&symbol, true)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "<unknown>".to_string());
                        ModelValue::Collection(value)
                    }
                };
                assignments.insert(variable.name.clone(), value);
            }
            Ok(Some(assignments))
        }
    }
}

fn assert_string_bounds(solver: &Solver, condition: &BoolExpr) {
    for variable in collect_string_vars_from_bool(condition) {
        let symbol = Z3String::new_const(variable);
        solver.assert(&symbol.length().le(MAX_STR_LEN));
    }
}

fn apply_solver_timeout(solver: &Solver, timeout_ms: u32) {
    let mut params = Params::new();
    params.set_u32("timeout", timeout_ms);
    solver.set_params(&params);
}

fn encode_int(expr: &IntExpr) -> Int {
    match expr {
        IntExpr::Const(value) => Int::from_i64(*value),
        IntExpr::Var(name) => Int::new_const(name.clone()),
        IntExpr::Add(left, right) => Int::add(&[&encode_int(left), &encode_int(right)]),
        IntExpr::Sub(left, right) => Int::sub(&[&encode_int(left), &encode_int(right)]),
        IntExpr::Mul(left, right) => Int::mul(&[&encode_int(left), &encode_int(right)]),
        IntExpr::Div(left, right) => {
            encode_truncating_division(&encode_int(left), &encode_int(right))
        }
        IntExpr::Mod(left, right) => encode_int(left).rem(&encode_int(right)),
        IntExpr::Abs(value) => {
            let encoded = encode_int(value);
            let is_nonnegative = encoded.ge(&Int::from_i64(0));
            is_nonnegative.ite(&encoded, &encoded.unary_minus())
        }
        IntExpr::Ord(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_str(value);
            unsafe {
                Int::wrap(ctx, z3_sys::Z3_mk_string_to_code(ctx.get_z3_context(), encoded.get_z3_ast()).unwrap())
            }
        }
        IntExpr::Ite(cond, then_int, else_int) => {
            let cond_bool = encode_bool(cond);
            let then_encoded = encode_int(then_int);
            let else_encoded = encode_int(else_int);
            cond_bool.ite(&then_encoded, &else_encoded)
        }
        IntExpr::Length(value) => encode_str(value).length(),
        IntExpr::Index(haystack, needle) => {
            encode_index_of(&encode_str(haystack), &encode_str(needle))
        }
        IntExpr::ArraySelect(array, index) => encode_array_int(array)
            .select(&encode_int(index))
            .as_int()
            .expect("array select should produce Int"),
        IntExpr::HashSelect(hash, key) => encode_hash_int(hash)
            .select(&encode_str(key))
            .as_int()
            .expect("hash select should produce Int"),
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
            encode_str(value).substr(encode_int(start), encode_int(len))
        }
        StrExpr::Chr(value) => {
            let ctx = &Context::thread_local();
            let encoded = encode_int(value);
            unsafe {
                Z3String::wrap(ctx, z3_sys::Z3_mk_string_from_code(ctx.get_z3_context(), encoded.get_z3_ast()).unwrap())
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
        ArrayIntExpr::Var(name) => Array::new_const(name.clone(), &Sort::int(), &Sort::int()),
        ArrayIntExpr::Store(base, index, value) => {
            encode_array_int(base).store(&encode_int(index), &encode_int(value))
        }
    }
}

fn encode_array_str(expr: &ArrayStrExpr) -> Array {
    match expr {
        ArrayStrExpr::Var(name) => Array::new_const(name.clone(), &Sort::int(), &Sort::string()),
        ArrayStrExpr::Store(base, index, value) => {
            encode_array_str(base).store(&encode_int(index), &encode_str(value))
        }
    }
}

fn encode_hash_int(expr: &HashIntExpr) -> Array {
    match expr {
        HashIntExpr::Var(name) => Array::new_const(name.clone(), &Sort::string(), &Sort::int()),
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

fn encode_index_of(haystack: &Z3String, needle: &Z3String) -> Int {
    let needle_len = needle.length();
    let haystack_len = haystack.length();
    let mut result = Int::from_i64(-1);
    for index in (0..=MAX_STR_LEN).rev() {
        let offset = Int::from_i64(index);
        let matches = Bool::and(&[
            &haystack_len.ge(index),
            &haystack
                .substr(offset.clone(), needle_len.clone())
                .eq(needle),
        ]);
        result = matches.ite(&offset, &result);
    }
    result
}

fn encode_truncating_division(left: &Int, right: &Int) -> Int {
    let left_nonnegative = left.ge(0);
    let right_nonnegative = right.ge(0);
    let left_abs = left_nonnegative.ite(left, &left.unary_minus());
    let right_abs = right_nonnegative.ite(right, &right.unary_minus());
    let magnitude = left_abs.div(&right_abs);
    left_nonnegative
        .iff(&right_nonnegative)
        .ite(&magnitude, &magnitude.unary_minus())
}

fn encode_bool(expr: &BoolExpr) -> Bool {
    match expr {
        BoolExpr::Const(value) => Bool::from_bool(*value),
        BoolExpr::Not(expr) => encode_bool(expr).not(),
        BoolExpr::And(left, right) => Bool::and(&[&encode_bool(left), &encode_bool(right)]),
        BoolExpr::Or(left, right) => Bool::or(&[&encode_bool(left), &encode_bool(right)]),
        BoolExpr::IntCmp(op, left, right) => {
            let left = encode_int(left);
            let right = encode_int(right);
            match op {
                crate::symexec::CmpOp::Lt => left.lt(&right),
                crate::symexec::CmpOp::Le => left.le(&right),
                crate::symexec::CmpOp::Gt => left.gt(&right),
                crate::symexec::CmpOp::Ge => left.ge(&right),
                crate::symexec::CmpOp::Eq => left.eq(&right),
                crate::symexec::CmpOp::Ne => left.eq(&right).not(),
            }
        }
        BoolExpr::StrEq(left, right) => encode_str(left).eq(encode_str(right)),
        BoolExpr::StrCmp(op, left, right) => {
            let left_encoded = encode_str(left);
            let right_encoded = encode_str(right);
            match op {
                crate::symexec::CmpOp::Lt | crate::symexec::CmpOp::Le
                | crate::symexec::CmpOp::Gt | crate::symexec::CmpOp::Ge => {
                    Bool::from_bool(true)
                }
                crate::symexec::CmpOp::Eq => left_encoded.eq(&right_encoded),
                crate::symexec::CmpOp::Ne => left_encoded.eq(&right_encoded).not(),
            }
        }
    }
}

fn encode_safety_constraints(expr: &BoolExpr) -> Bool {
    encode_bool_safety(expr)
}

fn encode_bool_safety(expr: &BoolExpr) -> Bool {
    match expr {
        BoolExpr::Const(_) => Bool::from_bool(true),
        BoolExpr::Not(expr) => encode_bool_safety(expr),
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            Bool::and(&[&encode_bool_safety(left), &encode_bool_safety(right)])
        }
        BoolExpr::IntCmp(_, left, right) => {
            Bool::and(&[&encode_int_safety(left), &encode_int_safety(right)])
        }
        BoolExpr::StrEq(left, right) | BoolExpr::StrCmp(_, left, right) => {
            Bool::and(&[&encode_str_safety(left), &encode_str_safety(right)])
        }
    }
}

fn encode_int_safety(expr: &IntExpr) -> Bool {
    match expr {
        IntExpr::Const(_) | IntExpr::Var(_) => Bool::from_bool(true),
        IntExpr::Add(left, right)
        | IntExpr::Sub(left, right)
        | IntExpr::Mul(left, right) => {
            Bool::and(&[&encode_int_safety(left), &encode_int_safety(right)])
        }
        IntExpr::Div(left, right) | IntExpr::Mod(left, right) => Bool::and(&[
            &encode_int_safety(left),
            &encode_int_safety(right),
            &encode_int(right).eq(Int::from_i64(0)).not(),
        ]),
        IntExpr::Abs(value) => encode_int_safety(value),
        IntExpr::Ord(value) => encode_str_safety(value),
        IntExpr::Ite(cond, then_int, else_int) => Bool::and(&[
            &encode_bool_safety(cond),
            &encode_int_safety(then_int),
            &encode_int_safety(else_int),
        ]),
        IntExpr::Length(value) => encode_str_safety(value),
        IntExpr::Index(haystack, needle) => {
            Bool::and(&[&encode_str_safety(haystack), &encode_str_safety(needle)])
        }
        IntExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_int_safety(array),
            &encode_int_safety(index),
        ]),
        IntExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_int_safety(hash),
            &encode_str_safety(key),
        ]),
    }
}

fn encode_str_safety(expr: &StrExpr) -> Bool {
    match expr {
        StrExpr::Const(_) | StrExpr::Var(_) => Bool::from_bool(true),
        StrExpr::Concat(left, right) => {
            Bool::and(&[&encode_str_safety(left), &encode_str_safety(right)])
        }
        StrExpr::Substr(value, start, len) => Bool::and(&[
            &encode_str_safety(value),
            &encode_int_safety(start),
            &encode_int_safety(len),
        ]),
        StrExpr::Chr(value) => encode_int_safety(value),
        StrExpr::Ite(cond, then_str, else_str) => Bool::and(&[
            &encode_bool_safety(cond),
            &encode_str_safety(then_str),
            &encode_str_safety(else_str),
        ]),
        StrExpr::ArraySelect(array, index) => Bool::and(&[
            &encode_array_str_safety(array),
            &encode_int_safety(index),
        ]),
        StrExpr::HashSelect(hash, key) => Bool::and(&[
            &encode_hash_str_safety(hash),
            &encode_str_safety(key),
        ]),
    }
}

fn encode_array_int_safety(expr: &ArrayIntExpr) -> Bool {
    match expr {
        ArrayIntExpr::Var(_) => Bool::from_bool(true),
        ArrayIntExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_int_safety(base),
            &encode_int_safety(index),
            &encode_int_safety(value),
        ]),
    }
}

fn encode_array_str_safety(expr: &ArrayStrExpr) -> Bool {
    match expr {
        ArrayStrExpr::Var(_) => Bool::from_bool(true),
        ArrayStrExpr::Store(base, index, value) => Bool::and(&[
            &encode_array_str_safety(base),
            &encode_int_safety(index),
            &encode_str_safety(value),
        ]),
    }
}

fn encode_hash_int_safety(expr: &HashIntExpr) -> Bool {
    match expr {
        HashIntExpr::Var(_) => Bool::from_bool(true),
        HashIntExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_int_safety(base),
            &encode_str_safety(key),
            &encode_int_safety(value),
        ]),
    }
}

fn encode_hash_str_safety(expr: &HashStrExpr) -> Bool {
    match expr {
        HashStrExpr::Var(_) => Bool::from_bool(true),
        HashStrExpr::Store(base, key, value) => Bool::and(&[
            &encode_hash_str_safety(base),
            &encode_str_safety(key),
            &encode_str_safety(value),
        ]),
    }
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
        | IntExpr::Mod(left, right) => {
            collect_string_vars_from_int(left, vars);
            collect_string_vars_from_int(right, vars);
        }
        IntExpr::Abs(value) => collect_string_vars_from_int(value, vars),
        IntExpr::Ord(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Ite(cond, then_int, else_int) => {
            collect_string_vars_from_bool_inner(cond, vars);
            collect_string_vars_from_int(then_int, vars);
            collect_string_vars_from_int(else_int, vars);
        }
        IntExpr::Length(value) => collect_string_vars_from_str(value, vars),
        IntExpr::Index(haystack, needle) => {
            collect_string_vars_from_str(haystack, vars);
            collect_string_vars_from_str(needle, vars);
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
        StrExpr::Ite(cond, then_str, else_str) => {
            collect_string_vars_from_bool_inner(cond, vars);
            collect_string_vars_from_str(then_str, vars);
            collect_string_vars_from_str(else_str, vars);
        }
        StrExpr::ArraySelect(_, index) => collect_string_vars_from_int(index, vars),
        StrExpr::HashSelect(_, key) => collect_string_vars_from_str(key, vars),
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_STR_LEN, ModelVar, find_model, is_satisfiable};
    use crate::{
        ast::Type,
        symexec::{BoolExpr, CmpOp, IntExpr, ModelValue, StrExpr},
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
            )),
            Box::new(IntExpr::Const(2)),
        );

        assert!(is_satisfiable("foo", &condition).unwrap());
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
}
