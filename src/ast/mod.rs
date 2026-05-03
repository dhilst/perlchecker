//! Core AST and type definitions shared across later phases.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use tracing::debug;

use crate::annotations::FunctionSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Int,
    Str,
    ArrayInt,
    ArrayStr,
    HashInt,
    HashStr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExprType {
    Int,
    Bool,
    Str,
    ArrayInt,
    ArrayStr,
    HashInt,
    HashStr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Builtin {
    Length,
    Substr,
    Index,
    Scalar,
    Abs,
    Min,
    Max,
    Ord,
    Chr,
    Chomp,
    Reverse,
    Int,
    Contains,
    StartsWith,
    EndsWith,
    Replace,
    CharAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessKind {
    Array,
    Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    String(String),
    Variable(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Access {
        kind: AccessKind,
        collection: String,
        index: Box<Expr>,
    },
    Call {
        function: String,
        args: Vec<Expr>,
    },
    Builtin {
        function: Builtin,
        args: Vec<Expr>,
    },
    Pop {
        array: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    StrEq,
    StrNe,
    StrLt,
    StrGt,
    StrLe,
    StrGe,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Spaceship,
    Cmp,
    Repeat,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionAst {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stmt {
    Declare {
        name: String,
    },
    Assign {
        name: String,
        expr: Expr,
        declaration: bool,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    Return(Expr),
    ArrayAssign {
        name: String,
        index: Expr,
        expr: Expr,
    },
    HashAssign {
        name: String,
        key: Expr,
        expr: Expr,
    },
    LoopBoundExceeded,
    Last,
    Next,
    Die(Expr),
    Push {
        array: String,
        value: Expr,
    },
    ArrayInit {
        name: String,
        elements: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VariableState {
    ty: Option<Type>,
    initialized: bool,
}

type TypeEnv = BTreeMap<String, VariableState>;
type Assumptions = BTreeSet<Expr>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error(
        "function `{function}` declares {expected} arguments in `# sig:` but parses {actual} parameters"
    )]
    ParameterCountMismatch {
        function: String,
        expected: usize,
        actual: usize,
    },
    #[error("function `{function}` references undeclared variable `${variable}`")]
    UndeclaredVariable { function: String, variable: String },
    #[error("function `{function}` reads uninitialized variable `${variable}`")]
    UninitializedVariable { function: String, variable: String },
    #[error("function `{function}` uses a negative substring {argument}")]
    NegativeSubstringArgument {
        function: String,
        argument: &'static str,
    },
    #[error("function `{function}` uses substr without proving the start offset is in bounds")]
    UnsafeSubstringStart { function: String },
    #[error(
        "function `{function}` uses substr without proving the requested length is nonnegative"
    )]
    UnsafeSubstringLength { function: String },
    #[error("function `{function}` expected `{context}` to be {expected} but found {found}")]
    TypeMismatch {
        function: String,
        context: &'static str,
        expected: &'static str,
        found: &'static str,
    },
}

pub fn type_check_function(
    spec: &FunctionSpec,
    function: &FunctionAst,
) -> std::result::Result<(), TypeCheckError> {
    type_check_function_with_signatures(spec, function, &BTreeMap::new())
}

pub fn type_check_function_with_signatures(
    spec: &FunctionSpec,
    function: &FunctionAst,
    signatures: &BTreeMap<String, (Vec<Type>, Type)>,
) -> std::result::Result<(), TypeCheckError> {
    if spec.arg_types.len() != function.params.len() {
        return Err(TypeCheckError::ParameterCountMismatch {
            function: function.name.clone(),
            expected: spec.arg_types.len(),
            actual: function.params.len(),
        });
    }

    let mut env = BTreeMap::new();
    for (param, ty) in function.params.iter().zip(spec.arg_types.iter()) {
        env.insert(
            param.clone(),
            VariableState {
                ty: Some(*ty),
                initialized: true,
            },
        );
    }

    let initial_assumptions = collect_true_assumptions(&spec.pre);
    expect_expr_type(
        &function.name,
        "precondition",
        &spec.pre,
        &env,
        &initial_assumptions,
        ExprType::Bool,
        signatures,
    )?;
    let (env, assumptions) = type_check_stmts(
        &function.name,
        &function.body,
        &env,
        &initial_assumptions,
        spec.ret_type,
        signatures,
    )?;

    let mut post_env = env;
    post_env.insert(
        "result".to_string(),
        VariableState {
            ty: Some(spec.ret_type),
            initialized: true,
        },
    );
    expect_expr_type(
        &function.name,
        "postcondition",
        &spec.post,
        &post_env,
        &assumptions,
        ExprType::Bool,
        signatures,
    )?;

    debug!(function = function.name, "type checking completed");
    Ok(())
}

fn type_check_stmts(
    function: &str,
    stmts: &[Stmt],
    env: &TypeEnv,
    assumptions: &Assumptions,
    return_type: Type,
    signatures: &BTreeMap<String, (Vec<Type>, Type)>,
) -> std::result::Result<(TypeEnv, Assumptions), TypeCheckError> {
    let mut env = env.clone();
    let mut assumptions = assumptions.clone();

    for stmt in stmts {
        match stmt {
            Stmt::Declare { name } => {
                env.insert(
                    name.clone(),
                    VariableState {
                        ty: None,
                        initialized: false,
                    },
                );
                assumptions = remove_variable_assumptions(&assumptions, name);
            }
            Stmt::Assign {
                name,
                expr,
                declaration,
            } => {
                let expr_type = infer_expr_type(function, expr, &env, &assumptions, signatures)?;
                let assign_type = expect_assignable_type(function, "assignment", expr_type)?;
                if *declaration {
                    env.insert(
                        name.clone(),
                        VariableState {
                            ty: Some(assign_type),
                            initialized: true,
                        },
                    );
                } else {
                    let prior = env.get(name).copied().ok_or_else(|| {
                        TypeCheckError::UndeclaredVariable {
                            function: function.to_string(),
                            variable: name.clone(),
                        }
                    })?;
                    if let Some(prior_type) = prior.ty
                        && prior_type != assign_type
                    {
                        return Err(TypeCheckError::TypeMismatch {
                            function: function.to_string(),
                            context: "assignment",
                            expected: render_expr_type(expr_type_from_type(prior_type)),
                            found: render_expr_type(expr_type),
                        });
                    }
                    env.insert(
                        name.clone(),
                        VariableState {
                            ty: Some(assign_type),
                            initialized: true,
                        },
                    );
                }
                assumptions = remove_variable_assumptions(&assumptions, name);
            }
            Stmt::ArrayAssign { name, index, expr } => {
                expect_expr_type(
                    function,
                    "array index",
                    index,
                    &env,
                    &assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                let expr_type = infer_expr_type(function, expr, &env, &assumptions, signatures)?;
                let element_type = collection_element_type(function, &env, name, AccessKind::Array)?;
                if expr_type != element_type {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "array assignment",
                        expected: render_expr_type(element_type),
                        found: render_expr_type(expr_type),
                    });
                }
                assumptions = remove_variable_assumptions(&assumptions, name);
            }
            Stmt::HashAssign { name, key, expr } => {
                expect_expr_type(
                    function,
                    "hash key",
                    key,
                    &env,
                    &assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                let expr_type = infer_expr_type(function, expr, &env, &assumptions, signatures)?;
                let element_type = collection_element_type(function, &env, name, AccessKind::Hash)?;
                if expr_type != element_type {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "hash assignment",
                        expected: render_expr_type(element_type),
                        found: render_expr_type(expr_type),
                    });
                }
                assumptions = remove_variable_assumptions(&assumptions, name);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                expect_expr_type(
                    function,
                    "if condition",
                    condition,
                    &env,
                    &assumptions,
                    ExprType::Bool,
                    signatures,
                )?;

                let mut then_assumptions = assumptions.clone();
                then_assumptions.extend(collect_true_assumptions(condition));
                let mut else_assumptions = assumptions.clone();
                else_assumptions.extend(collect_false_assumptions(condition));

                let (then_env, then_assumptions) =
                    type_check_stmts(
                        function,
                        then_branch,
                        &env,
                        &then_assumptions,
                        return_type,
                        signatures,
                    )?;
                let (else_env, else_assumptions) = if else_branch.is_empty() {
                    (env.clone(), else_assumptions)
                } else {
                    type_check_stmts(
                        function,
                        else_branch,
                        &env,
                        &else_assumptions,
                        return_type,
                        signatures,
                    )?
                };
                env = merge_branch_envs(function, &env, &then_env, &else_env)?;
                assumptions = intersect_assumptions(&then_assumptions, &else_assumptions);
            }
            Stmt::Return(expr) => {
                expect_expr_type(
                    function,
                    "return expression",
                    expr,
                    &env,
                    &assumptions,
                    expr_type_from_type(return_type),
                    signatures,
                )?;
            }
            Stmt::Push { array, value } => {
                let expr_type = infer_expr_type(function, value, &env, &assumptions, signatures)?;
                let element_type = collection_element_type(function, &env, array, AccessKind::Array)?;
                if expr_type != element_type {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "push value",
                        expected: render_expr_type(element_type),
                        found: render_expr_type(expr_type),
                    });
                }
            }
            Stmt::ArrayInit { name, elements } => {
                // Infer type from first element, check all elements match
                let first_type = infer_expr_type(function, &elements[0], &env, &assumptions, signatures)?;
                let array_type = match first_type {
                    ExprType::Int => Type::ArrayInt,
                    ExprType::Str => Type::ArrayStr,
                    _ => {
                        return Err(TypeCheckError::TypeMismatch {
                            function: function.to_string(),
                            context: "array init element",
                            expected: "Int or Str",
                            found: render_expr_type(first_type),
                        });
                    }
                };
                for elem in &elements[1..] {
                    let elem_type = infer_expr_type(function, elem, &env, &assumptions, signatures)?;
                    if elem_type != first_type {
                        return Err(TypeCheckError::TypeMismatch {
                            function: function.to_string(),
                            context: "array init element",
                            expected: render_expr_type(first_type),
                            found: render_expr_type(elem_type),
                        });
                    }
                }
                env.insert(
                    name.clone(),
                    VariableState {
                        ty: Some(array_type),
                        initialized: true,
                    },
                );
            }
            Stmt::LoopBoundExceeded => {}
            Stmt::Last => {}
            Stmt::Next => {}
            Stmt::Die(_) => {}
        }
    }

    Ok((env, assumptions))
}

fn merge_branch_envs(
    function: &str,
    base: &TypeEnv,
    then_env: &TypeEnv,
    else_env: &TypeEnv,
) -> std::result::Result<TypeEnv, TypeCheckError> {
    let mut merged = BTreeMap::new();
    let keys = base
        .keys()
        .chain(then_env.keys())
        .chain(else_env.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for key in keys {
        let base_state = base.get(&key).copied();
        let then_state = then_env.get(&key).copied().or(base_state);
        let else_state = else_env.get(&key).copied().or(base_state);
        if let (Some(then_state), Some(else_state)) = (then_state, else_state) {
            let unified_type =
                unify_optional_type(function, "branch assignment", then_state.ty, else_state.ty)?;
            let initialized = then_state.initialized && else_state.initialized;
            let ty = if initialized {
                unified_type.or(base_state.and_then(|state| state.ty))
            } else {
                base_state.and_then(|state| state.ty)
            };
            merged.insert(key, VariableState { ty, initialized });
        }
    }

    Ok(merged)
}

fn unify_optional_type(
    function: &str,
    context: &'static str,
    left: Option<Type>,
    right: Option<Type>,
) -> std::result::Result<Option<Type>, TypeCheckError> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(TypeCheckError::TypeMismatch {
            function: function.to_string(),
            context,
            expected: render_expr_type(expr_type_from_type(left)),
            found: render_expr_type(expr_type_from_type(right)),
        }),
        (Some(left), Some(_)) => Ok(Some(left)),
        (Some(left), None) | (None, Some(left)) => Ok(Some(left)),
        (None, None) => Ok(None),
    }
}

fn intersect_assumptions(left: &Assumptions, right: &Assumptions) -> Assumptions {
    left.intersection(right).cloned().collect()
}

fn remove_variable_assumptions(assumptions: &Assumptions, name: &str) -> Assumptions {
    assumptions
        .iter()
        .filter(|expr| !references_variable(expr, name))
        .cloned()
        .collect()
}

fn references_variable(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Variable(variable) => variable == name,
        Expr::Unary { expr, .. } => references_variable(expr, name),
        Expr::Binary { left, right, .. } => {
            references_variable(left, name) || references_variable(right, name)
        }
        Expr::Ternary { condition, then_expr, else_expr } => {
            references_variable(condition, name) || references_variable(then_expr, name) || references_variable(else_expr, name)
        }
        Expr::Access {
            collection, index, ..
        } => collection == name || references_variable(index, name),
        Expr::Call { args, .. } => args.iter().any(|arg| references_variable(arg, name)),
        Expr::Builtin { args, .. } => args.iter().any(|arg| references_variable(arg, name)),
        Expr::Pop { array } => array == name,
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) => false,
    }
}

fn expect_expr_type(
    function: &str,
    context: &'static str,
    expr: &Expr,
    env: &TypeEnv,
    assumptions: &Assumptions,
    expected: ExprType,
    signatures: &BTreeMap<String, (Vec<Type>, Type)>,
) -> std::result::Result<(), TypeCheckError> {
    let found = infer_expr_type(function, expr, env, assumptions, signatures)?;
    if found == expected {
        Ok(())
    } else {
        Err(TypeCheckError::TypeMismatch {
            function: function.to_string(),
            context,
            expected: render_expr_type(expected),
            found: render_expr_type(found),
        })
    }
}

fn infer_expr_type(
    function: &str,
    expr: &Expr,
    env: &TypeEnv,
    assumptions: &Assumptions,
    signatures: &BTreeMap<String, (Vec<Type>, Type)>,
) -> std::result::Result<ExprType, TypeCheckError> {
    match expr {
        Expr::Int(_) => Ok(ExprType::Int),
        Expr::Bool(_) => Ok(ExprType::Bool),
        Expr::String(_) => Ok(ExprType::Str),
        Expr::Variable(name) => env
            .get(name)
            .copied()
            .ok_or_else(|| TypeCheckError::UndeclaredVariable {
                function: function.to_string(),
                variable: name.clone(),
            })
            .and_then(|state| {
                if !state.initialized {
                    return Err(TypeCheckError::UninitializedVariable {
                        function: function.to_string(),
                        variable: name.clone(),
                    });
                }
                let ty = state.ty.expect("initialized variables must have a type");
                Ok(expr_type_from_type(ty))
            }),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => {
                expect_expr_type(
                    function,
                    "unary negation",
                    expr,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            UnaryOp::Not => {
                expect_expr_type(
                    function,
                    "logical negation",
                    expr,
                    env,
                    assumptions,
                    ExprType::Bool,
                    signatures,
                )?;
                Ok(ExprType::Bool)
            }
            UnaryOp::BitNot => {
                expect_expr_type(
                    function,
                    "bitwise complement",
                    expr,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
        },
        Expr::Binary { left, op, right } => match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
            | BinaryOp::Pow | BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor
            | BinaryOp::Shl | BinaryOp::Shr | BinaryOp::Spaceship => {
                expect_expr_type(
                    function,
                    "arithmetic operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "arithmetic operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            BinaryOp::Concat => {
                let left_ty = infer_expr_type(function, left, env, assumptions, signatures)?;
                if left_ty != ExprType::Str && left_ty != ExprType::Int {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "concat operand",
                        expected: "Str or Int",
                        found: render_expr_type(left_ty),
                    });
                }
                let right_ty = infer_expr_type(function, right, env, assumptions, signatures)?;
                if right_ty != ExprType::Str && right_ty != ExprType::Int {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "concat operand",
                        expected: "Str or Int",
                        found: render_expr_type(right_ty),
                    });
                }
                Ok(ExprType::Str)
            }
            BinaryOp::Repeat => {
                expect_expr_type(
                    function,
                    "repeat operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "repeat count",
                    right,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                expect_expr_type(
                    function,
                    "comparison operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "comparison operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Bool)
            }
            BinaryOp::Eq | BinaryOp::Ne => {
                expect_expr_type(
                    function,
                    "equality operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "equality operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Bool)
            }
            BinaryOp::StrEq | BinaryOp::StrNe | BinaryOp::StrLt | BinaryOp::StrGt | BinaryOp::StrLe | BinaryOp::StrGe => {
                expect_expr_type(
                    function,
                    "string comparison operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "string comparison operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Bool)
            }
            BinaryOp::Cmp => {
                expect_expr_type(
                    function,
                    "string comparison operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "string comparison operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            BinaryOp::And | BinaryOp::Or => {
                expect_expr_type(
                    function,
                    "boolean operand",
                    left,
                    env,
                    assumptions,
                    ExprType::Bool,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "boolean operand",
                    right,
                    env,
                    assumptions,
                    ExprType::Bool,
                    signatures,
                )?;
                Ok(ExprType::Bool)
            }
        },
        Expr::Access {
            kind,
            collection,
            index,
        } => {
            let collection_type = infer_expr_type(
                function,
                &Expr::Variable(collection.clone()),
                env,
                assumptions,
                signatures,
            )?;
            match kind {
                AccessKind::Array => {
                    expect_expr_type(
                        function,
                        "array index",
                        index,
                        env,
                        assumptions,
                        ExprType::Int,
                        signatures,
                    )?;
                    match collection_type {
                        ExprType::ArrayInt => Ok(ExprType::Int),
                        ExprType::ArrayStr => Ok(ExprType::Str),
                        _ => Err(TypeCheckError::TypeMismatch {
                            function: function.to_string(),
                            context: "array access",
                            expected: "Array<Int> or Array<Str>",
                            found: render_expr_type(collection_type),
                        }),
                    }
                }
                AccessKind::Hash => {
                    expect_expr_type(
                        function,
                        "hash key",
                        index,
                        env,
                        assumptions,
                        ExprType::Str,
                        signatures,
                    )?;
                    match collection_type {
                        ExprType::HashInt => Ok(ExprType::Int),
                        ExprType::HashStr => Ok(ExprType::Str),
                        _ => Err(TypeCheckError::TypeMismatch {
                            function: function.to_string(),
                            context: "hash access",
                            expected: "Hash<Str, Int> or Hash<Str, Str>",
                            found: render_expr_type(collection_type),
                        }),
                    }
                }
            }
        }
        Expr::Call { function: callee, args } => {
            let (arg_types, ret_type) =
                signatures
                    .get(callee)
                    .ok_or_else(|| TypeCheckError::UndeclaredVariable {
                        function: function.to_string(),
                        variable: callee.clone(),
                    })?;
            if arg_types.len() != args.len() {
                return Err(TypeCheckError::TypeMismatch {
                    function: function.to_string(),
                    context: "function call arity",
                    expected: "matching signature arity",
                    found: "different arity",
                });
            }
            for (arg, expected) in args.iter().zip(arg_types.iter()) {
                expect_expr_type(
                    function,
                    "function argument",
                    arg,
                    env,
                    assumptions,
                    expr_type_from_type(*expected),
                    signatures,
                )?;
            }
            Ok(expr_type_from_type(*ret_type))
        }
        Expr::Builtin {
            function: builtin,
            args,
        } => match builtin {
            Builtin::Length => {
                let [value] = args.as_slice() else {
                    unreachable!("length arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "length argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::Substr => {
                let [value, start, len] = args.as_slice() else {
                    unreachable!("substr arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "substr value",
                    value,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "substr start",
                    start,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "substr length",
                    len,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                if is_negative_literal(start) {
                    return Err(TypeCheckError::NegativeSubstringArgument {
                        function: function.to_string(),
                        argument: "start",
                    });
                }
                if is_negative_literal(len) {
                    return Err(TypeCheckError::NegativeSubstringArgument {
                        function: function.to_string(),
                        argument: "length",
                    });
                }
                if !is_proven_nonnegative(start, assumptions)
                    || !is_proven_leq_to_length(start, value, assumptions)
                {
                    return Err(TypeCheckError::UnsafeSubstringStart {
                        function: function.to_string(),
                    });
                }
                // Skip length-nonneg check when len is the desugared form
                // `length(value) - start` (from 2-arg substr), since the start
                // check above already guarantees start <= length(value).
                if !is_desugared_substr_length(len, value, start)
                    && !is_proven_nonnegative(len, assumptions)
                {
                    return Err(TypeCheckError::UnsafeSubstringLength {
                        function: function.to_string(),
                    });
                }
                Ok(ExprType::Str)
            }
            Builtin::Index => {
                let (haystack, needle, start) = match args.as_slice() {
                    [haystack, needle] => (haystack, needle, None),
                    [haystack, needle, start] => (haystack, needle, Some(start)),
                    _ => unreachable!("index arity is enforced by the parser"),
                };
                expect_expr_type(
                    function,
                    "index haystack",
                    haystack,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "index needle",
                    needle,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                if let Some(start) = start {
                    expect_expr_type(
                        function,
                        "index start position",
                        start,
                        env,
                        assumptions,
                        ExprType::Int,
                        signatures,
                    )?;
                }
                Ok(ExprType::Int)
            }
            Builtin::Scalar => {
                let [array] = args.as_slice() else {
                    unreachable!("scalar arity is enforced by the parser");
                };
                let array_type = infer_expr_type(
                    function,
                    array,
                    env,
                    assumptions,
                    signatures,
                )?;
                match array_type {
                    ExprType::ArrayInt | ExprType::ArrayStr => Ok(ExprType::Int),
                    _ => Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "scalar argument",
                        expected: "Array<Int> or Array<Str>",
                        found: render_expr_type(array_type),
                    }),
                }
            }
            Builtin::Abs => {
                let [value] = args.as_slice() else {
                    unreachable!("abs arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "abs argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::Min | Builtin::Max => {
                let [left, right] = args.as_slice() else {
                    unreachable!("min/max arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "min/max argument",
                    left,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "min/max argument",
                    right,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::Ord => {
                let [value] = args.as_slice() else {
                    unreachable!("ord arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "ord argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::Chr => {
                let [value] = args.as_slice() else {
                    unreachable!("chr arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "chr argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
            Builtin::Chomp => {
                let [value] = args.as_slice() else {
                    unreachable!("chomp arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "chomp argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
            Builtin::Reverse => {
                let [value] = args.as_slice() else {
                    unreachable!("reverse arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "reverse argument",
                    value,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
            Builtin::Int => {
                let [value] = args.as_slice() else {
                    unreachable!("int arity is enforced by the parser");
                };
                let arg_type = infer_expr_type(function, value, env, assumptions, signatures)?;
                if arg_type != ExprType::Str && arg_type != ExprType::Int {
                    return Err(TypeCheckError::TypeMismatch {
                        function: function.to_string(),
                        context: "int argument",
                        expected: "Str or Int",
                        found: render_expr_type(arg_type),
                    });
                }
                Ok(ExprType::Int)
            }
            Builtin::Contains => {
                let [haystack, needle] = args.as_slice() else {
                    unreachable!("contains arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "contains haystack",
                    haystack,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "contains needle",
                    needle,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::StartsWith => {
                let [string, prefix] = args.as_slice() else {
                    unreachable!("starts_with arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "starts_with string",
                    string,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "starts_with prefix",
                    prefix,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::EndsWith => {
                let [string, suffix] = args.as_slice() else {
                    unreachable!("ends_with arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "ends_with string",
                    string,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "ends_with suffix",
                    suffix,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Int)
            }
            Builtin::Replace => {
                let [string, from, to] = args.as_slice() else {
                    unreachable!("replace arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "replace string",
                    string,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "replace from",
                    from,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "replace to",
                    to,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
            Builtin::CharAt => {
                let [string, index] = args.as_slice() else {
                    unreachable!("char_at arity is enforced by the parser");
                };
                expect_expr_type(
                    function,
                    "char_at string",
                    string,
                    env,
                    assumptions,
                    ExprType::Str,
                    signatures,
                )?;
                expect_expr_type(
                    function,
                    "char_at index",
                    index,
                    env,
                    assumptions,
                    ExprType::Int,
                    signatures,
                )?;
                Ok(ExprType::Str)
            }
        },
        Expr::Ternary { condition, then_expr, else_expr } => {
            expect_expr_type(
                function,
                "ternary condition",
                condition,
                env,
                assumptions,
                ExprType::Bool,
                signatures,
            )?;
            let then_type = infer_expr_type(function, then_expr, env, assumptions, signatures)?;
            let else_type = infer_expr_type(function, else_expr, env, assumptions, signatures)?;
            if then_type != else_type {
                return Err(TypeCheckError::TypeMismatch {
                    function: function.to_string(),
                    context: "ternary branches",
                    expected: render_expr_type(then_type),
                    found: render_expr_type(else_type),
                });
            }
            Ok(then_type)
        }
        Expr::Pop { array } => {
            collection_element_type(function, env, array, AccessKind::Array)
        }
    }
}

fn expect_assignable_type(
    function: &str,
    context: &'static str,
    found: ExprType,
) -> std::result::Result<Type, TypeCheckError> {
    match found {
        ExprType::Int => Ok(Type::Int),
        ExprType::Str => Ok(Type::Str),
        ExprType::ArrayInt => Ok(Type::ArrayInt),
        ExprType::ArrayStr => Ok(Type::ArrayStr),
        ExprType::HashInt => Ok(Type::HashInt),
        ExprType::HashStr => Ok(Type::HashStr),
        ExprType::Bool => Err(TypeCheckError::TypeMismatch {
            function: function.to_string(),
            context,
            expected: "Int, Str, Array<Int>, Array<Str>, Hash<Str, Int>, or Hash<Str, Str>",
            found: render_expr_type(found),
        }),
    }
}

fn is_negative_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Int(value) if *value < 0)
        || matches!(
            expr,
            Expr::Unary {
                op: UnaryOp::Neg,
                expr
            } if matches!(expr.as_ref(), Expr::Int(value) if *value > 0)
        )
}

fn is_proven_nonnegative(expr: &Expr, assumptions: &Assumptions) -> bool {
    match expr {
        Expr::Int(value) => *value >= 0,
        Expr::Builtin {
            function: Builtin::Length,
            ..
        } => true,
        _ => assumptions
            .iter()
            .any(|assumption| assumption_implies_nonnegative(expr, assumption)),
    }
}

fn is_proven_leq_to_length(expr: &Expr, string_expr: &Expr, assumptions: &Assumptions) -> bool {
    if matches!(expr, Expr::Int(0)) {
        return true;
    }
    if *expr == length_expr(string_expr.clone()) {
        return true;
    }

    let target = length_expr(string_expr.clone());
    assumptions
        .iter()
        .any(|assumption| assumption_implies_leq(expr, &target, assumption))
}

fn collect_true_assumptions(expr: &Expr) -> Assumptions {
    match expr {
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
        } => collect_false_assumptions(expr),
        Expr::Binary {
            left,
            op: BinaryOp::And,
            right,
        } => {
            let mut assumptions = collect_true_assumptions(left);
            assumptions.extend(collect_true_assumptions(right));
            assumptions
        }
        Expr::Bool(true) => BTreeSet::new(),
        _ => [expr.clone()].into_iter().collect(),
    }
}

fn collect_false_assumptions(expr: &Expr) -> Assumptions {
    match expr {
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
        } => collect_true_assumptions(expr),
        Expr::Binary { left, op, right } => {
            if let Some(complement) = complement_of_comparison(left, op, right) {
                [complement].into_iter().collect()
            } else {
                [Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr.clone()),
                }]
                .into_iter()
                .collect()
            }
        }
        _ => [Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(expr.clone()),
        }]
        .into_iter()
        .collect(),
    }
}

fn complement_of_comparison(left: &Expr, op: &BinaryOp, right: &Expr) -> Option<Expr> {
    let inverse = match op {
        BinaryOp::Lt => BinaryOp::Ge,
        BinaryOp::Le => BinaryOp::Gt,
        BinaryOp::Gt => BinaryOp::Le,
        BinaryOp::Ge => BinaryOp::Lt,
        BinaryOp::Eq => BinaryOp::Ne,
        BinaryOp::Ne => BinaryOp::Eq,
        BinaryOp::StrEq => BinaryOp::StrNe,
        BinaryOp::StrNe => BinaryOp::StrEq,
        BinaryOp::StrLt => BinaryOp::StrGe,
        BinaryOp::StrLe => BinaryOp::StrGt,
        BinaryOp::StrGt => BinaryOp::StrLe,
        BinaryOp::StrGe => BinaryOp::StrLt,
        BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Pow
        | BinaryOp::Concat
        | BinaryOp::And
        | BinaryOp::Or
        | BinaryOp::BitAnd
        | BinaryOp::BitOr
        | BinaryOp::BitXor
        | BinaryOp::Shl
        | BinaryOp::Shr
        | BinaryOp::Spaceship
        | BinaryOp::Cmp
        | BinaryOp::Repeat => return None,
    };
    Some(Expr::Binary {
        left: Box::new(left.clone()),
        op: inverse,
        right: Box::new(right.clone()),
    })
}

fn assumption_implies_nonnegative(target: &Expr, assumption: &Expr) -> bool {
    match assumption {
        Expr::Binary { left, op, right } => match op {
            BinaryOp::Ge => {
                (left.as_ref() == target
                    && matches!(right.as_ref(), Expr::Int(value) if *value >= 0))
                    || (right.as_ref() == target
                        && matches!(left.as_ref(), Expr::Int(value) if *value <= 0))
            }
            BinaryOp::Gt => {
                (left.as_ref() == target
                    && matches!(right.as_ref(), Expr::Int(value) if *value >= -1))
                    || (right.as_ref() == target
                        && matches!(left.as_ref(), Expr::Int(value) if *value <= 1))
            }
            BinaryOp::Le => {
                (right.as_ref() == target
                    && matches!(left.as_ref(), Expr::Int(value) if *value >= 0))
                    || (left.as_ref() == target
                        && matches!(right.as_ref(), Expr::Int(value) if *value <= 0))
            }
            BinaryOp::Lt => {
                (right.as_ref() == target
                    && matches!(left.as_ref(), Expr::Int(value) if *value >= -1))
                    || (left.as_ref() == target
                        && matches!(right.as_ref(), Expr::Int(value) if *value <= 1))
            }
            _ => false,
        },
        _ => false,
    }
}

fn assumption_implies_leq(left_target: &Expr, right_target: &Expr, assumption: &Expr) -> bool {
    match assumption {
        Expr::Binary { left, op, right } => match op {
            BinaryOp::Le | BinaryOp::Lt => {
                left.as_ref() == left_target && right.as_ref() == right_target
            }
            BinaryOp::Ge | BinaryOp::Gt => {
                left.as_ref() == right_target && right.as_ref() == left_target
            }
            _ => false,
        },
        _ => false,
    }
}

fn length_expr(expr: Expr) -> Expr {
    Expr::Builtin {
        function: Builtin::Length,
        args: vec![expr],
    }
}

/// Detect the desugared 2-arg substr pattern: `length(value) - start`.
fn is_desugared_substr_length(len: &Expr, value: &Expr, start: &Expr) -> bool {
    matches!(
        len,
        Expr::Binary {
            left,
            op: BinaryOp::Sub,
            right,
        } if **left == length_expr(value.clone()) && **right == *start
    )
}

fn collection_element_type(
    function: &str,
    env: &TypeEnv,
    name: &str,
    kind: AccessKind,
) -> std::result::Result<ExprType, TypeCheckError> {
    let state = env
        .get(name)
        .copied()
        .ok_or_else(|| TypeCheckError::UndeclaredVariable {
            function: function.to_string(),
            variable: name.to_string(),
        })?;
    if !state.initialized {
        return Err(TypeCheckError::UninitializedVariable {
            function: function.to_string(),
            variable: name.to_string(),
        });
    }
    match (kind, state.ty.expect("initialized variables must have a type")) {
        (AccessKind::Array, Type::ArrayInt) => Ok(ExprType::Int),
        (AccessKind::Array, Type::ArrayStr) => Ok(ExprType::Str),
        (AccessKind::Hash, Type::HashInt) => Ok(ExprType::Int),
        (AccessKind::Hash, Type::HashStr) => Ok(ExprType::Str),
        (AccessKind::Array, ty) => Err(TypeCheckError::TypeMismatch {
            function: function.to_string(),
            context: "array assignment",
            expected: "Array<Int> or Array<Str>",
            found: render_expr_type(expr_type_from_type(ty)),
        }),
        (AccessKind::Hash, ty) => Err(TypeCheckError::TypeMismatch {
            function: function.to_string(),
            context: "hash assignment",
            expected: "Hash<Str, Int> or Hash<Str, Str>",
            found: render_expr_type(expr_type_from_type(ty)),
        }),
    }
}

fn expr_type_from_type(ty: Type) -> ExprType {
    match ty {
        Type::Int => ExprType::Int,
        Type::Str => ExprType::Str,
        Type::ArrayInt => ExprType::ArrayInt,
        Type::ArrayStr => ExprType::ArrayStr,
        Type::HashInt => ExprType::HashInt,
        Type::HashStr => ExprType::HashStr,
    }
}

fn render_expr_type(ty: ExprType) -> &'static str {
    match ty {
        ExprType::Int => "Int",
        ExprType::Bool => "Bool",
        ExprType::Str => "Str",
        ExprType::ArrayInt => "Array<Int>",
        ExprType::ArrayStr => "Array<Str>",
        ExprType::HashInt => "Hash<Str, Int>",
        ExprType::HashStr => "Hash<Str, Str>",
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        annotations::parse_function_spec, extractor::ExtractedFunction, parser::parse_function_ast,
    };

    use super::{TypeCheckError, type_check_function};

    #[test]
    fn detects_undeclared_variables() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result > $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $y;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        let error = type_check_function(&spec, &ast).unwrap_err();

        assert_eq!(
            error,
            TypeCheckError::UndeclaredVariable {
                function: "foo".to_string(),
                variable: "y".to_string(),
            }
        );
    }

    #[test]
    fn detects_uninitialized_variables() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result >= $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    my $y;\n    return $y;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        let error = type_check_function(&spec, &ast).unwrap_err();

        assert_eq!(
            error,
            TypeCheckError::UninitializedVariable {
                function: "foo".to_string(),
                variable: "y".to_string(),
            }
        );
    }

    #[test]
    fn allows_declared_then_assigned_strings() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Str".to_string(),
                "# post: $result eq $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    my $y;\n    $y = $x;\n    return $y;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();

        type_check_function(&spec, &ast).unwrap();
    }

    #[test]
    fn rejects_string_and_int_mixing() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Int".to_string(),
                "# post: $result >= 0".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $x + 1;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        let error = type_check_function(&spec, &ast).unwrap_err();

        assert_eq!(
            error,
            TypeCheckError::TypeMismatch {
                function: "foo".to_string(),
                context: "arithmetic operand",
                expected: "Int",
                found: "Str",
            }
        );
    }

    #[test]
    fn accepts_division_without_static_nonzero_proof() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int, Int) -> Int".to_string(),
                "# post: $result == $x".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    return $x / $y;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        type_check_function(&spec, &ast).unwrap();
    }

    #[test]
    fn accepts_modulo_without_static_nonzero_proof() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int, Int) -> Int".to_string(),
                "# post: $result == $x % $y".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    return $x % $y;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        type_check_function(&spec, &ast).unwrap();
    }

    #[test]
    fn accepts_safe_substring() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Str".to_string(),
                "# post: $result eq substr($x, 0, length($x))".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return substr($x, 0, length($x));\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();

        type_check_function(&spec, &ast).unwrap();
    }

    #[test]
    fn rejects_negative_substring_start() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Str".to_string(),
                "# post: $result eq $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return substr($x, -1, 1);\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        let error = type_check_function(&spec, &ast).unwrap_err();

        assert_eq!(
            error,
            TypeCheckError::NegativeSubstringArgument {
                function: "foo".to_string(),
                argument: "start",
            }
        );
    }

    #[test]
    fn accepts_array_read_and_write_types() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Array<Int>, Int, Int) -> Int".to_string(),
                "# post: $result == $v".to_string(),
            ],
            body: "\n    my ($arr, $i, $v) = @_;\n    $arr[$i] = $v;\n    return $arr[$i];\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        type_check_function(&spec, &ast).unwrap();
    }

    #[test]
    fn rejects_hash_access_with_non_string_key() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Hash<Str, Int>, Int) -> Int".to_string(),
                "# post: $result >= 0".to_string(),
            ],
            body: "\n    my ($h, $k) = @_;\n    return $h{$k};\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        let error = type_check_function(&spec, &ast).unwrap_err();

        assert_eq!(
            error,
            TypeCheckError::TypeMismatch {
                function: "foo".to_string(),
                context: "hash key",
                expected: "Str",
                found: "Int",
            }
        );
    }
}
