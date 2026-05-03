use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use tracing::debug;

use crate::ast::{AccessKind, BinaryOp, Builtin, Expr, FunctionAst, UnaryOp};

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
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IrError {
    #[error(
        "function `{function}` references unsupported variable `{variable}` during SSA lowering"
    )]
    UnknownVariable { function: String, variable: String },
}

pub fn lower_to_ssa(function: &FunctionAst) -> std::result::Result<SsaFunction, IrError> {
    let mut builder = SsaBuilder::new(&function.name);
    let mut env = BTreeMap::new();
    let mut params = Vec::new();

    for param in &function.params {
        let ssa_name = builder.fresh_name(param);
        env.insert(param.clone(), ssa_name.clone());
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

fn base_name(ssa_name: &str) -> &str {
    ssa_name
        .split_once("__")
        .map(|(base, _)| base)
        .unwrap_or(ssa_name)
}


struct SsaBuilder<'a> {
    function: &'a str,
    versions: BTreeMap<String, usize>,
    /// Tracks the current SSA name for each array's length companion variable.
    /// Key: array base name (e.g., "arr"), Value: current SSA name (e.g., "arr__len__1").
    len_companions: BTreeMap<String, String>,
}

impl<'a> SsaBuilder<'a> {
    fn new(function: &'a str) -> Self {
        Self {
            function,
            versions: BTreeMap::new(),
            len_companions: BTreeMap::new(),
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
                SsaExpr::Builtin {
                    function: *function,
                    args: args
                        .iter()
                        .map(|arg| self.rewrite_expr(arg, env, prefix))
                        .collect::<std::result::Result<Vec<_>, _>>()?,
                }
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
                crate::ast::Stmt::Declare { .. } => {}
                crate::ast::Stmt::LoopBoundExceeded => lowered.push(SsaStmt::LoopBoundExceeded),
                crate::ast::Stmt::Assign { name, expr, .. } => {
                    let mut prefix = Vec::new();
                    let rhs = self.rewrite_expr(expr, &mut env, &mut prefix)?;
                    lowered.extend(prefix);
                    let ssa_name = self.fresh_name(name);
                    env.insert(name.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, rhs));
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
                        index: Box::new(key_expr),
                        value: Box::new(value_expr),
                    };
                    let ssa_name = self.fresh_name(name);
                    env.insert(name.clone(), ssa_name.clone());
                    lowered.push(SsaStmt::Assign(ssa_name, store));
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
                crate::ast::Stmt::Die(_) => {
                    lowered.push(SsaStmt::Die);
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
        assert_eq!(ssa.body.len(), 2);
        assert!(matches!(ssa.body[0], super::SsaStmt::Assign(_, _)));
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
