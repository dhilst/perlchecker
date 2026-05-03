use std::collections::{BTreeMap, VecDeque};

use thiserror::Error;
use tracing::debug;

use crate::{
    annotations::{FunctionSpec, parse_function_spec},
    ast::{AccessKind, Builtin, Expr, Type, type_check_function_with_signatures},
    extractor::ExtractedFunction,
    ir::{self, BlockStmt, ControlFlowGraph, SsaExpr, Terminator},
    limits::Limits,
    parser::parse_function_ast_with_limits,
    smt,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntExpr {
    Const(i64),
    Var(String),
    Add(Box<IntExpr>, Box<IntExpr>),
    Sub(Box<IntExpr>, Box<IntExpr>),
    Mul(Box<IntExpr>, Box<IntExpr>),
    Div(Box<IntExpr>, Box<IntExpr>),
    Mod(Box<IntExpr>, Box<IntExpr>),
    Pow(Box<IntExpr>, Box<IntExpr>),
    BitAnd(Box<IntExpr>, Box<IntExpr>),
    BitOr(Box<IntExpr>, Box<IntExpr>),
    BitXor(Box<IntExpr>, Box<IntExpr>),
    Abs(Box<IntExpr>),
    Ord(Box<StrExpr>),
    Ite(Box<BoolExpr>, Box<IntExpr>, Box<IntExpr>),
    Length(Box<StrExpr>),
    Index(Box<StrExpr>, Box<StrExpr>),
    ArraySelect(Box<ArrayIntExpr>, Box<IntExpr>),
    HashSelect(Box<HashIntExpr>, Box<StrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrExpr {
    Const(String),
    Var(String),
    Concat(Box<StrExpr>, Box<StrExpr>),
    Substr(Box<StrExpr>, Box<IntExpr>, Box<IntExpr>),
    Chr(Box<IntExpr>),
    Ite(Box<BoolExpr>, Box<StrExpr>, Box<StrExpr>),
    ArraySelect(Box<ArrayStrExpr>, Box<IntExpr>),
    HashSelect(Box<HashStrExpr>, Box<StrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayIntExpr {
    Var(String),
    Store(Box<ArrayIntExpr>, Box<IntExpr>, Box<IntExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayStrExpr {
    Var(String),
    Store(Box<ArrayStrExpr>, Box<IntExpr>, Box<StrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashIntExpr {
    Var(String),
    Store(Box<HashIntExpr>, Box<StrExpr>, Box<IntExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashStrExpr {
    Var(String),
    Store(Box<HashStrExpr>, Box<StrExpr>, Box<StrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CmpOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoolExpr {
    Const(bool),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    IntCmp(CmpOp, Box<IntExpr>, Box<IntExpr>),
    StrEq(Box<StrExpr>, Box<StrExpr>),
    StrCmp(CmpOp, Box<StrExpr>, Box<StrExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub env: BTreeMap<String, SymValue>,
    pub path_condition: BoolExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymValue {
    Int(IntExpr),
    Bool(BoolExpr),
    Str(StrExpr),
    ArrayInt(ArrayIntExpr),
    ArrayStr(ArrayStrExpr),
    HashInt(HashIntExpr),
    HashStr(HashStrExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedState {
    pub env: BTreeMap<String, SymValue>,
    pub path_condition: BoolExpr,
    pub result: SymValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelValue {
    Int(i64),
    Str(String),
    Collection(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counterexample {
    pub function: String,
    pub assignments: BTreeMap<String, ModelValue>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub order: Vec<String>,
    pub specs: BTreeMap<String, FunctionSpec>,
    pub cfgs: BTreeMap<String, ControlFlowGraph>,
    pub limits: Limits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Verified { function: String },
    Counterexample(Counterexample),
}

#[derive(Debug, Error)]
pub enum SymExecError {
    #[error("CFG references missing block {block_id} in function `{function}`")]
    MissingBlock { function: String, block_id: usize },
    #[error("symbolic environment is missing `{name}` in function `{function}`")]
    MissingSymbol { function: String, name: String },
    #[error("type mismatch during symbolic evaluation in function `{function}`")]
    TypeMismatch { function: String },
    #[error("function `{function}` calls unknown callee `{callee}`")]
    UnknownCallee { function: String, callee: String },
    #[error("recursive call graph detected involving `{function}`")]
    RecursionDetected { function: String },
    #[error("function `{function}` exceeded the maximum number of symbolic paths ({max_paths})")]
    PathLimitExceeded { function: String, max_paths: usize },
    #[error("function `{function}` exceeded the loop unroll bound on a feasible path")]
    LoopBoundExceeded { function: String },
    #[error("function `{function}` can reach a `die` statement on a feasible path")]
    DieReached { function: String },
    #[error("function `{function}` has no valid execution paths after discarding invalid arithmetic paths")]
    NoValidPaths { function: String },
    #[error(transparent)]
    Ir(#[from] ir::IrError),
    #[error(transparent)]
    Smt(#[from] smt::SmtError),
}

pub fn verify_extracted_function(
    function: &ExtractedFunction,
) -> crate::Result<VerificationResult> {
    verify_extracted_function_with_limits(function, Limits::default())
}

pub fn verify_extracted_function_with_limits(
    function: &ExtractedFunction,
    limits: Limits,
) -> crate::Result<VerificationResult> {
    let results = verify_extracted_functions(std::slice::from_ref(function), limits)?;
    Ok(results
        .into_iter()
        .next()
        .expect("single-function program must produce one result"))
}

pub fn verify_extracted_functions(
    functions: &[ExtractedFunction],
    limits: Limits,
) -> crate::Result<Vec<VerificationResult>> {
    let program = prepare_program(functions, limits)?;
    program
        .order
        .iter()
        .map(|name| {
            let spec = program.specs.get(name).expect("program spec must exist");
            let cfg = program.cfgs.get(name).expect("program cfg must exist");
            verify_cfg(&program, spec, cfg).map_err(crate::PerlcheckerError::from)
        })
        .collect()
}

pub fn execute_cfg(
    program: &Program,
    function: &FunctionSpec,
    cfg: &ControlFlowGraph,
) -> std::result::Result<Vec<CompletedState>, SymExecError> {
    let mut env = BTreeMap::new();
    let mut annotation_env = BTreeMap::new();
    for (cfg_param, ty) in cfg.params.iter().zip(function.arg_types.iter()) {
        let symbolic = symbolic_value(cfg_param.source.as_str(), *ty);
        env.insert(cfg_param.ssa_name.clone(), symbolic.clone());
        annotation_env.insert(cfg_param.source.clone(), symbolic.clone());

        // For array parameters, add companion length variables
        match ty {
            Type::ArrayInt | Type::ArrayStr => {
                let len_var = format!("{}__len", cfg_param.source);
                annotation_env.insert(len_var.clone(), SymValue::Int(IntExpr::Var(len_var)));
            }
            _ => {}
        }
    }
    let initial_path = expect_bool(
        eval_expr(&cfg.name, &function.pre, &annotation_env)?,
        &cfg.name,
    )?;
    execute_cfg_from_state(
        program,
        function,
        cfg,
        State {
            env,
            path_condition: initial_path,
        },
    )
}

fn execute_cfg_from_state(
    program: &Program,
    _function: &FunctionSpec,
    cfg: &ControlFlowGraph,
    initial_state: State,
) -> std::result::Result<Vec<CompletedState>, SymExecError> {
    let mut worklist = VecDeque::new();
    worklist.push_back((cfg.entry, initial_state));
    let mut path_budget = 1usize;

    let mut completed = Vec::new();

    while let Some((block_id, state)) = worklist.pop_front() {
        let block = cfg
            .blocks
            .get(block_id)
            .ok_or_else(|| SymExecError::MissingBlock {
                function: cfg.name.clone(),
                block_id,
            })?;

        let mut active_states = vec![state];
        for stmt in &block.stmts {
            match stmt {
                BlockStmt::Assign(name, expr) => {
                    for state in &mut active_states {
                        let value = eval_ssa_expr(&cfg.name, expr, &state.env)?;
                        state.env.insert(name.clone(), value);
                    }
                }
                BlockStmt::CallAssign { name, callee, args } => {
                    let mut next_states = Vec::new();
                    for state in active_states {
                        let arg_values = args
                            .iter()
                            .map(|arg| eval_ssa_expr(&cfg.name, arg, &state.env))
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        let outcomes =
                            execute_call(program, &cfg.name, callee, arg_values, state.path_condition.clone())?;
                        for outcome in outcomes {
                            let mut env = state.env.clone();
                            env.insert(name.clone(), outcome.result);
                            next_states.push(State {
                                env,
                                path_condition: outcome.path_condition,
                            });
                        }
                    }
                    active_states = next_states;
                }
            }
        }

        for state in active_states {
            match &block.terminator {
                Terminator::Return(expr) => {
                    let result = eval_ssa_expr(&cfg.name, expr, &state.env)?;
                    completed.push(CompletedState {
                        env: state.env,
                        path_condition: state.path_condition,
                        result,
                    });
                }
                Terminator::Goto { target, args } => {
                    let target_block = cfg.blocks.get(*target).ok_or_else(|| SymExecError::MissingBlock {
                        function: cfg.name.clone(),
                        block_id: *target,
                    })?;
                    let mut next_env = state.env.clone();
                    for (param, arg) in target_block.params.iter().zip(args.iter()) {
                        let value = eval_ssa_expr(&cfg.name, arg, &state.env)?;
                        next_env.insert(param.ssa_name.clone(), value);
                    }
                    worklist.push_back((
                        *target,
                        State {
                            env: next_env,
                            path_condition: state.path_condition,
                        },
                    ));
                    path_budget += 1;
                }
                Terminator::Branch {
                    condition,
                    then_target,
                    else_target,
                } => {
                    let condition =
                        expect_bool(eval_ssa_expr(&cfg.name, condition, &state.env)?, &cfg.name)?;
                    let then_pc = BoolExpr::And(
                        Box::new(state.path_condition.clone()),
                        Box::new(condition.clone()),
                    );
                    let else_pc = BoolExpr::And(
                        Box::new(state.path_condition),
                        Box::new(BoolExpr::Not(Box::new(condition))),
                    );
                    if smt::is_satisfiable_with_timeout(&cfg.name, &then_pc, program.limits.solver_timeout_ms)? {
                        if path_budget >= program.limits.max_paths {
                            return Err(SymExecError::PathLimitExceeded {
                                function: cfg.name.clone(),
                                max_paths: program.limits.max_paths,
                            });
                        }
                        worklist.push_back((
                            *then_target,
                            State {
                                env: state.env.clone(),
                                path_condition: then_pc,
                            },
                        ));
                        path_budget += 1;
                    }
                    if smt::is_satisfiable_with_timeout(&cfg.name, &else_pc, program.limits.solver_timeout_ms)? {
                        if path_budget >= program.limits.max_paths {
                            return Err(SymExecError::PathLimitExceeded {
                                function: cfg.name.clone(),
                                max_paths: program.limits.max_paths,
                            });
                        }
                        worklist.push_back((
                            *else_target,
                            State {
                                env: state.env,
                                path_condition: else_pc,
                            },
                        ));
                        path_budget += 1;
                    }
                }
                Terminator::LoopBoundExceeded => {
                    if smt::is_satisfiable_with_timeout(&cfg.name, &state.path_condition, program.limits.solver_timeout_ms)? {
                        return Err(SymExecError::LoopBoundExceeded {
                            function: cfg.name.clone(),
                        });
                    }
                }
                Terminator::Die => {
                    if smt::is_satisfiable_with_timeout(&cfg.name, &state.path_condition, program.limits.solver_timeout_ms)? {
                        return Err(SymExecError::DieReached {
                            function: cfg.name.clone(),
                        });
                    }
                }
                Terminator::Unreachable => {}
            }
        }
    }

    debug!(
        function = cfg.name,
        path_count = completed.len(),
        "symbolic execution completed"
    );
    Ok(completed)
}

pub fn verify_cfg(
    program: &Program,
    spec: &FunctionSpec,
    cfg: &ControlFlowGraph,
) -> std::result::Result<VerificationResult, SymExecError> {
    let completed = execute_cfg(program, spec, cfg)?;
    let variables = cfg
        .params
        .iter()
        .zip(spec.arg_types.iter())
        .map(|(param, ty)| smt::ModelVar {
            name: param.source.clone(),
            ty: *ty,
        })
        .collect::<Vec<_>>();

    let mut saw_valid_path = false;
    let mut saw_invalid_path = false;
    for state in completed {
        let validity_condition = BoolExpr::And(
            Box::new(state.path_condition.clone()),
            Box::new(well_defined_result_condition(&state.result)),
        );
        if !smt::is_satisfiable_with_timeout(&cfg.name, &validity_condition, program.limits.solver_timeout_ms)? {
            saw_invalid_path = true;
            continue;
        }
        saw_valid_path = true;

        let mut annotation_env = BTreeMap::new();
        for (param, ty) in cfg.params.iter().zip(spec.arg_types.iter()) {
            annotation_env.insert(param.source.clone(), symbolic_value(&param.source, *ty));
        }
        annotation_env.insert("result".to_string(), state.result.clone());
        let post = expect_bool(
            eval_expr(&cfg.name, &spec.post, &annotation_env)?,
            &cfg.name,
        )?;
        let failure_condition = BoolExpr::And(
            Box::new(state.path_condition.clone()),
            Box::new(BoolExpr::Not(Box::new(post))),
        );
        if let Some(assignments) = smt::find_model_with_timeout(&cfg.name, &failure_condition, &variables, program.limits.solver_timeout_ms)? {
            return Ok(VerificationResult::Counterexample(Counterexample {
                function: cfg.name.clone(),
                assignments,
            }));
        }
    }

    if !saw_valid_path && saw_invalid_path {
        return Err(SymExecError::NoValidPaths {
            function: cfg.name.clone(),
        });
    }

    Ok(VerificationResult::Verified {
        function: cfg.name.clone(),
    })
}

fn prepare_program(functions: &[ExtractedFunction], limits: Limits) -> crate::Result<Program> {
    let mut order = Vec::new();
    let mut specs = BTreeMap::new();
    let mut asts = BTreeMap::new();

    for function in functions {
        let spec = parse_function_spec(function)?;
        let ast = parse_function_ast_with_limits(function, limits.max_loop_unroll)?;
        order.push(function.name.clone());
        specs.insert(function.name.clone(), spec);
        asts.insert(function.name.clone(), ast);
    }

    let signatures = specs
        .iter()
        .map(|(name, spec)| (name.clone(), (spec.arg_types.clone(), spec.ret_type)))
        .collect::<BTreeMap<_, _>>();

    for name in &order {
        let spec = specs.get(name).expect("spec must exist");
        let ast = asts.get(name).expect("ast must exist");
        type_check_function_with_signatures(spec, ast, &signatures)?;
    }

    let call_graph = asts
        .iter()
        .map(|(name, ast)| (name.clone(), collect_called_functions(ast)))
        .collect::<BTreeMap<_, _>>();
    for (function, callees) in &call_graph {
        for callee in callees {
            if !specs.contains_key(callee) {
                return Err(crate::PerlcheckerError::from(SymExecError::UnknownCallee {
                    function: function.clone(),
                    callee: callee.clone(),
                }));
            }
        }
    }
    detect_recursion(&call_graph)?;

    let mut cfgs = BTreeMap::new();
    for name in &order {
        let ast = asts.get(name).expect("ast must exist");
        let ssa = ir::lower_to_ssa(ast)?;
        cfgs.insert(name.clone(), ir::build_cfg(&ssa));
    }

    Ok(Program { order, specs, cfgs, limits })
}

fn collect_called_functions(function: &crate::ast::FunctionAst) -> Vec<String> {
    let mut calls = Vec::new();
    collect_calls_from_stmts(&function.body, &mut calls);
    calls.sort();
    calls.dedup();
    calls
}

fn collect_calls_from_stmts(stmts: &[crate::ast::Stmt], calls: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            crate::ast::Stmt::Declare { .. } | crate::ast::Stmt::LoopBoundExceeded | crate::ast::Stmt::Last | crate::ast::Stmt::Next | crate::ast::Stmt::Die(_) => {}
            crate::ast::Stmt::Assign { expr, .. } | crate::ast::Stmt::Return(expr) => {
                collect_calls_from_expr(expr, calls);
            }
            crate::ast::Stmt::ArrayAssign { index, expr, .. } => {
                collect_calls_from_expr(index, calls);
                collect_calls_from_expr(expr, calls);
            }
            crate::ast::Stmt::HashAssign { key, expr, .. } => {
                collect_calls_from_expr(key, calls);
                collect_calls_from_expr(expr, calls);
            }
            crate::ast::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_calls_from_expr(condition, calls);
                collect_calls_from_stmts(then_branch, calls);
                collect_calls_from_stmts(else_branch, calls);
            }
        }
    }
}

fn collect_calls_from_expr(expr: &Expr, calls: &mut Vec<String>) {
    match expr {
        Expr::Unary { expr, .. } => collect_calls_from_expr(expr, calls),
        Expr::Binary { left, right, .. } => {
            collect_calls_from_expr(left, calls);
            collect_calls_from_expr(right, calls);
        }
        Expr::Ternary { condition, then_expr, else_expr } => {
            collect_calls_from_expr(condition, calls);
            collect_calls_from_expr(then_expr, calls);
            collect_calls_from_expr(else_expr, calls);
        }
        Expr::Access { index, .. } => collect_calls_from_expr(index, calls),
        Expr::Call { function, args } => {
            calls.push(function.clone());
            for arg in args {
                collect_calls_from_expr(arg, calls);
            }
        }
        Expr::Builtin { args, .. } => {
            for arg in args {
                collect_calls_from_expr(arg, calls);
            }
        }
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Variable(_) => {}
    }
}

fn detect_recursion(call_graph: &BTreeMap<String, Vec<String>>) -> crate::Result<()> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn visit(
        node: &str,
        graph: &BTreeMap<String, Vec<String>>,
        states: &mut BTreeMap<String, VisitState>,
    ) -> crate::Result<()> {
        if let Some(VisitState::Visiting) = states.get(node) {
            return Err(crate::PerlcheckerError::from(SymExecError::RecursionDetected {
                function: node.to_string(),
            }));
        }
        if states.get(node) == Some(&VisitState::Visited) {
            return Ok(());
        }
        states.insert(node.to_string(), VisitState::Visiting);
        if let Some(callees) = graph.get(node) {
            for callee in callees {
                visit(callee, graph, states)?;
            }
        }
        states.insert(node.to_string(), VisitState::Visited);
        Ok(())
    }

    let mut states = BTreeMap::new();
    for node in call_graph.keys() {
        visit(node, call_graph, &mut states)?;
    }
    Ok(())
}

fn execute_call(
    program: &Program,
    caller: &str,
    callee: &str,
    args: Vec<SymValue>,
    caller_path_condition: BoolExpr,
) -> std::result::Result<Vec<CompletedState>, SymExecError> {
    let spec = program
        .specs
        .get(callee)
        .ok_or_else(|| SymExecError::UnknownCallee {
            function: caller.to_string(),
            callee: callee.to_string(),
        })?;
    let cfg = program
        .cfgs
        .get(callee)
        .ok_or_else(|| SymExecError::UnknownCallee {
            function: caller.to_string(),
            callee: callee.to_string(),
        })?;

    let mut env = BTreeMap::new();
    let mut annotation_env = BTreeMap::new();
    for ((param, arg_type), arg_value) in cfg
        .params
        .iter()
        .zip(spec.arg_types.iter())
        .zip(args.into_iter())
    {
        env.insert(param.ssa_name.clone(), arg_value.clone());
        annotation_env.insert(param.source.clone(), arg_value.clone());

        // For array parameters, add companion length variables
        match arg_type {
            Type::ArrayInt | Type::ArrayStr => {
                let len_var = format!("{}__len", param.source);
                annotation_env.insert(len_var.clone(), SymValue::Int(IntExpr::Var(len_var)));
            }
            _ => {}
        }
    }
    let pre = expect_bool(eval_expr(callee, &spec.pre, &annotation_env)?, callee)?;
    let initial_path_condition = BoolExpr::And(Box::new(caller_path_condition), Box::new(pre));

    execute_cfg_from_state(
        program,
        spec,
        cfg,
        State {
            env,
            path_condition: initial_path_condition,
        },
    )
}

fn symbolic_value(name: &str, ty: Type) -> SymValue {
    match ty {
        Type::Int => SymValue::Int(IntExpr::Var(name.to_string())),
        Type::Str => SymValue::Str(StrExpr::Var(name.to_string())),
        Type::ArrayInt => SymValue::ArrayInt(ArrayIntExpr::Var(name.to_string())),
        Type::ArrayStr => SymValue::ArrayStr(ArrayStrExpr::Var(name.to_string())),
        Type::HashInt => SymValue::HashInt(HashIntExpr::Var(name.to_string())),
        Type::HashStr => SymValue::HashStr(HashStrExpr::Var(name.to_string())),
    }
}

fn eval_ssa_expr(
    function: &str,
    expr: &SsaExpr,
    env: &BTreeMap<String, SymValue>,
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match expr {
        SsaExpr::Int(value) => SymValue::Int(IntExpr::Const(*value)),
        SsaExpr::Bool(value) => SymValue::Bool(BoolExpr::Const(*value)),
        SsaExpr::String(value) => SymValue::Str(StrExpr::Const(value.clone())),
        SsaExpr::Var(name) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| SymExecError::MissingSymbol {
                    function: function.to_string(),
                    name: name.clone(),
                })?
        }
        SsaExpr::Unary { op, expr } => match op {
            crate::ast::UnaryOp::Neg => {
                let value = expect_int(eval_ssa_expr(function, expr, env)?, function)?;
                SymValue::Int(IntExpr::Sub(Box::new(IntExpr::Const(0)), Box::new(value)))
            }
            crate::ast::UnaryOp::Not => {
                let value = expect_bool(eval_ssa_expr(function, expr, env)?, function)?;
                SymValue::Bool(BoolExpr::Not(Box::new(value)))
            }
        },
        SsaExpr::Binary { left, op, right } => eval_binary(
            function,
            op,
            eval_ssa_expr(function, left, env)?,
            eval_ssa_expr(function, right, env)?,
        )?,
        SsaExpr::Ite { condition, then_expr, else_expr } => {
            let cond_val = eval_ssa_expr(function, condition, env)?;
            let then_val = eval_ssa_expr(function, then_expr, env)?;
            let else_val = eval_ssa_expr(function, else_expr, env)?;
            let cond_bool = expect_bool(cond_val, function)?;
            match (then_val, else_val) {
                (SymValue::Int(then_int), SymValue::Int(else_int)) => {
                    SymValue::Int(IntExpr::Ite(Box::new(cond_bool), Box::new(then_int), Box::new(else_int)))
                }
                (SymValue::Str(then_str), SymValue::Str(else_str)) => {
                    SymValue::Str(StrExpr::Ite(Box::new(cond_bool), Box::new(then_str), Box::new(else_str)))
                }
                (SymValue::Bool(then_bool), SymValue::Bool(else_bool)) => {
                    SymValue::Bool(BoolExpr::And(
                        Box::new(BoolExpr::Or(
                            Box::new(BoolExpr::Not(Box::new(cond_bool.clone()))),
                            Box::new(then_bool),
                        )),
                        Box::new(BoolExpr::Or(
                            Box::new(cond_bool),
                            Box::new(else_bool),
                        )),
                    ))
                }
                _ => {
                    return Err(SymExecError::TypeMismatch {
                        function: function.to_string(),
                    })
                }
            }
        }
        SsaExpr::Access {
            kind,
            collection,
            index,
        } => eval_access(
            function,
            *kind,
            eval_ssa_expr(function, collection, env)?,
            eval_ssa_expr(function, index, env)?,
        )?,
        SsaExpr::Store {
            kind,
            collection,
            index,
            value,
        } => eval_store(
            function,
            *kind,
            eval_ssa_expr(function, collection, env)?,
            eval_ssa_expr(function, index, env)?,
            eval_ssa_expr(function, value, env)?,
        )?,
        SsaExpr::Builtin {
            function: builtin,
            args,
        } => {
            let args = args
                .iter()
                .map(|arg| eval_ssa_expr(function, arg, env))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            eval_builtin(function, *builtin, &args)?
        }
    })
}

fn eval_expr(
    function: &str,
    expr: &Expr,
    env: &BTreeMap<String, SymValue>,
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match expr {
        Expr::Int(value) => SymValue::Int(IntExpr::Const(*value)),
        Expr::Bool(value) => SymValue::Bool(BoolExpr::Const(*value)),
        Expr::String(value) => SymValue::Str(StrExpr::Const(value.clone())),
        Expr::Variable(name) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| SymExecError::MissingSymbol {
                    function: function.to_string(),
                    name: name.clone(),
                })?
        }
        Expr::Unary { op, expr } => match op {
            crate::ast::UnaryOp::Neg => {
                let value = expect_int(eval_expr(function, expr, env)?, function)?;
                SymValue::Int(IntExpr::Sub(Box::new(IntExpr::Const(0)), Box::new(value)))
            }
            crate::ast::UnaryOp::Not => {
                let value = expect_bool(eval_expr(function, expr, env)?, function)?;
                SymValue::Bool(BoolExpr::Not(Box::new(value)))
            }
        },
        Expr::Binary { left, op, right } => eval_binary(
            function,
            op,
            eval_expr(function, left, env)?,
            eval_expr(function, right, env)?,
        )?,
        Expr::Ternary { condition, then_expr, else_expr } => {
            let cond_val = eval_expr(function, condition, env)?;
            let then_val = eval_expr(function, then_expr, env)?;
            let else_val = eval_expr(function, else_expr, env)?;
            let cond_bool = expect_bool(cond_val, function)?;
            match (then_val, else_val) {
                (SymValue::Int(then_int), SymValue::Int(else_int)) => {
                    SymValue::Int(IntExpr::Ite(Box::new(cond_bool), Box::new(then_int), Box::new(else_int)))
                }
                (SymValue::Str(then_str), SymValue::Str(else_str)) => {
                    SymValue::Str(StrExpr::Ite(Box::new(cond_bool), Box::new(then_str), Box::new(else_str)))
                }
                (SymValue::Bool(then_bool), SymValue::Bool(else_bool)) => {
                    SymValue::Bool(BoolExpr::And(
                        Box::new(BoolExpr::Or(
                            Box::new(BoolExpr::Not(Box::new(cond_bool.clone()))),
                            Box::new(then_bool),
                        )),
                        Box::new(BoolExpr::Or(
                            Box::new(cond_bool),
                            Box::new(else_bool),
                        )),
                    ))
                }
                _ => {
                    return Err(SymExecError::TypeMismatch {
                        function: function.to_string(),
                    })
                }
            }
        }
        Expr::Access {
            kind,
            collection,
            index,
        } => eval_access(
            function,
            *kind,
            env.get(collection)
                .cloned()
                .ok_or_else(|| SymExecError::MissingSymbol {
                    function: function.to_string(),
                    name: collection.clone(),
                })?,
            eval_expr(function, index, env)?,
        )?,
        Expr::Call { .. } => {
            return Err(SymExecError::TypeMismatch {
                function: function.to_string(),
            });
        }
        Expr::Builtin {
            function: builtin,
            args,
        } => {
            let args = args
                .iter()
                .map(|arg| eval_expr(function, arg, env))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            eval_builtin(function, *builtin, &args)?
        }
    })
}

fn eval_binary(
    function: &str,
    op: &crate::ast::BinaryOp,
    left: SymValue,
    right: SymValue,
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match op {
        crate::ast::BinaryOp::Add => SymValue::Int(IntExpr::Add(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Sub => SymValue::Int(IntExpr::Sub(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Mul => SymValue::Int(IntExpr::Mul(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Div => SymValue::Int(IntExpr::Div(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Mod => SymValue::Int(IntExpr::Mod(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Pow => SymValue::Int(IntExpr::Pow(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::BitAnd => SymValue::Int(IntExpr::BitAnd(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::BitOr => SymValue::Int(IntExpr::BitOr(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::BitXor => SymValue::Int(IntExpr::BitXor(
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Concat => SymValue::Str(StrExpr::Concat(
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::Lt => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Lt,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Le => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Le,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Gt => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Gt,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Ge => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Ge,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Eq => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Eq,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::Ne => SymValue::Bool(BoolExpr::IntCmp(
            CmpOp::Ne,
            Box::new(expect_int(left, function)?),
            Box::new(expect_int(right, function)?),
        )),
        crate::ast::BinaryOp::StrEq => SymValue::Bool(BoolExpr::StrEq(
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::StrNe => SymValue::Bool(BoolExpr::Not(Box::new(BoolExpr::StrEq(
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )))),
        crate::ast::BinaryOp::StrLt => SymValue::Bool(BoolExpr::StrCmp(
            CmpOp::Lt,
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::StrLe => SymValue::Bool(BoolExpr::StrCmp(
            CmpOp::Le,
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::StrGt => SymValue::Bool(BoolExpr::StrCmp(
            CmpOp::Gt,
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::StrGe => SymValue::Bool(BoolExpr::StrCmp(
            CmpOp::Ge,
            Box::new(expect_str(left, function)?),
            Box::new(expect_str(right, function)?),
        )),
        crate::ast::BinaryOp::And => SymValue::Bool(BoolExpr::And(
            Box::new(expect_bool(left, function)?),
            Box::new(expect_bool(right, function)?),
        )),
        crate::ast::BinaryOp::Or => SymValue::Bool(BoolExpr::Or(
            Box::new(expect_bool(left, function)?),
            Box::new(expect_bool(right, function)?),
        )),
    })
}

fn extract_array_int_base_name(array: &SymValue) -> String {
    match array {
        SymValue::ArrayInt(ArrayIntExpr::Var(name)) => name.clone(),
        SymValue::ArrayInt(ArrayIntExpr::Store(inner, _, _)) => extract_array_int_base_name(
            &SymValue::ArrayInt((**inner).clone())
        ),
        _ => unreachable!("extract_array_int_base_name should only be called with ArrayInt"),
    }
}

fn extract_array_str_base_name(array: &SymValue) -> String {
    match array {
        SymValue::ArrayStr(ArrayStrExpr::Var(name)) => name.clone(),
        SymValue::ArrayStr(ArrayStrExpr::Store(inner, _, _)) => extract_array_str_base_name(
            &SymValue::ArrayStr((**inner).clone())
        ),
        _ => unreachable!("extract_array_str_base_name should only be called with ArrayStr"),
    }
}

fn eval_builtin(
    function: &str,
    builtin: Builtin,
    args: &[SymValue],
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match builtin {
        Builtin::Length => {
            let [value] = args else {
                unreachable!("length arity is enforced by the parser");
            };
            SymValue::Int(IntExpr::Length(Box::new(expect_str(
                value.clone(),
                function,
            )?)))
        }
        Builtin::Substr => {
            let [value, start, len] = args else {
                unreachable!("substr arity is enforced by the parser");
            };
            SymValue::Str(StrExpr::Substr(
                Box::new(expect_str(value.clone(), function)?),
                Box::new(expect_int(start.clone(), function)?),
                Box::new(expect_int(len.clone(), function)?),
            ))
        }
        Builtin::Index => {
            let [haystack, needle] = args else {
                unreachable!("index arity is enforced by the parser");
            };
            SymValue::Int(IntExpr::Index(
                Box::new(expect_str(haystack.clone(), function)?),
                Box::new(expect_str(needle.clone(), function)?),
            ))
        }
        Builtin::Scalar => {
            let [array] = args else {
                unreachable!("scalar arity is enforced by the parser");
            };
            match array {
                SymValue::ArrayInt(ArrayIntExpr::Var(name)) => {
                    SymValue::Int(IntExpr::Var(format!("{name}__len")))
                }
                SymValue::ArrayInt(ArrayIntExpr::Store(_, _, _)) => {
                    // For Store variants, we need to extract the base variable name
                    let base_name = extract_array_int_base_name(array);
                    SymValue::Int(IntExpr::Var(format!("{base_name}__len")))
                }
                SymValue::ArrayStr(ArrayStrExpr::Var(name)) => {
                    SymValue::Int(IntExpr::Var(format!("{name}__len")))
                }
                SymValue::ArrayStr(ArrayStrExpr::Store(_, _, _)) => {
                    // For Store variants, we need to extract the base variable name
                    let base_name = extract_array_str_base_name(array);
                    SymValue::Int(IntExpr::Var(format!("{base_name}__len")))
                }
                _ => {
                    return Err(SymExecError::TypeMismatch {
                        function: function.to_string(),
                    });
                }
            }
        }
        Builtin::Abs => {
            let [value] = args else {
                unreachable!("abs arity is enforced by the parser");
            };
            SymValue::Int(IntExpr::Abs(Box::new(expect_int(
                value.clone(),
                function,
            )?)))
        }
        Builtin::Min => {
            let [left, right] = args else {
                unreachable!("min arity is enforced by the parser");
            };
            let left_int = expect_int(left.clone(), function)?;
            let right_int = expect_int(right.clone(), function)?;
            // min(a, b) = ite(a <= b, a, b)
            SymValue::Int(IntExpr::Ite(
                Box::new(BoolExpr::IntCmp(
                    CmpOp::Le,
                    Box::new(left_int.clone()),
                    Box::new(right_int.clone()),
                )),
                Box::new(left_int),
                Box::new(right_int),
            ))
        }
        Builtin::Max => {
            let [left, right] = args else {
                unreachable!("max arity is enforced by the parser");
            };
            let left_int = expect_int(left.clone(), function)?;
            let right_int = expect_int(right.clone(), function)?;
            // max(a, b) = ite(a >= b, a, b)
            SymValue::Int(IntExpr::Ite(
                Box::new(BoolExpr::IntCmp(
                    CmpOp::Ge,
                    Box::new(left_int.clone()),
                    Box::new(right_int.clone()),
                )),
                Box::new(left_int),
                Box::new(right_int),
            ))
        }
        Builtin::Ord => {
            let [value] = args else {
                unreachable!("ord arity is enforced by the parser");
            };
            SymValue::Int(IntExpr::Ord(Box::new(expect_str(
                value.clone(),
                function,
            )?)))
        }
        Builtin::Chr => {
            let [value] = args else {
                unreachable!("chr arity is enforced by the parser");
            };
            SymValue::Str(StrExpr::Chr(Box::new(expect_int(
                value.clone(),
                function,
            )?)))
        }
    })
}

fn eval_access(
    function: &str,
    kind: AccessKind,
    collection: SymValue,
    index: SymValue,
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match kind {
        AccessKind::Array => match collection {
            SymValue::ArrayInt(array) => SymValue::Int(IntExpr::ArraySelect(
                Box::new(array),
                Box::new(expect_int(index, function)?),
            )),
            SymValue::ArrayStr(array) => SymValue::Str(StrExpr::ArraySelect(
                Box::new(array),
                Box::new(expect_int(index, function)?),
            )),
            _ => {
                return Err(SymExecError::TypeMismatch {
                    function: function.to_string(),
                });
            }
        },
        AccessKind::Hash => match collection {
            SymValue::HashInt(hash) => SymValue::Int(IntExpr::HashSelect(
                Box::new(hash),
                Box::new(expect_str(index, function)?),
            )),
            SymValue::HashStr(hash) => SymValue::Str(StrExpr::HashSelect(
                Box::new(hash),
                Box::new(expect_str(index, function)?),
            )),
            _ => {
                return Err(SymExecError::TypeMismatch {
                    function: function.to_string(),
                });
            }
        },
    })
}

fn eval_store(
    function: &str,
    kind: AccessKind,
    collection: SymValue,
    index: SymValue,
    value: SymValue,
) -> std::result::Result<SymValue, SymExecError> {
    Ok(match kind {
        AccessKind::Array => match (collection, value) {
            (SymValue::ArrayInt(array), SymValue::Int(value)) => SymValue::ArrayInt(
                ArrayIntExpr::Store(
                    Box::new(array),
                    Box::new(expect_int(index, function)?),
                    Box::new(value),
                ),
            ),
            (SymValue::ArrayStr(array), SymValue::Str(value)) => SymValue::ArrayStr(
                ArrayStrExpr::Store(
                    Box::new(array),
                    Box::new(expect_int(index, function)?),
                    Box::new(value),
                ),
            ),
            _ => {
                return Err(SymExecError::TypeMismatch {
                    function: function.to_string(),
                });
            }
        },
        AccessKind::Hash => match (collection, value) {
            (SymValue::HashInt(hash), SymValue::Int(value)) => SymValue::HashInt(
                HashIntExpr::Store(
                    Box::new(hash),
                    Box::new(expect_str(index, function)?),
                    Box::new(value),
                ),
            ),
            (SymValue::HashStr(hash), SymValue::Str(value)) => SymValue::HashStr(
                HashStrExpr::Store(
                    Box::new(hash),
                    Box::new(expect_str(index, function)?),
                    Box::new(value),
                ),
            ),
            _ => {
                return Err(SymExecError::TypeMismatch {
                    function: function.to_string(),
                });
            }
        },
    })
}

fn expect_int(value: SymValue, function: &str) -> std::result::Result<IntExpr, SymExecError> {
    match value {
        SymValue::Int(value) => Ok(value),
        SymValue::Bool(_)
        | SymValue::Str(_)
        | SymValue::ArrayInt(_)
        | SymValue::ArrayStr(_)
        | SymValue::HashInt(_)
        | SymValue::HashStr(_) => Err(SymExecError::TypeMismatch {
            function: function.to_string(),
        }),
    }
}

fn expect_bool(value: SymValue, function: &str) -> std::result::Result<BoolExpr, SymExecError> {
    match value {
        SymValue::Bool(value) => Ok(value),
        SymValue::Int(_)
        | SymValue::Str(_)
        | SymValue::ArrayInt(_)
        | SymValue::ArrayStr(_)
        | SymValue::HashInt(_)
        | SymValue::HashStr(_) => Err(SymExecError::TypeMismatch {
            function: function.to_string(),
        }),
    }
}

fn expect_str(value: SymValue, function: &str) -> std::result::Result<StrExpr, SymExecError> {
    match value {
        SymValue::Str(value) => Ok(value),
        SymValue::Bool(_)
        | SymValue::Int(_)
        | SymValue::ArrayInt(_)
        | SymValue::ArrayStr(_)
        | SymValue::HashInt(_)
        | SymValue::HashStr(_) => Err(SymExecError::TypeMismatch {
            function: function.to_string(),
        }),
    }
}

fn well_defined_result_condition(result: &SymValue) -> BoolExpr {
    match result {
        SymValue::Int(expr) => {
            BoolExpr::IntCmp(CmpOp::Eq, Box::new(expr.clone()), Box::new(expr.clone()))
        }
        SymValue::Str(expr) => BoolExpr::StrEq(Box::new(expr.clone()), Box::new(expr.clone())),
        SymValue::Bool(expr) => expr.clone(),
        SymValue::ArrayInt(_)
        | SymValue::ArrayStr(_)
        | SymValue::HashInt(_)
        | SymValue::HashStr(_) => BoolExpr::Const(true),
    }
}

#[cfg(test)]
mod tests {
    use crate::{extractor::ExtractedFunction, parser::parse_function_ast};

    use super::{
        VerificationResult, execute_cfg, prepare_program, verify_cfg,
        verify_extracted_function,
    };

    #[test]
    fn counterexample_reports_int_inputs() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result > $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Counterexample(_)));
    }

    #[test]
    fn verifies_string_concat_property() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str, Str) -> Str".to_string(),
                "# post: length($result) == length($x) + length($y)".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    return $x . $y;\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn verifies_bounded_substring_property() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Str".to_string(),
                "# post: $result eq substr($x, 0, length($x))".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return substr($x, 0, length($x));\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn discards_division_by_zero_paths() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int, Int) -> Int".to_string(),
                "# post: $result == 1".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    if ($y == 0) {\n        return $x / $y;\n    }\n    return 1;\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn verifies_array_store_then_read() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Array<Int>, Int, Int) -> Int".to_string(),
                "# post: $result == $v".to_string(),
            ],
            body: "\n    my ($arr, $i, $v) = @_;\n    $arr[$i] = $v;\n    return $arr[$i];\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn verifies_hash_store_then_read() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Hash<Str, Str>, Str, Str) -> Str".to_string(),
                "# post: $result eq $v".to_string(),
            ],
            body: "\n    my ($h, $k, $v) = @_;\n    $h{$k} = $v;\n    return $h{$k};\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn verifies_intra_file_function_calls() {
        let functions = vec![
            ExtractedFunction {
                name: "inc".to_string(),
                annotations: vec![
                    "# sig: (Int) -> Int".to_string(),
                    "# post: $result == $x + 1".to_string(),
                ],
                body: "\n    my ($x) = @_;\n    return $x + 1;\n".to_string(),
                start_line: 1,
            },
            ExtractedFunction {
                name: "use_inc".to_string(),
                annotations: vec![
                    "# sig: (Int) -> Int".to_string(),
                    "# post: $result == $x + 1".to_string(),
                ],
                body: "\n    my ($x) = @_;\n    return inc($x);\n".to_string(),
                start_line: 6,
            },
        ];

        let results = super::verify_extracted_functions(&functions, Default::default()).unwrap();
        assert_eq!(results.len(), 2);
        assert!(matches!(results[0], VerificationResult::Verified { .. }));
        assert!(matches!(results[1], VerificationResult::Verified { .. }));
    }

    #[test]
    fn rejects_recursive_call_graphs() {
        let functions = vec![ExtractedFunction {
            name: "loop".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result >= $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return loop($x);\n".to_string(),
            start_line: 1,
        }];

        let error = super::verify_extracted_functions(&functions, Default::default()).unwrap_err();
        assert!(error.to_string().contains("recursive call graph"));
    }

    #[test]
    fn verifies_bounded_while_loop() {
        let function = ExtractedFunction {
            name: "countdown".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# pre: $x >= 0 && $x <= 5".to_string(),
                "# post: $result == 0".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    while ($x > 0) {\n        $x = $x - 1;\n    }\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn rejects_feasible_loop_bound_exhaustion() {
        let function = ExtractedFunction {
            name: "spin".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result == 0".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    while ($x >= 0) {\n        $x = $x + 1;\n    }\n    return 0;\n".to_string(),
            start_line: 1,
        };

        let error = verify_extracted_function(&function).unwrap_err();
        assert!(error.to_string().contains("loop unroll bound"));
    }

    #[test]
    fn rejects_path_explosion_beyond_limit() {
        let function = ExtractedFunction {
            name: "too_many_paths".to_string(),
            annotations: vec![
                "# sig: (Int, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) -> Int"
                    .to_string(),
                "# post: $result >= 0".to_string(),
            ],
            body: "\n    my ($a, $b, $c, $d, $e, $f, $g, $h, $i, $j, $k) = @_;\n    my $x = 0;\n    if ($a > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($b > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($c > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($d > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($e > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($f > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($g > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($h > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($i > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($j > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    if ($k > 0) { $x = $x + 1; } else { $x = $x + 1; }\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let error = verify_extracted_function(&function).unwrap_err();
        assert!(error.to_string().contains("maximum number of symbolic paths"));
    }

    #[test]
    fn execution_splits_branches_deterministically() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result >= 0".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    if ($x > 0) {\n        return $x;\n    }\n    return 0 - $x;\n".to_string(),
            start_line: 1,
        };

        let spec = crate::annotations::parse_function_spec(&function).unwrap();
        let ast = parse_function_ast(&function).unwrap();
        crate::ast::type_check_function(&spec, &ast).unwrap();
        let ssa = crate::ir::lower_to_ssa(&ast).unwrap();
        let cfg = crate::ir::build_cfg(&ssa);
        let program = prepare_program(std::slice::from_ref(&function), Default::default()).unwrap();
        let states = execute_cfg(&program, &spec, &cfg).unwrap();

        assert_eq!(states.len(), 2);
        assert!(matches!(
            verify_cfg(&program, &spec, &cfg).unwrap(),
            VerificationResult::Verified { .. }
        ));
    }

    #[test]
    fn verifies_die_unreachable_guarded_by_precondition() {
        let function = ExtractedFunction {
            name: "guarded_positive".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# pre: $x >= 1 && $x <= 100".to_string(),
                "# post: $result >= 1".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    if ($x <= 0) {\n        die \"must be positive\";\n    }\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let result = verify_extracted_function(&function).unwrap();
        assert!(matches!(result, VerificationResult::Verified { .. }));
    }

    #[test]
    fn rejects_reachable_die_statement() {
        let function = ExtractedFunction {
            name: "die_is_reachable".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# pre: $x >= 0 && $x <= 10".to_string(),
                "# post: $result >= 0".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    if ($x == 0) {\n        die \"zero!\";\n    }\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let error = verify_extracted_function(&function).unwrap_err();
        assert!(error.to_string().contains("die"));
    }
}
