use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use tracing::debug;

use crate::ast::{AccessKind, BinaryOp, Builtin, Expr, FunctionAst, Stmt, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaParam {
    pub source: String,
    pub ssa_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaFunction {
    pub name: String,
    pub params: Vec<SsaParam>,
    pub body: Vec<SsaStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaStmt {
    Assign(String, SsaExpr),
    CallAssign {
        name: String,
        callee: String,
        args: Vec<SsaExpr>,
    },
    If {
        condition: SsaExpr,
        then_branch: Vec<SsaStmt>,
        else_branch: Vec<SsaStmt>,
        merges: Vec<SsaMerge>,
    },
    Return(SsaExpr),
    LoopBoundExceeded,
    Die,
    /// Silently prune this path (used for loop invariant exit assumptions).
    Unreachable,
    /// Loop invariant initialization check failed.
    InvariantInitFailed,
    /// Loop invariant preservation check failed.
    InvariantPreservationFailed,
    /// Mid-function assertion failed.
    AssertFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsaMerge {
    pub source: String,
    pub then_name: String,
    pub else_name: String,
    pub result_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsaExpr {
    Int(i64),
    Bool(bool),
    String(String),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<SsaExpr>,
    },
    Binary {
        left: Box<SsaExpr>,
        op: BinaryOp,
        right: Box<SsaExpr>,
    },
    Ite {
        condition: Box<SsaExpr>,
        then_expr: Box<SsaExpr>,
        else_expr: Box<SsaExpr>,
    },
    Access {
        kind: AccessKind,
        collection: Box<SsaExpr>,
        index: Box<SsaExpr>,
    },
    Store {
        kind: AccessKind,
        collection: Box<SsaExpr>,
        index: Box<SsaExpr>,
        value: Box<SsaExpr>,
    },
    Builtin {
        function: Builtin,
        args: Vec<SsaExpr>,
    },
    /// A fresh unconstrained scalar variable, used for loop invariant induction.
    FreshVar(String),
    /// A fresh unconstrained array, used as the base when declaring a local array.
    FreshArray {
        /// Whether elements are Int or Str.
        element_int: bool,
        /// The base name used to create the symbolic variable name.
        name: String,
    },
    /// A fresh unconstrained hash (Array<String, Int> or Array<String, String>).
    /// Used for exists companion arrays and other hash-shaped companions.
    FreshHash {
        /// Whether values are Int or Str.
        value_int: bool,
        /// The base name used to create the symbolic variable name.
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowGraph {
    pub name: String,
    pub params: Vec<SsaParam>,
    pub entry: usize,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub id: usize,
    pub params: Vec<BlockParam>,
    pub stmts: Vec<BlockStmt>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockParam {
    pub source: String,
    pub ssa_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStmt {
    Assign(String, SsaExpr),
    CallAssign {
        name: String,
        callee: String,
        args: Vec<SsaExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminator {
    Goto {
        target: usize,
        args: Vec<SsaExpr>,
    },
    Branch {
        condition: SsaExpr,
        then_target: usize,
        else_target: usize,
    },
    Return(SsaExpr),
    LoopBoundExceeded,
    Die,
    Unreachable,
    InvariantInitFailed,
    InvariantPreservationFailed,
    AssertFailed,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IrError {
    #[error(
        "function `{function}` references unsupported variable `{variable}` during SSA lowering"
    )]
    UnknownVariable { function: String, variable: String },
    #[error(
        "function `{function}`: ghost variable `{variable}` shadows a real variable"
    )]
    GhostShadowsReal { function: String, variable: String },
}

pub fn lower_to_ssa(function: &FunctionAst) -> std::result::Result<SsaFunction, IrError> {
    let mut builder = SsaBuilder::new(&function.name);
    let mut env = BTreeMap::new();
    let mut params = Vec::new();

    for param in &function.params {
        let ssa_name = builder.fresh_name(param);
        env.insert(param.clone(), ssa_name.clone());
        builder.real_vars.insert(param.clone());
        params.push(SsaParam {
            source: param.clone(),
            ssa_name,
        });
    }

    let (body, _) = builder.lower_stmts(&function.body, &env)?;
    debug!(function = function.name, "lowered AST to SSA");

    Ok(SsaFunction {
        name: function.name.clone(),
        params,
        body,
    })
}

pub fn build_cfg(function: &SsaFunction) -> ControlFlowGraph {
    let mut builder = CfgBuilder::new(function);
    let mut env = BTreeMap::new();
    for param in &function.params {
        env.insert(param.source.clone(), param.ssa_name.clone());
    }
    builder.lower_sequence(0, &function.body, &env);
    builder.finish()
}

/// Heuristic to determine whether an SSA expression represents an Int value.
/// Used when creating a FreshArray to know whether the array is Array<Int> or Array<Str>.
fn is_int_element(expr: &SsaExpr) -> bool {
    match expr {
        SsaExpr::Int(_) => true,
        SsaExpr::String(_) => false,
        SsaExpr::Bool(_) => true, // bools are represented as ints
        SsaExpr::Unary { .. } | SsaExpr::Binary { .. } => true, // arithmetic produces ints
        SsaExpr::Ite { then_expr, .. } => is_int_element(then_expr),
        SsaExpr::Builtin { function, .. } => {
            // Most builtins return Int, except string-producing ones
            !matches!(function,
                Builtin::Chr | Builtin::Reverse | Builtin::Replace | Builtin::CharAt
            )
        }
        SsaExpr::Access { kind, .. } => {
            // Array/hash access -- we don't know the element type, default to Int
            matches!(kind, AccessKind::Array)
        }
        SsaExpr::Store { .. } => true,
        SsaExpr::Var(_) => true, // assume Int by default for variables
        SsaExpr::FreshVar(_) => true,
        SsaExpr::FreshArray { .. } => true,
        SsaExpr::FreshHash { .. } => true,
    }
}

fn base_name(ssa_name: &str) -> &str {
    ssa_name
        .split_once("__")
        .map(|(base, _)| base)
        .unwrap_or(ssa_name)
}

/// Collect all variable names that are assigned (modified) in a statement list.
/// Used to determine which variables need freshening in loop invariant induction.
fn collect_assigned_vars(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut assigned = BTreeSet::new();
    for stmt in stmts {
        match stmt {
            Stmt::Assign { name, .. } => { assigned.insert(name.clone()); }
            Stmt::ArrayAssign { name, .. } => { assigned.insert(name.clone()); }
            Stmt::HashAssign { name, .. } => { assigned.insert(name.clone()); }
            Stmt::Push { array, .. } => { assigned.insert(array.clone()); }
            Stmt::ArrayInit { name, .. } => { assigned.insert(name.clone()); }
            Stmt::DerefAssign { ref_name, .. } => { assigned.insert(ref_name.clone()); }
            Stmt::ArrowArrayAssign { ref_var, .. } => { assigned.insert(ref_var.clone()); }
            Stmt::ArrowHashAssign { ref_var, .. } => { assigned.insert(ref_var.clone()); }
            Stmt::GhostAssign { name, .. } => { assigned.insert(name.clone()); }
            Stmt::If { then_branch, else_branch, .. } => {
                assigned.extend(collect_assigned_vars(then_branch));
                assigned.extend(collect_assigned_vars(else_branch));
            }
            Stmt::While { body, step, .. } => {
                assigned.extend(collect_assigned_vars(body));
                assigned.extend(collect_assigned_vars(step));
            }
            Stmt::Declare { .. } | Stmt::Return(_) | Stmt::LoopBoundExceeded
            | Stmt::Last | Stmt::Next | Stmt::Die(_) | Stmt::Assert(_) => {}
        }
    }
    assigned
}

struct SsaBuilder<'a> {
    function: &'a str,
    versions: BTreeMap<String, usize>,
    /// Tracks the current SSA name for each array's length companion variable.
    /// Key: array base name (e.g., "arr"), Value: current SSA name (e.g., "arr__len__1").
    len_companions: BTreeMap<String, String>,
    /// Tracks which variables have a definedness companion.
    /// The companion is stored in env under the key "varname__defined".
    defined_companions: BTreeSet<String>,
    /// Tracks the current SSA name for each hash's existence companion array.
    /// Key: hash base name (e.g., "h"), Value: current SSA name (e.g., "h__exists__1").
    /// The companion is a Hash<Str, Int> where 1 = key exists, 0 = does not exist.
    exists_companions: BTreeMap<String, String>,
    /// Tracks static reference aliases: ref_name -> target_name.
    /// When we see `$ref = \$x`, we record alias_map["ref"] = "x".
    alias_map: BTreeMap<String, String>,
    /// Tracks real (non-ghost) variable names so ghost assignments cannot shadow them.
    real_vars: BTreeSet<String>,
}

impl<'a> SsaBuilder<'a> {
    fn new(function: &'a str) -> Self {
        Self {
            function,
            versions: BTreeMap::new(),
            len_companions: BTreeMap::new(),
            defined_companions: BTreeSet::new(),
            exists_companions: BTreeMap::new(),
            alias_map: BTreeMap::new(),
            real_vars: BTreeSet::new(),
        }
    }

    fn fresh_name(&mut self, base: &str) -> String {
        let counter = self.versions.entry(base.to_string()).or_insert(0);
        let name = format!("{base}__{counter}");
        *counter += 1;
        name
    }

    fn rewrite_expr(
        &mut self,
        expr: &Expr,
        env: &mut BTreeMap<String, String>,
        prefix: &mut Vec<SsaStmt>,
    ) -> std::result::Result<SsaExpr, IrError> {
        Ok(match expr {
            Expr::Int(value) => SsaExpr::Int(*value),
            Expr::Bool(value) => SsaExpr::Bool(*value),
            Expr::String(value) => SsaExpr::String(value.clone()),
            Expr::Variable(name) => {
                SsaExpr::Var(env.get(name).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: name.clone(),
                })?)
            }
            Expr::Unary { op, expr } => SsaExpr::Unary {
                op: *op,
                expr: Box::new(self.rewrite_expr(expr, env, prefix)?),
            },
            Expr::Binary { left, op: BinaryOp::Repeat, right } => {
                // Desugar: $s x N => repeated concatenation (constant N only)
                let n = match right.as_ref() {
                    Expr::Int(n) => *n,
                    Expr::Unary { op: UnaryOp::Neg, expr } => match expr.as_ref() {
                        Expr::Int(n) => -(*n),
                        _ => return Err(IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: "x operator requires a constant integer count".to_string(),
                        }),
                    },
                    _ => return Err(IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: "x operator requires a constant integer count".to_string(),
                    }),
                };
                let lhs = self.rewrite_expr(left, env, prefix)?;
                if n <= 0 {
                    SsaExpr::String(String::new())
                } else if n == 1 {
                    lhs
                } else {
                    let mut result = lhs.clone();
                    for _ in 1..n {
                        result = SsaExpr::Binary {
                            left: Box::new(result),
                            op: BinaryOp::Concat,
                            right: Box::new(lhs.clone()),
                        };
                    }
                    result
                }
            }
            Expr::Binary { left, op: BinaryOp::Spaceship, right } => {
                let lhs = self.rewrite_expr(left, env, prefix)?;
                let rhs = self.rewrite_expr(right, env, prefix)?;
                // Desugar: ($a <=> $b) => (($a < $b) ? -1 : (($a == $b) ? 0 : 1))
                SsaExpr::Ite {
                    condition: Box::new(SsaExpr::Binary {
                        left: Box::new(lhs.clone()),
                        op: BinaryOp::Lt,
                        right: Box::new(rhs.clone()),
                    }),
                    then_expr: Box::new(SsaExpr::Int(-1)),
                    else_expr: Box::new(SsaExpr::Ite {
                        condition: Box::new(SsaExpr::Binary {
                            left: Box::new(lhs),
                            op: BinaryOp::Eq,
                            right: Box::new(rhs),
                        }),
                        then_expr: Box::new(SsaExpr::Int(0)),
                        else_expr: Box::new(SsaExpr::Int(1)),
                    }),
                }
            }
            Expr::Binary { left, op: BinaryOp::Cmp, right } => {
                let lhs = self.rewrite_expr(left, env, prefix)?;
                let rhs = self.rewrite_expr(right, env, prefix)?;
                // Desugar: ($a cmp $b) => (($a lt $b) ? -1 : (($a eq $b) ? 0 : 1))
                SsaExpr::Ite {
                    condition: Box::new(SsaExpr::Binary {
                        left: Box::new(lhs.clone()),
                        op: BinaryOp::StrLt,
                        right: Box::new(rhs.clone()),
                    }),
                    then_expr: Box::new(SsaExpr::Int(-1)),
                    else_expr: Box::new(SsaExpr::Ite {
                        condition: Box::new(SsaExpr::Binary {
                            left: Box::new(lhs),
                            op: BinaryOp::StrEq,
                            right: Box::new(rhs),
                        }),
                        then_expr: Box::new(SsaExpr::Int(0)),
                        else_expr: Box::new(SsaExpr::Int(1)),
                    }),
                }
            }
            Expr::Binary { left, op: BinaryOp::And, right } => {
                // Desugar short-circuit: A && B => if A { B } else { false }
                // This ensures calls in B are only executed when A is true.
                // Only desugar when the RHS produces hoisted statements (function calls);
                // otherwise keep as a plain Binary expr for efficiency.
                let left_val = self.rewrite_expr(left, env, prefix)?;
                let mut rhs_prefix = Vec::new();
                let right_val = self.rewrite_expr(right, env, &mut rhs_prefix)?;
                if rhs_prefix.is_empty() {
                    // No side effects in RHS — use plain And (semantically equivalent)
                    SsaExpr::Binary {
                        left: Box::new(left_val),
                        op: BinaryOp::And,
                        right: Box::new(right_val),
                    }
                } else {
                    let then_name = self.fresh_name("__sc_and");
                    let else_name = self.fresh_name("__sc_and");
                    let result_name = self.fresh_name("__sc_and");
                    let mut then_branch = rhs_prefix;
                    then_branch.push(SsaStmt::Assign(then_name.clone(), right_val));
                    let else_branch = vec![SsaStmt::Assign(else_name.clone(), SsaExpr::Bool(false))];
                    prefix.push(SsaStmt::If {
                        condition: left_val,
                        then_branch,
                        else_branch,
                        merges: vec![SsaMerge {
                            source: "__sc_and".to_string(),
                            then_name,
                            else_name,
                            result_name: result_name.clone(),
                        }],
                    });
                    SsaExpr::Var(result_name)
                }
            }
            Expr::Binary { left, op: BinaryOp::Or, right } => {
                // Desugar short-circuit: A || B => if A { true } else { B }
                // This ensures calls in B are only executed when A is false.
                // Only desugar when the RHS produces hoisted statements (function calls);
                // otherwise keep as a plain Binary expr for efficiency.
                let left_val = self.rewrite_expr(left, env, prefix)?;
                let mut rhs_prefix = Vec::new();
                let right_val = self.rewrite_expr(right, env, &mut rhs_prefix)?;
                if rhs_prefix.is_empty() {
                    // No side effects in RHS — use plain Or (semantically equivalent)
                    SsaExpr::Binary {
                        left: Box::new(left_val),
                        op: BinaryOp::Or,
                        right: Box::new(right_val),
                    }
                } else {
                    let then_name = self.fresh_name("__sc_or");
                    let else_name = self.fresh_name("__sc_or");
                    let result_name = self.fresh_name("__sc_or");
                    let then_branch = vec![SsaStmt::Assign(then_name.clone(), SsaExpr::Bool(true))];
                    let mut else_branch = rhs_prefix;
                    else_branch.push(SsaStmt::Assign(else_name.clone(), right_val));
                    prefix.push(SsaStmt::If {
                        condition: left_val,
                        then_branch,
                        else_branch,
                        merges: vec![SsaMerge {
                            source: "__sc_or".to_string(),
                            then_name,
                            else_name,
                            result_name: result_name.clone(),
                        }],
                    });
                    SsaExpr::Var(result_name)
                }
            }
            Expr::Binary { left, op, right } => SsaExpr::Binary {
                left: Box::new(self.rewrite_expr(left, env, prefix)?),
                op: *op,
                right: Box::new(self.rewrite_expr(right, env, prefix)?),
            },
            Expr::Ternary { condition, then_expr, else_expr } => SsaExpr::Ite {
                condition: Box::new(self.rewrite_expr(condition, env, prefix)?),
                then_expr: Box::new(self.rewrite_expr(then_expr, env, prefix)?),
                else_expr: Box::new(self.rewrite_expr(else_expr, env, prefix)?),
            },
            Expr::Access {
                kind,
                collection,
                index,
            } => SsaExpr::Access {
                kind: *kind,
                collection: Box::new(SsaExpr::Var(
                    env.get(collection).cloned().ok_or_else(|| IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: collection.clone(),
                    })?,
                )),
                index: Box::new(self.rewrite_expr(index, env, prefix)?),
            },
            Expr::Call {
                function: callee,
                args,
            } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.rewrite_expr(arg, env, prefix))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let temp = self.fresh_name("__call_result");
                prefix.push(SsaStmt::CallAssign {
                    name: temp.clone(),
                    callee: callee.clone(),
                    args: lowered_args,
                });
                SsaExpr::Var(temp)
            }
            Expr::Builtin { function, args } => {
                // If this is a Scalar call and we have a tracked len companion, use it
                if *function == crate::ast::Builtin::Scalar {
                    if let Some(Expr::Variable(arr_name)) = args.first() {
                        if let Some(len_ssa) = self.len_companions.get(arr_name) {
                            return Ok(SsaExpr::Var(len_ssa.clone()));
                        }
                    }
                }
                // If this is a Defined call, look up the companion variable in env
                if *function == crate::ast::Builtin::Defined {
                    if let Some(Expr::Variable(var_name)) = args.first() {
                        let def_key = format!("{var_name}__defined");
                        if let Some(def_ssa) = env.get(&def_key) {
                            return Ok(SsaExpr::Var(def_ssa.clone()));
                        }
                        // No companion found — variable is a function parameter or
                        // was never tracked. Parameters are always defined.
                        return Ok(SsaExpr::Int(1));
                    }
                    // defined() on a non-variable expression: always defined
                    return Ok(SsaExpr::Int(1));
                }
                // Chomp side-effect: chomp($var) modifies $var in place by
                // removing a trailing newline. We desugar it to:
                //   $var = ends_with($var, "\n") ? substr($var, 0, length($var)-1) : $var
                //   chomp_result = ends_with($var_old, "\n") ? 1 : 0
                if *function == crate::ast::Builtin::Chomp {
                    if let Some(Expr::Variable(var_name)) = args.first() {
                        let old_ssa = env.get(var_name).cloned().ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: var_name.clone(),
                        })?;
                        let old_var = SsaExpr::Var(old_ssa);
                        // ends_with($var, "\n") — returns 1 or 0
                        let has_newline = SsaExpr::Builtin {
                            function: crate::ast::Builtin::EndsWith,
                            args: vec![old_var.clone(), SsaExpr::String("\n".to_string())],
                        };
                        // Convert to Bool for ITE condition: ends_with(...) > 0
                        let condition = SsaExpr::Binary {
                            left: Box::new(has_newline),
                            op: BinaryOp::Gt,
                            right: Box::new(SsaExpr::Int(0)),
                        };
                        // substr($var, 0, length($var) - 1)
                        let var_len = SsaExpr::Builtin {
                            function: crate::ast::Builtin::Length,
                            args: vec![old_var.clone()],
                        };
                        let len_minus_one = SsaExpr::Binary {
                            left: Box::new(var_len),
                            op: BinaryOp::Sub,
                            right: Box::new(SsaExpr::Int(1)),
                        };
                        let chomped = SsaExpr::Builtin {
                            function: crate::ast::Builtin::Substr,
                            args: vec![old_var.clone(), SsaExpr::Int(0), len_minus_one],
                        };
                        // $var_new = has_newline ? chomped : $var
                        let new_value = SsaExpr::Ite {
                            condition: Box::new(condition.clone()),
                            then_expr: Box::new(chomped),
                            else_expr: Box::new(old_var),
                        };
                        let new_ssa = self.fresh_name(var_name);
                        prefix.push(SsaStmt::Assign(new_ssa.clone(), new_value));
                        env.insert(var_name.clone(), new_ssa);
                        // Return the count (1 if newline was removed, 0 otherwise)
                        return Ok(SsaExpr::Ite {
                            condition: Box::new(condition),
                            then_expr: Box::new(SsaExpr::Int(1)),
                            else_expr: Box::new(SsaExpr::Int(0)),
                        });
                    }
                }
                SsaExpr::Builtin {
                    function: *function,
                    args: args
                        .iter()
                        .map(|arg| self.rewrite_expr(arg, env, prefix))
                        .collect::<std::result::Result<Vec<_>, _>>()?,
                }
            }
            Expr::Exists { hash, key } => {
                let key_expr = self.rewrite_expr(key, env, prefix)?;
                // Look up the exists companion for this hash
                let companion_ssa = if let Some(existing) = self.exists_companions.get(hash) {
                    existing.clone()
                } else {
                    // No companion initialized — hash is a parameter or never assigned.
                    // Create a fresh unconstrained companion hash (Array<Str, Int>).
                    let exists_base = format!("{hash}__exists");
                    let fresh_name = self.fresh_name(&exists_base);
                    prefix.push(SsaStmt::Assign(
                        fresh_name.clone(),
                        SsaExpr::FreshHash {
                            value_int: true,
                            name: fresh_name.clone(),
                        },
                    ));
                    self.exists_companions.insert(hash.clone(), fresh_name.clone());
                    fresh_name
                };
                // Normalize to 0/1: ite(select(companion, key) != 0, 1, 0)
                // The companion hash is unconstrained in Z3, so raw select could
                // return any integer. Perl's exists() always returns 1 or 0.
                SsaExpr::Ite {
                    condition: Box::new(SsaExpr::Binary {
                        left: Box::new(SsaExpr::Access {
                            kind: AccessKind::Hash,
                            collection: Box::new(SsaExpr::Var(companion_ssa)),
                            index: Box::new(key_expr),
                        }),
                        op: crate::ast::BinaryOp::Ne,
                        right: Box::new(SsaExpr::Int(0)),
                    }),
                    then_expr: Box::new(SsaExpr::Int(1)),
                    else_expr: Box::new(SsaExpr::Int(0)),
                }
            }
            Expr::Pop { array } => {
                let collection_name = env
                    .get(array)
                    .cloned()
                    .ok_or_else(|| IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: array.clone(),
                    })?;

                // Get or initialize the length companion variable
                let len_base = format!("{array}__len");
                let len_ssa = if let Some(existing) = self.len_companions.get(array) {
                    existing.clone()
                } else {
                    // First pop for this array: initialize len from Scalar builtin
                    let init_name = self.fresh_name(&len_base);
                    let scalar_expr = SsaExpr::Builtin {
                        function: crate::ast::Builtin::Scalar,
                        args: vec![SsaExpr::Var(collection_name.clone())],
                    };
                    prefix.push(SsaStmt::Assign(init_name.clone(), scalar_expr));
                    self.len_companions.insert(array.clone(), init_name.clone());
                    init_name
                };

                // Emit: new_len = ite(old_len > 0, old_len - 1, 0)
                // Guards against negative length when popping an empty array.
                let new_len_name = self.fresh_name(&len_base);
                let decrement = SsaExpr::Ite {
                    condition: Box::new(SsaExpr::Binary {
                        left: Box::new(SsaExpr::Var(len_ssa.clone())),
                        op: BinaryOp::Gt,
                        right: Box::new(SsaExpr::Int(0)),
                    }),
                    then_expr: Box::new(SsaExpr::Binary {
                        left: Box::new(SsaExpr::Var(len_ssa.clone())),
                        op: BinaryOp::Sub,
                        right: Box::new(SsaExpr::Int(1)),
                    }),
                    else_expr: Box::new(SsaExpr::Int(0)),
                };
                prefix.push(SsaStmt::Assign(new_len_name.clone(), decrement));
                self.len_companions.insert(array.clone(), new_len_name.clone());

                // The popped value is at index new_len (i.e., the last element).
                // Capture it into a temp before invalidating the slot.
                let pop_val_name = self.fresh_name(&format!("{array}__pop"));
                let access_expr = SsaExpr::Access {
                    kind: AccessKind::Array,
                    collection: Box::new(SsaExpr::Var(collection_name.clone())),
                    index: Box::new(SsaExpr::Var(new_len_name.clone())),
                };
                prefix.push(SsaStmt::Assign(pop_val_name.clone(), access_expr));

                // Invalidate the popped slot by storing a fresh unconstrained value.
                // This prevents unsound reads at indices >= scalar(@arr).
                let ghost_name = self.fresh_name(&format!("{array}__ghost"));
                prefix.push(SsaStmt::Assign(ghost_name.clone(), SsaExpr::FreshVar(ghost_name.clone())));
                let invalidation_store = SsaExpr::Store {
                    kind: AccessKind::Array,
                    collection: Box::new(SsaExpr::Var(collection_name)),
                    index: Box::new(SsaExpr::Var(new_len_name.clone())),
                    value: Box::new(SsaExpr::Var(ghost_name)),
                };
                let new_arr_name = self.fresh_name(array);
                prefix.push(SsaStmt::Assign(new_arr_name.clone(), invalidation_store));
                env.insert(array.clone(), new_arr_name);

                // Return: ite(old_len > 0, pop_val, 0)
                // In Perl, pop on empty array returns undef (0 in numeric context).
                SsaExpr::Ite {
                    condition: Box::new(SsaExpr::Binary {
                        left: Box::new(SsaExpr::Var(len_ssa)),
                        op: BinaryOp::Gt,
                        right: Box::new(SsaExpr::Int(0)),
                    }),
                    then_expr: Box::new(SsaExpr::Var(pop_val_name)),
                    else_expr: Box::new(SsaExpr::Int(0)),
                }
            }
            Expr::Ref(_target) | Expr::RefArray(_target) | Expr::RefHash(_target) => {
                // Reference creation is handled at the Stmt::Assign level.
                // If we reach here, the ref is used in a non-assignment context,
                // which shouldn't happen after type checking. Treat as a no-op.
                SsaExpr::Int(0)
            }
            Expr::ArrowArrayAccess { ref_var, index } => {
                // Resolve alias to find the target array, then emit Select
                let target = self.alias_map.get(ref_var).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: format!("{}->[] (not a known reference)", ref_var),
                })?;
                let collection_name = env.get(&target).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: target,
                })?;
                SsaExpr::Access {
                    kind: AccessKind::Array,
                    collection: Box::new(SsaExpr::Var(collection_name)),
                    index: Box::new(self.rewrite_expr(index, env, prefix)?),
                }
            }
            Expr::ArrowHashAccess { ref_var, key } => {
                // Resolve alias to find the target hash, then emit Select
                let target = self.alias_map.get(ref_var).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: format!("{}->{{}} (not a known reference)", ref_var),
                })?;
                let collection_name = env.get(&target).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: target,
                })?;
                SsaExpr::Access {
                    kind: AccessKind::Hash,
                    collection: Box::new(SsaExpr::Var(collection_name)),
                    index: Box::new(self.rewrite_expr(key, env, prefix)?),
                }
            }
            Expr::Deref(ref_name) => {
                // Dereference: look up the alias in the alias_map to find the target,
                // then resolve the target's current SSA name.
                let target = self.alias_map.get(ref_name).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: format!("$${} (not a known reference)", ref_name),
                })?;
                SsaExpr::Var(env.get(&target).cloned().ok_or_else(|| IrError::UnknownVariable {
                    function: self.function.to_string(),
                    variable: target,
                })?)
            }
        })
    }

    fn lower_stmts(
        &mut self,
        stmts: &[crate::ast::Stmt],
        env: &BTreeMap<String, String>,
    ) -> std::result::Result<(Vec<SsaStmt>, BTreeMap<String, String>), IrError> {
        let mut env = env.clone();
        let mut lowered = Vec::new();

        for stmt in stmts {
            match stmt {
                crate::ast::Stmt::Declare { name } => {
                    // Emit companion definedness variable: name__defined = 0
                    let def_base = format!("{name}__defined");
                    let def_name = self.fresh_name(&def_base);
                    lowered.push(SsaStmt::Assign(def_name.clone(), SsaExpr::Int(0)));
                    env.insert(def_base, def_name);
                    self.defined_companions.insert(name.clone());
                    self.real_vars.insert(name.clone());
                }
                crate::ast::Stmt::LoopBoundExceeded => lowered.push(SsaStmt::LoopBoundExceeded),
                crate::ast::Stmt::Assign { name, expr, .. } => {
                    self.real_vars.insert(name.clone());
                    // Check if this is a reference assignment: $ref = \$target / \@arr / \%hash
                    if let crate::ast::Expr::Ref(target) = expr {
                        // Record the alias and skip emitting any SSA assignment.
                        // The reference variable doesn't exist at IR level.
                        self.alias_map.insert(name.clone(), target.clone());
                    } else if let crate::ast::Expr::RefArray(target) = expr {
                        self.alias_map.insert(name.clone(), target.clone());
                    } else if let crate::ast::Expr::RefHash(target) = expr {
                        self.alias_map.insert(name.clone(), target.clone());
                    } else {
                        let mut prefix = Vec::new();
                        let rhs = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                        lowered.extend(prefix);
                        let ssa_name = self.fresh_name(name);
                        env.insert(name.clone(), ssa_name.clone());
                        lowered.push(SsaStmt::Assign(ssa_name, rhs));
                        // Emit companion definedness variable: name__defined = 1
                        if self.defined_companions.contains(name) {
                            let def_base = format!("{name}__defined");
                            let def_name = self.fresh_name(&def_base);
                            lowered.push(SsaStmt::Assign(def_name.clone(), SsaExpr::Int(1)));
                            env.insert(def_base, def_name);
                        }
                    }
                }
                crate::ast::Stmt::ArrayAssign { name, index, expr } => {
                    let collection_name = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: name.clone(),
                        })?;
                    let mut prefix = Vec::new();
                    let index_expr = self.rewrite_expr(index, &mut env, &mut prefix)?;
                    let value_expr = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let store = SsaExpr::Store {
                        kind: AccessKind::Array,
                        collection: Box::new(SsaExpr::Var(collection_name)),
                        index: Box::new(index_expr),
                        value: Box::new(value_expr),
                    };
                    let ssa_name = self.fresh_name(name);
                    env.insert(name.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, store));
                }
                crate::ast::Stmt::HashAssign { name, key, expr } => {
                    let collection_name = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: name.clone(),
                        })?;
                    let mut prefix = Vec::new();
                    let key_expr = self.rewrite_expr(key, &mut env, &mut prefix)?;
                    let value_expr = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let store = SsaExpr::Store {
                        kind: AccessKind::Hash,
                        collection: Box::new(SsaExpr::Var(collection_name)),
                        index: Box::new(key_expr.clone()),
                        value: Box::new(value_expr),
                    };
                    let ssa_name = self.fresh_name(name);
                    env.insert(name.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, store));

                    // Update the exists companion: store 1 for this key
                    let exists_base = format!("{name}__exists");
                    let old_exists = if let Some(existing) = self.exists_companions.get(name) {
                        existing.clone()
                    } else {
                        // First hash assignment — create a fresh unconstrained companion
                        let fresh_name = self.fresh_name(&exists_base);
                        lowered.push(SsaStmt::Assign(
                            fresh_name.clone(),
                            SsaExpr::FreshHash {
                                value_int: true,
                                name: fresh_name.clone(),
                            },
                        ));
                        self.exists_companions.insert(name.clone(), fresh_name.clone());
                        fresh_name
                    };
                    let exists_store = SsaExpr::Store {
                        kind: AccessKind::Hash,
                        collection: Box::new(SsaExpr::Var(old_exists)),
                        index: Box::new(key_expr),
                        value: Box::new(SsaExpr::Int(1)),
                    };
                    let new_exists_name = self.fresh_name(&exists_base);
                    lowered.push(SsaStmt::Assign(new_exists_name.clone(), exists_store));
                    self.exists_companions.insert(name.clone(), new_exists_name);
                }
                crate::ast::Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let mut prefix = Vec::new();
                    let condition = self.rewrite_expr(condition, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let (then_branch, then_env) = self.lower_stmts(then_branch, &env)?;
                    let (else_branch, else_env) = self.lower_stmts(else_branch, &env)?;
                    let mut merges = Vec::new();
                    let mut next_env = env.clone();
                    let keys = then_env
                        .keys()
                        .chain(else_env.keys())
                        .cloned()
                        .collect::<BTreeSet<_>>();

                    for key in keys {
                        let then_name = then_env.get(&key).cloned();
                        let else_name = else_env.get(&key).cloned();
                        match (then_name, else_name) {
                            (Some(then_name), Some(else_name)) if then_name == else_name => {
                                next_env.insert(key, then_name);
                            }
                            (Some(then_name), Some(else_name)) => {
                                let result_name = self.fresh_name(&key);
                                merges.push(SsaMerge {
                                    source: key.clone(),
                                    then_name,
                                    else_name,
                                    result_name: result_name.clone(),
                                });
                                next_env.insert(key, result_name);
                            }
                            _ => {}
                        }
                    }

                    lowered.push(SsaStmt::If {
                        condition,
                        then_branch,
                        else_branch,
                        merges,
                    });
                    env = next_env;
                }
                crate::ast::Stmt::Return(expr) => {
                    let mut prefix = Vec::new();
                    let value = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    lowered.push(SsaStmt::Return(value));
                }
                crate::ast::Stmt::Push { array, value } => {
                    let collection_name = env
                        .get(array)
                        .cloned()
                        .ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: array.clone(),
                        })?;

                    // Get or initialize the length companion variable
                    let len_base = format!("{array}__len");
                    let len_ssa = if let Some(existing) = self.len_companions.get(array) {
                        existing.clone()
                    } else {
                        // First push for this array: initialize len from Scalar builtin
                        let init_name = self.fresh_name(&len_base);
                        let scalar_expr = SsaExpr::Builtin {
                            function: crate::ast::Builtin::Scalar,
                            args: vec![SsaExpr::Var(collection_name.clone())],
                        };
                        lowered.push(SsaStmt::Assign(init_name.clone(), scalar_expr));
                        self.len_companions.insert(array.clone(), init_name.clone());
                        init_name
                    };

                    // Lower the value expression
                    let mut prefix = Vec::new();
                    let value_expr = self.rewrite_expr(value, &mut env, &mut prefix)?;
                    lowered.extend(prefix);

                    // Emit: new_arr = Store(old_arr, len, value)
                    let store = SsaExpr::Store {
                        kind: AccessKind::Array,
                        collection: Box::new(SsaExpr::Var(collection_name)),
                        index: Box::new(SsaExpr::Var(len_ssa.clone())),
                        value: Box::new(value_expr),
                    };
                    let new_arr_name = self.fresh_name(array);
                    env.insert(array.clone(), new_arr_name.clone());
                    lowered.push(SsaStmt::Assign(new_arr_name, store));

                    // Emit: new_len = len + 1
                    let new_len_name = self.fresh_name(&len_base);
                    let increment = SsaExpr::Binary {
                        left: Box::new(SsaExpr::Var(len_ssa)),
                        op: BinaryOp::Add,
                        right: Box::new(SsaExpr::Int(1)),
                    };
                    lowered.push(SsaStmt::Assign(new_len_name.clone(), increment));
                    self.len_companions.insert(array.clone(), new_len_name);
                }
                crate::ast::Stmt::ArrayInit { name, elements } => {
                    // Lower all element expressions first
                    let mut element_exprs = Vec::new();
                    for elem in elements {
                        let mut prefix = Vec::new();
                        let value_expr = self.rewrite_expr(elem, &mut env, &mut prefix)?;
                        lowered.extend(prefix);
                        element_exprs.push(value_expr);
                    }

                    // Determine element type from first element expression.
                    let element_int = is_int_element(&element_exprs[0]);

                    // Emit a FreshArray as the base, then build Store chain on top.
                    let arr_name = self.fresh_name(name);
                    env.insert(name.clone(), arr_name.clone());
                    lowered.push(SsaStmt::Assign(arr_name.clone(), SsaExpr::FreshArray {
                        element_int,
                        name: arr_name.clone(),
                    }));

                    // Build stores iteratively: store element 0 at index 0, etc.
                    for (i, value_expr) in element_exprs.into_iter().enumerate() {
                        let collection_name = env
                            .get(name)
                            .cloned()
                            .expect("array must be in env after init");
                        let store = SsaExpr::Store {
                            kind: AccessKind::Array,
                            collection: Box::new(SsaExpr::Var(collection_name)),
                            index: Box::new(SsaExpr::Int(i as i64)),
                            value: Box::new(value_expr),
                        };
                        let new_arr_name = self.fresh_name(name);
                        env.insert(name.clone(), new_arr_name.clone());
                        lowered.push(SsaStmt::Assign(new_arr_name, store));
                    }

                    // Initialize the companion length variable to the number of elements
                    let len_base = format!("{name}__len");
                    let len_init_name = self.fresh_name(&len_base);
                    lowered.push(SsaStmt::Assign(len_init_name.clone(), SsaExpr::Int(elements.len() as i64)));
                    self.len_companions.insert(name.clone(), len_init_name);
                }
                crate::ast::Stmt::While {
                    condition,
                    body: loop_body,
                    step,
                    has_last,
                    has_next,
                    max_unroll,
                    invariant,
                } => {
                    if let Some(inv_expr) = invariant {
                        // Inductive verification using loop invariant.
                        // Desugar into SSA statements that the existing symexec can handle:
                        //
                        // 1. Init check: if (¬invariant) { InvariantInitFailed }
                        // 2. Preservation: create fresh vars, assume inv∧cond,
                        //    execute body, check if (¬invariant_after) { InvariantPreservationFailed }
                        // 3. Post-loop: create fresh vars, assume inv∧¬cond, continue

                        let full_body: Vec<crate::ast::Stmt> = if !step.is_empty() {
                            let mut b = loop_body.clone();
                            b.extend(step.clone());
                            b
                        } else {
                            loop_body.clone()
                        };

                        // --- Step 1: Init check ---
                        // Evaluate the invariant in the current env
                        let mut prefix = Vec::new();
                        let inv_ssa = self.rewrite_expr(inv_expr, &mut env, &mut prefix)?;
                        lowered.extend(prefix);
                        // Negate it: if (¬inv) { InvariantInitFailed }
                        let neg_inv = SsaExpr::Unary {
                            op: UnaryOp::Not,
                            expr: Box::new(inv_ssa),
                        };
                        lowered.push(SsaStmt::If {
                            condition: neg_inv,
                            then_branch: vec![SsaStmt::InvariantInitFailed],
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });

                        // --- Step 2: Preservation check ---
                        // Create fresh symbolic variables only for variables that
                        // are modified by the loop body. Unmodified variables (like
                        // function parameters) keep their original constrained values
                        // so that precondition knowledge is available during the
                        // preservation check.
                        let body_assigned = collect_assigned_vars(&full_body);
                        let pre_body_env = env.clone();
                        let mut fresh_env = env.clone();
                        for (perl_name, _ssa_name) in &pre_body_env {
                            if body_assigned.contains(perl_name) {
                                let fresh_ssa = self.fresh_name(perl_name);
                                fresh_env.insert(perl_name.clone(), fresh_ssa.clone());
                                // Emit an assignment to a fresh unconstrained symbolic variable.
                                lowered.push(SsaStmt::Assign(fresh_ssa.clone(), SsaExpr::FreshVar(fresh_ssa)));
                            }
                        }

                        // Evaluate invariant and condition in the fresh env
                        let mut prefix2 = Vec::new();
                        let fresh_inv = self.rewrite_expr(inv_expr, &mut fresh_env, &mut prefix2)?;
                        let fresh_cond = self.rewrite_expr(condition, &mut fresh_env, &mut prefix2)?;

                        // Execute the body in the fresh env
                        let (body_ssa, body_env) = self.lower_stmts(&full_body, &fresh_env)?;

                        // Evaluate the invariant in the post-body env
                        let mut body_env_for_inv = body_env.clone();
                        let mut prefix3 = Vec::new();
                        let post_inv = self.rewrite_expr(inv_expr, &mut body_env_for_inv, &mut prefix3)?;

                        // Build preservation check:
                        // if (inv ∧ cond) { BODY; prefix3; if (¬post_inv) { InvariantPreservationFailed } }
                        let preservation_cond = SsaExpr::Binary {
                            left: Box::new(fresh_inv),
                            op: BinaryOp::And,
                            right: Box::new(fresh_cond),
                        };

                        let mut preservation_body = Vec::new();
                        preservation_body.extend(prefix2);
                        preservation_body.extend(body_ssa);
                        preservation_body.extend(prefix3);
                        let neg_post_inv = SsaExpr::Unary {
                            op: UnaryOp::Not,
                            expr: Box::new(post_inv),
                        };
                        preservation_body.push(SsaStmt::If {
                            condition: neg_post_inv,
                            then_branch: vec![SsaStmt::InvariantPreservationFailed],
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });

                        lowered.push(SsaStmt::If {
                            condition: preservation_cond,
                            then_branch: preservation_body,
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });

                        // Determine which variables are modified by the loop body
                        // by comparing pre-body and post-body SSA names.
                        let modified_vars: std::collections::BTreeSet<String> = fresh_env
                            .iter()
                            .filter(|(perl_name, pre_ssa)| {
                                body_env.get(*perl_name).map_or(false, |post_ssa| post_ssa != *pre_ssa)
                            })
                            .map(|(perl_name, _)| perl_name.clone())
                            .collect();

                        // --- Step 3: Post-loop ---
                        // Create fresh variables only for modified variables,
                        // representing the state at loop exit where inv∧¬cond holds.
                        // Unmodified variables (like parameters) keep their original SSA name.
                        let mut exit_env = env.clone();
                        for (perl_name, _ssa_name) in &pre_body_env {
                            if modified_vars.contains(perl_name) {
                                let exit_ssa = self.fresh_name(perl_name);
                                exit_env.insert(perl_name.clone(), exit_ssa.clone());
                                lowered.push(SsaStmt::Assign(exit_ssa.clone(), SsaExpr::FreshVar(exit_ssa)));
                            }
                            // Unmodified variables keep their pre-loop SSA name
                        }

                        // Evaluate invariant and condition in the exit env
                        let mut prefix4 = Vec::new();
                        let exit_inv = self.rewrite_expr(inv_expr, &mut exit_env, &mut prefix4)?;
                        let exit_cond = self.rewrite_expr(condition, &mut exit_env, &mut prefix4)?;
                        lowered.extend(prefix4);

                        // Assert inv∧¬cond by adding: if (¬(inv∧¬cond)) { Unreachable/skip }
                        // Actually, we assume it. In the SSA framework, we can add an If
                        // that guards the rest of execution: the symexec will only proceed
                        // along the path where inv∧¬cond holds because we emit a branch.
                        //
                        // Simpler approach: the "exit assumption" becomes a condition on
                        // an If that wraps everything after the loop. But that's complex.
                        //
                        // Simplest approach: emit the exit_inv and exit_cond as assertions
                        // that the symexec will add to the path condition. We do this by
                        // emitting: if (¬inv) { Unreachable } and if (cond) { Unreachable }
                        // These prune paths where the exit assumptions don't hold.
                        let neg_exit_inv = SsaExpr::Unary {
                            op: UnaryOp::Not,
                            expr: Box::new(exit_inv),
                        };
                        lowered.push(SsaStmt::If {
                            condition: neg_exit_inv,
                            then_branch: vec![SsaStmt::Unreachable],
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });
                        // Also assume ¬condition at exit
                        lowered.push(SsaStmt::If {
                            condition: exit_cond,
                            then_branch: vec![SsaStmt::Unreachable],
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });

                        env = exit_env;
                    } else if *has_last || *has_next {
                        // Loops with `last` or `next` use the AST-level unrolling
                        // which handles flag-based desugaring correctly.
                        let unrolled = if !step.is_empty() && *has_next {
                            crate::parser::unroll_for_with_next(
                                condition.clone(),
                                loop_body.clone(),
                                step.clone(),
                                *max_unroll,
                            )
                        } else if !step.is_empty() {
                            let mut full_body = loop_body.clone();
                            full_body.extend(step.clone());
                            crate::parser::unroll_while(condition.clone(), full_body, *max_unroll)
                        } else {
                            crate::parser::unroll_while(
                                condition.clone(),
                                loop_body.clone(),
                                *max_unroll,
                            )
                        };
                        let (unrolled_ssa, unrolled_env) = self.lower_stmts(&unrolled, &env)?;
                        lowered.extend(unrolled_ssa);
                        env = unrolled_env;
                    } else {
                        // SSA-level loop unrolling: avoid creating intermediate AST
                        // nodes by directly producing SsaStmt::If chains. Each
                        // iteration lowers the condition and body fresh with new SSA
                        // names, then emits merges for modified variables. This is
                        // more memory-efficient than AST-level unrolling because no
                        // intermediate Stmt::If chains are allocated.
                        let full_body: Vec<crate::ast::Stmt> = if !step.is_empty() {
                            let mut b = loop_body.clone();
                            b.extend(step.clone());
                            b
                        } else {
                            loop_body.clone()
                        };

                        for _iteration in 0..*max_unroll {
                            let mut prefix = Vec::new();
                            let cond = self.rewrite_expr(condition, &mut env, &mut prefix)?;
                            lowered.extend(prefix);

                            let (body_ssa, body_env) = self.lower_stmts(&full_body, &env)?;

                            // Compute merges: the else branch is empty, so the
                            // "else env" is the pre-body env.
                            let else_env = env.clone();
                            let mut merges = Vec::new();
                            let mut next_env = env.clone();
                            let keys = body_env
                                .keys()
                                .chain(else_env.keys())
                                .cloned()
                                .collect::<BTreeSet<_>>();

                            for key in keys {
                                let then_name = body_env.get(&key).cloned();
                                let else_name = else_env.get(&key).cloned();
                                match (then_name, else_name) {
                                    (Some(tn), Some(en)) if tn == en => {
                                        next_env.insert(key, tn);
                                    }
                                    (Some(tn), Some(en)) => {
                                        let result_name = self.fresh_name(&key);
                                        merges.push(SsaMerge {
                                            source: key.clone(),
                                            then_name: tn,
                                            else_name: en,
                                            result_name: result_name.clone(),
                                        });
                                        next_env.insert(key, result_name);
                                    }
                                    _ => {}
                                }
                            }

                            lowered.push(SsaStmt::If {
                                condition: cond,
                                then_branch: body_ssa,
                                else_branch: Vec::new(),
                                merges,
                            });
                            env = next_env;
                        }

                        // Final bound-exceeded check
                        let mut prefix = Vec::new();
                        let cond = self.rewrite_expr(condition, &mut env, &mut prefix)?;
                        lowered.extend(prefix);
                        lowered.push(SsaStmt::If {
                            condition: cond,
                            then_branch: vec![SsaStmt::LoopBoundExceeded],
                            else_branch: Vec::new(),
                            merges: Vec::new(),
                        });
                    }
                }
                crate::ast::Stmt::Die(_) => {
                    lowered.push(SsaStmt::Die);
                }
                crate::ast::Stmt::GhostAssign { name, expr } => {
                    // Reject ghost variables that shadow real variables — this would
                    // be unsound because subsequent code referencing the variable
                    // would use the ghost value instead of the real value.
                    if self.real_vars.contains(name) {
                        return Err(IrError::GhostShadowsReal {
                            function: self.function.to_string(),
                            variable: name.clone(),
                        });
                    }
                    // Ghost variables are lowered to regular SSA assignments.
                    let mut prefix = Vec::new();
                    let rhs = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let ssa_name = self.fresh_name(name);
                    env.insert(name.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, rhs));
                }
                crate::ast::Stmt::Assert(expr) => {
                    let mut prefix = Vec::new();
                    let assert_expr = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    // Desugar: if (!assert_expr) { AssertFailed }
                    let neg = SsaExpr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(assert_expr),
                    };
                    lowered.push(SsaStmt::If {
                        condition: neg,
                        then_branch: vec![SsaStmt::AssertFailed],
                        else_branch: Vec::new(),
                        merges: Vec::new(),
                    });
                }
                crate::ast::Stmt::ArrowArrayAssign { ref_var, index, expr } => {
                    // Resolve alias to find the target array, then emit Store
                    let target = self.alias_map.get(ref_var).cloned().ok_or_else(|| IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: format!("{}->[] (not a known reference)", ref_var),
                    })?;
                    let collection_name = env
                        .get(&target)
                        .cloned()
                        .ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: target.clone(),
                        })?;
                    let mut prefix = Vec::new();
                    let index_expr = self.rewrite_expr(index, &mut env, &mut prefix)?;
                    let value_expr = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let store = SsaExpr::Store {
                        kind: AccessKind::Array,
                        collection: Box::new(SsaExpr::Var(collection_name)),
                        index: Box::new(index_expr),
                        value: Box::new(value_expr),
                    };
                    let ssa_name = self.fresh_name(&target);
                    env.insert(target, ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, store));
                }
                crate::ast::Stmt::ArrowHashAssign { ref_var, key, expr } => {
                    // Resolve alias to find the target hash, then emit Store + exists companion
                    let target = self.alias_map.get(ref_var).cloned().ok_or_else(|| IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: format!("{}->{{}} (not a known reference)", ref_var),
                    })?;
                    let collection_name = env
                        .get(&target)
                        .cloned()
                        .ok_or_else(|| IrError::UnknownVariable {
                            function: self.function.to_string(),
                            variable: target.clone(),
                        })?;
                    let mut prefix = Vec::new();
                    let key_expr = self.rewrite_expr(key, &mut env, &mut prefix)?;
                    let value_expr = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let store = SsaExpr::Store {
                        kind: AccessKind::Hash,
                        collection: Box::new(SsaExpr::Var(collection_name)),
                        index: Box::new(key_expr.clone()),
                        value: Box::new(value_expr),
                    };
                    let ssa_name = self.fresh_name(&target);
                    env.insert(target.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, store));

                    // Update the exists companion: store 1 for this key
                    let exists_base = format!("{target}__exists");
                    let old_exists = if let Some(existing) = self.exists_companions.get(&target) {
                        existing.clone()
                    } else {
                        let fresh_name = self.fresh_name(&exists_base);
                        lowered.push(SsaStmt::Assign(
                            fresh_name.clone(),
                            SsaExpr::FreshHash {
                                value_int: true,
                                name: fresh_name.clone(),
                            },
                        ));
                        self.exists_companions.insert(target.clone(), fresh_name.clone());
                        fresh_name
                    };
                    let exists_store = SsaExpr::Store {
                        kind: AccessKind::Hash,
                        collection: Box::new(SsaExpr::Var(old_exists)),
                        index: Box::new(key_expr),
                        value: Box::new(SsaExpr::Int(1)),
                    };
                    let new_exists_name = self.fresh_name(&exists_base);
                    lowered.push(SsaStmt::Assign(new_exists_name.clone(), exists_store));
                    self.exists_companions.insert(target, new_exists_name);
                }
                crate::ast::Stmt::DerefAssign { ref_name, expr } => {
                    // Desugar $$ref = expr to target = expr using the alias map
                    let target = self.alias_map.get(ref_name).cloned().ok_or_else(|| IrError::UnknownVariable {
                        function: self.function.to_string(),
                        variable: format!("$${} (not a known reference)", ref_name),
                    })?;
                    let mut prefix = Vec::new();
                    let rhs = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let ssa_name = self.fresh_name(&target);
                    env.insert(target, ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, rhs));
                }
                crate::ast::Stmt::Last => {
                    unreachable!("`last` should be desugared by the parser before IR lowering")
                }
                crate::ast::Stmt::Next => {
                    unreachable!("`next` should be desugared by the parser before IR lowering")
                }
            }
        }

        Ok((lowered, env))
    }
}

struct CfgBuilder<'a> {
    function: &'a SsaFunction,
    blocks: Vec<BasicBlock>,
}

impl<'a> CfgBuilder<'a> {
    fn new(function: &'a SsaFunction) -> Self {
        Self {
            function,
            blocks: vec![BasicBlock {
                id: 0,
                params: Vec::new(),
                stmts: Vec::new(),
                terminator: Terminator::Unreachable,
            }],
        }
    }

    fn finish(self) -> ControlFlowGraph {
        debug!(
            function = self.function.name,
            block_count = self.blocks.len(),
            "built CFG"
        );
        ControlFlowGraph {
            name: self.function.name.clone(),
            params: self.function.params.clone(),
            entry: 0,
            blocks: self.blocks,
        }
    }

    fn new_block(&mut self, params: Vec<BlockParam>) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            params,
            stmts: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        id
    }

    fn lower_sequence(
        &mut self,
        block_id: usize,
        stmts: &[SsaStmt],
        env: &BTreeMap<String, String>,
    ) -> BlockExit {
        let mut env = env.clone();
        let current = block_id;

        for (index, stmt) in stmts.iter().enumerate() {
            match stmt {
                SsaStmt::Assign(name, expr) => {
                    self.blocks[current]
                        .stmts
                        .push(BlockStmt::Assign(name.clone(), expr.clone()));
                    env.insert(base_name(name).to_string(), name.clone());
                }
                SsaStmt::CallAssign { name, callee, args } => {
                    self.blocks[current].stmts.push(BlockStmt::CallAssign {
                        name: name.clone(),
                        callee: callee.clone(),
                        args: args.clone(),
                    });
                    env.insert(base_name(name).to_string(), name.clone());
                }
                SsaStmt::Return(expr) => {
                    self.blocks[current].terminator = Terminator::Return(expr.clone());
                    return BlockExit::terminated(env);
                }
                SsaStmt::LoopBoundExceeded => {
                    self.blocks[current].terminator = Terminator::LoopBoundExceeded;
                    return BlockExit::terminated(env);
                }
                SsaStmt::Die => {
                    self.blocks[current].terminator = Terminator::Die;
                    return BlockExit::terminated(env);
                }
                SsaStmt::Unreachable => {
                    self.blocks[current].terminator = Terminator::Unreachable;
                    return BlockExit::terminated(env);
                }
                SsaStmt::If {
                    condition,
                    then_branch,
                    else_branch,
                    merges,
                } => {
                    let then_block = self.new_block(Vec::new());
                    let else_block = self.new_block(Vec::new());
                    self.blocks[current].terminator = Terminator::Branch {
                        condition: condition.clone(),
                        then_target: then_block,
                        else_target: else_block,
                    };

                    let then_exit = self.lower_sequence(then_block, then_branch, &env);
                    let else_exit = self.lower_sequence(else_block, else_branch, &env);
                    let rest = &stmts[index + 1..];

                    if rest.is_empty() && then_exit.terminated && else_exit.terminated {
                        return BlockExit::terminated(env);
                    }

                    let join_params = merges
                        .iter()
                        .map(|merge| BlockParam {
                            source: merge.source.clone(),
                            ssa_name: merge.result_name.clone(),
                        })
                        .collect::<Vec<_>>();
                    let join_block = self.new_block(join_params);

                    if !then_exit.terminated {
                        self.blocks[then_exit.block_id].terminator = Terminator::Goto {
                            target: join_block,
                            args: merges
                                .iter()
                                .map(|merge| SsaExpr::Var(merge.then_name.clone()))
                                .collect(),
                        };
                    }
                    if !else_exit.terminated {
                        self.blocks[else_exit.block_id].terminator = Terminator::Goto {
                            target: join_block,
                            args: merges
                                .iter()
                                .map(|merge| SsaExpr::Var(merge.else_name.clone()))
                                .collect(),
                        };
                    }

                    for merge in merges {
                        env.insert(merge.source.clone(), merge.result_name.clone());
                    }

                    return self.lower_sequence(join_block, rest, &env);
                }
                SsaStmt::InvariantInitFailed => {
                    self.blocks[current].terminator = Terminator::InvariantInitFailed;
                    return BlockExit::terminated(env);
                }
                SsaStmt::InvariantPreservationFailed => {
                    self.blocks[current].terminator = Terminator::InvariantPreservationFailed;
                    return BlockExit::terminated(env);
                }
                SsaStmt::AssertFailed => {
                    self.blocks[current].terminator = Terminator::AssertFailed;
                    return BlockExit::terminated(env);
                }
            }
        }

        BlockExit::fallthrough(current, env)
    }
}

struct BlockExit {
    block_id: usize,
    terminated: bool,
}

impl BlockExit {
    fn terminated(_env: BTreeMap<String, String>) -> Self {
        Self {
            block_id: 0,
            terminated: true,
        }
    }

    fn fallthrough(block_id: usize, _env: BTreeMap<String, String>) -> Self {
        Self {
            block_id,
            terminated: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        ast::{BinaryOp, Expr, FunctionAst, Stmt},
        parser::parse_function_ast,
    };

    use super::{BlockStmt, Terminator, build_cfg, lower_to_ssa};

    #[test]
    fn ssa_has_unique_assignment_targets() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::Assign {
                    name: "x".to_string(),
                    declaration: false,
                    expr: Expr::Binary {
                        left: Box::new(Expr::Variable("x".to_string())),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Int(1)),
                    },
                },
                Stmt::Assign {
                    name: "x".to_string(),
                    declaration: false,
                    expr: Expr::Binary {
                        left: Box::new(Expr::Variable("x".to_string())),
                        op: BinaryOp::Mul,
                        right: Box::new(Expr::Int(2)),
                    },
                },
                Stmt::Return(Expr::Variable("x".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        let mut names = BTreeSet::new();

        fn visit(stmts: &[super::SsaStmt], names: &mut BTreeSet<String>) {
            for stmt in stmts {
                match stmt {
                    super::SsaStmt::Assign(name, _) => {
                        assert!(names.insert(name.clone()));
                    }
                    super::SsaStmt::CallAssign { name, .. } => {
                        assert!(names.insert(name.clone()));
                    }
                    super::SsaStmt::If {
                        then_branch,
                        else_branch,
                        merges,
                        ..
                    } => {
                        visit(then_branch, names);
                        visit(else_branch, names);
                        for merge in merges {
                            assert!(names.insert(merge.result_name.clone()));
                        }
                    }
                    super::SsaStmt::Return(_) => {}
                    super::SsaStmt::LoopBoundExceeded => {}
                    super::SsaStmt::Die => {}
                    super::SsaStmt::Unreachable => {}
                    super::SsaStmt::InvariantInitFailed => {}
                    super::SsaStmt::InvariantPreservationFailed => {}
                    super::SsaStmt::AssertFailed => {}
                }
            }
        }

        visit(&ssa.body, &mut names);
    }

    #[test]
    fn ssa_versions_reassigned_variables() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::Assign {
                    name: "x".to_string(),
                    declaration: false,
                    expr: Expr::Int(1),
                },
                Stmt::Assign {
                    name: "x".to_string(),
                    declaration: false,
                    expr: Expr::Int(2),
                },
                Stmt::Return(Expr::Variable("x".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        assert_eq!(ssa.params[0].ssa_name, "x__0");
        match &ssa.body[0] {
            super::SsaStmt::Assign(name, _) => assert_eq!(name, "x__1"),
            _ => panic!("expected assignment"),
        }
        match &ssa.body[1] {
            super::SsaStmt::Assign(name, _) => assert_eq!(name, "x__2"),
            _ => panic!("expected assignment"),
        }
    }

    #[test]
    fn cfg_contains_branch_and_merge_block() {
        let extracted = crate::extractor::ExtractedFunction {
            name: "foo".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($x) = @_;
    if ($x > 0) {
        my $y = $x + 1;
        return $y;
    } else {
        my $y = $x - 1;
    }
    return $x;
"#
            .to_string(),
            start_line: 1,
        };

        let ast = parse_function_ast(&extracted).unwrap();
        let ssa = lower_to_ssa(&ast).unwrap();
        let cfg = build_cfg(&ssa);

        assert!(
            cfg.blocks
                .iter()
                .any(|block| matches!(block.terminator, Terminator::Branch { .. }))
        );
        assert!(cfg.blocks.iter().any(|block| !block.params.is_empty()));
    }

    #[test]
    fn cfg_preserves_straight_line_assignments() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::Declare {
                    name: "y".to_string(),
                },
                Stmt::Assign {
                    name: "y".to_string(),
                    declaration: true,
                    expr: Expr::Variable("x".to_string()),
                },
                Stmt::Return(Expr::Variable("y".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        let cfg = build_cfg(&ssa);

        assert!(matches!(cfg.blocks[0].stmts[0], BlockStmt::Assign(_, _)));
        assert!(matches!(cfg.blocks[0].terminator, Terminator::Return(_)));
    }

    #[test]
    fn bare_declaration_does_not_create_ssa_assignment() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::Declare {
                    name: "y".to_string(),
                },
                Stmt::Assign {
                    name: "y".to_string(),
                    declaration: false,
                    expr: Expr::Variable("x".to_string()),
                },
                Stmt::Return(Expr::Variable("y".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        // 4 statements: y__defined=0 (from Declare), y=x (Assign),
        // y__defined=1 (Assign companion), return y
        assert_eq!(ssa.body.len(), 4);
        assert!(matches!(ssa.body[0], super::SsaStmt::Assign(_, _)));
        // First statement is the definedness companion init (y__defined = 0)
        assert!(matches!(&ssa.body[0], super::SsaStmt::Assign(name, super::SsaExpr::Int(0)) if name.starts_with("y__defined")));
        // Second statement is the actual assignment (y = x)
        assert!(matches!(&ssa.body[1], super::SsaStmt::Assign(name, _) if name.starts_with("y__")));
        // Third statement is the definedness companion update (y__defined = 1)
        assert!(matches!(&ssa.body[2], super::SsaStmt::Assign(name, super::SsaExpr::Int(1)) if name.starts_with("y__defined")));
    }

    #[test]
    fn call_in_binary_expression_is_lifted() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::Assign {
                    name: "z".to_string(),
                    declaration: true,
                    expr: Expr::Binary {
                        left: Box::new(Expr::Call {
                            function: "inc".to_string(),
                            args: vec![Expr::Variable("x".to_string())],
                        }),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Int(1)),
                    },
                },
                Stmt::Return(Expr::Variable("z".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        assert!(matches!(ssa.body[0], super::SsaStmt::CallAssign { ref callee, .. } if callee == "inc"));
        assert!(matches!(ssa.body[1], super::SsaStmt::Assign(_, super::SsaExpr::Binary { .. })));
    }

    #[test]
    fn nested_calls_in_arguments_are_lifted_in_order() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![Stmt::Return(Expr::Call {
                function: "outer".to_string(),
                args: vec![Expr::Call {
                    function: "inner".to_string(),
                    args: vec![Expr::Variable("x".to_string())],
                }],
            })],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        assert!(matches!(ssa.body[0], super::SsaStmt::CallAssign { ref callee, .. } if callee == "inner"));
        assert!(matches!(ssa.body[1], super::SsaStmt::CallAssign { ref callee, .. } if callee == "outer"));
    }

    #[test]
    fn call_in_condition_is_lifted_before_if() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![
                Stmt::If {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Call {
                            function: "check".to_string(),
                            args: vec![Expr::Variable("x".to_string())],
                        }),
                        op: BinaryOp::Gt,
                        right: Box::new(Expr::Int(0)),
                    },
                    then_branch: vec![Stmt::Return(Expr::Int(1))],
                    else_branch: vec![Stmt::Return(Expr::Int(0))],
                },
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        assert!(matches!(ssa.body[0], super::SsaStmt::CallAssign { ref callee, .. } if callee == "check"));
        assert!(matches!(ssa.body[1], super::SsaStmt::If { .. }));
    }

    #[test]
    fn multiple_calls_in_expression_are_lifted() {
        let function = FunctionAst {
            name: "foo".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: vec![
                Stmt::Assign {
                    name: "z".to_string(),
                    declaration: true,
                    expr: Expr::Binary {
                        left: Box::new(Expr::Call {
                            function: "f".to_string(),
                            args: vec![Expr::Variable("x".to_string())],
                        }),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::Call {
                            function: "g".to_string(),
                            args: vec![Expr::Variable("y".to_string())],
                        }),
                    },
                },
                Stmt::Return(Expr::Variable("z".to_string())),
            ],
        };

        let ssa = lower_to_ssa(&function).unwrap();
        assert!(matches!(ssa.body[0], super::SsaStmt::CallAssign { ref callee, .. } if callee == "f"));
        assert!(matches!(ssa.body[1], super::SsaStmt::CallAssign { ref callee, .. } if callee == "g"));
        assert!(matches!(ssa.body[2], super::SsaStmt::Assign(_, super::SsaExpr::Binary { .. })));
    }
}
