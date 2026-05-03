use pest::{Parser, iterators::Pair, pratt_parser::PrattParser};
use pest_derive::Parser;
use thiserror::Error;
use tracing::debug;

use crate::{
    ast::{AccessKind, BinaryOp, Builtin, Expr, FunctionAst, Stmt, UnaryOp},
    extractor::ExtractedFunction,
};

#[derive(Parser)]
#[grammar = "parser/perl_subset.pest"]
struct PerlSubsetParser;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("function `{function}` has invalid syntax at {line}:{column}: {message}")]
    InvalidSyntax {
        function: String,
        line: usize,
        column: usize,
        message: String,
    },
}

pub fn parse_expr(input: &str) -> std::result::Result<Expr, String> {
    let mut pairs =
        PerlSubsetParser::parse(Rule::annotation_expr, input).map_err(render_expr_error)?;
    let expr = pairs
        .next()
        .expect("annotation_expr parser must yield a root pair")
        .into_inner()
        .next()
        .expect("annotation_expr must contain an expr");
    build_expr(expr)
}

pub fn parse_function_ast(
    function: &ExtractedFunction,
) -> std::result::Result<FunctionAst, ParseError> {
    parse_function_ast_with_limits(function, crate::limits::DEFAULT_MAX_LOOP_UNROLL)
}

pub fn parse_function_ast_with_limits(
    function: &ExtractedFunction,
    max_loop_unroll: usize,
) -> std::result::Result<FunctionAst, ParseError> {
    let mut pairs =
        PerlSubsetParser::parse(Rule::function_body, &function.body).map_err(|error| {
            let ((line, column), message) = render_body_error(error);
            ParseError::InvalidSyntax {
                function: function.name.clone(),
                line,
                column,
                message,
            }
        })?;

    let body = pairs
        .next()
        .expect("function_body parser must yield a root pair");
    let mut inner = body.into_inner();
    let params = parse_parameter_binding(
        inner
            .next()
            .expect("function_body must begin with a parameter binding"),
    );
    let stmts = inner
        .filter(|pair| {
            matches!(
                pair.as_rule(),
                Rule::do_while_stmt
                    | Rule::do_until_stmt
                    | Rule::while_stmt
                    | Rule::until_stmt
                    | Rule::for_stmt
                    | Rule::foreach_stmt
                    | Rule::assign_stmt
                    | Rule::array_assign_stmt
                    | Rule::hash_assign_stmt
                    | Rule::list_assign_stmt
                    | Rule::declare_stmt
                    | Rule::if_stmt
                    | Rule::unless_stmt
                    | Rule::return_stmt
                    | Rule::die_stmt
                    | Rule::warn_stmt
                    | Rule::print_stmt
                    | Rule::say_stmt
                    | Rule::last_stmt
                    | Rule::next_stmt
                    | Rule::push_stmt
                    | Rule::inc_stmt
                    | Rule::dec_stmt
            )
        })
        .flat_map(|pair| parse_stmt(pair, max_loop_unroll))
        .collect();

    let ast = FunctionAst {
        name: function.name.clone(),
        params,
        body: stmts,
    };
    debug!(
        function = ast.name,
        stmt_count = ast.body.len(),
        "parsed function AST"
    );
    Ok(ast)
}

fn parse_parameter_binding(pair: Pair<'_, Rule>) -> Vec<String> {
    pair.into_inner()
        .flat_map(|pair| match pair.as_rule() {
            Rule::var_list => pair
                .into_inner()
                .filter(|inner| inner.as_rule() == Rule::var)
                .map(parse_variable)
                .collect::<Vec<_>>(),
            Rule::var => vec![parse_variable(pair)],
            _ => Vec::new(),
        })
        .collect()
}

fn parse_stmt(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    match pair.as_rule() {
        Rule::assign_stmt => vec![parse_assign(pair)],
        Rule::array_assign_stmt => vec![parse_array_assign(pair)],
        Rule::hash_assign_stmt => vec![parse_hash_assign(pair)],
        Rule::list_assign_stmt => parse_list_assign(pair),
        Rule::declare_stmt => vec![parse_declare(pair)],
        Rule::if_stmt => vec![parse_if(pair, max_loop_unroll)],
        Rule::unless_stmt => vec![parse_unless(pair, max_loop_unroll)],
        Rule::do_while_stmt => parse_do_while(pair, max_loop_unroll),
        Rule::do_until_stmt => parse_do_until(pair, max_loop_unroll),
        Rule::while_stmt => parse_while(pair, max_loop_unroll),
        Rule::until_stmt => parse_until(pair, max_loop_unroll),
        Rule::for_stmt => parse_for(pair, max_loop_unroll),
        Rule::foreach_stmt => parse_foreach(pair, max_loop_unroll),
        Rule::return_stmt => vec![parse_return(pair)],
        Rule::die_stmt => vec![parse_die(pair)],
        Rule::warn_stmt => vec![], // warn is a no-op for verification
        Rule::print_stmt => vec![], // print is a no-op for verification
        Rule::say_stmt => vec![], // say is a no-op for verification
        Rule::last_stmt => vec![parse_last(pair)],
        Rule::next_stmt => vec![parse_next(pair)],
        Rule::push_stmt => vec![parse_push(pair)],
        Rule::inc_stmt => vec![parse_inc(pair)],
        Rule::dec_stmt => vec![parse_dec(pair)],
        other => unreachable!("unexpected statement rule: {other:?}"),
    }
}

fn parse_assign(pair: Pair<'_, Rule>) -> Stmt {
    let mut declaration = false;
    let mut name = None;
    let mut expr = None;
    let mut compound_op = None;
    let mut modifier: Option<Pair<'_, Rule>> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::declaration => declaration = true,
            Rule::var => name = Some(parse_variable(inner)),
            Rule::assign_op => {
                compound_op = match inner.as_str() {
                    "+=" => Some(BinaryOp::Add),
                    "-=" => Some(BinaryOp::Sub),
                    "**=" => Some(BinaryOp::Pow),
                    "*=" => Some(BinaryOp::Mul),
                    "/=" => Some(BinaryOp::Div),
                    "%=" => Some(BinaryOp::Mod),
                    ".=" => Some(BinaryOp::Concat),
                    "&=" => Some(BinaryOp::BitAnd),
                    "|=" => Some(BinaryOp::BitOr),
                    "^=" => Some(BinaryOp::BitXor),
                    "<<=" => Some(BinaryOp::Shl),
                    ">>=" => Some(BinaryOp::Shr),
                    _ => None,
                };
            }
            Rule::expr => expr = Some(build_expr(inner).expect("validated expression")),
            Rule::assign_if | Rule::assign_unless => modifier = Some(inner),
            _ => {}
        }
    }

    let name = name.expect("assignment must have a variable");
    let rhs = expr.expect("assignment must have an expression");
    let final_expr = match compound_op {
        Some(op) => Expr::Binary {
            left: Box::new(Expr::Variable(name.clone())),
            op,
            right: Box::new(rhs),
        },
        None => rhs,
    };

    let assign_stmt = Stmt::Assign {
        name,
        expr: final_expr,
        declaration,
    };

    // Handle statement modifier (ASSIGN if/unless COND)
    if let Some(mod_pair) = modifier {
        match mod_pair.as_rule() {
            Rule::assign_if => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("assign_if must have a condition"),
                )
                .expect("validated assign_if condition");
                return Stmt::If {
                    condition,
                    then_branch: vec![assign_stmt],
                    else_branch: Vec::new(),
                };
            }
            Rule::assign_unless => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("assign_unless must have a condition"),
                )
                .expect("validated assign_unless condition");
                let negated = Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(condition),
                };
                return Stmt::If {
                    condition: negated,
                    then_branch: vec![assign_stmt],
                    else_branch: Vec::new(),
                };
            }
            _ => {}
        }
    }

    assign_stmt
}

fn parse_array_assign(pair: Pair<'_, Rule>) -> Stmt {
    let mut inner = pair.into_inner();
    let name = parse_variable(inner.next().expect("array assignment must have a variable"));
    let index = parse_access_operand(inner.next().expect("array assignment must have an index"))
        .expect("validated array index");
    let expr = build_expr(inner.next().expect("array assignment must have an expression"))
        .expect("validated array assignment expression");
    Stmt::ArrayAssign { name, index, expr }
}

fn parse_hash_assign(pair: Pair<'_, Rule>) -> Stmt {
    let mut inner = pair.into_inner();
    let name = parse_variable(inner.next().expect("hash assignment must have a variable"));
    let key = parse_access_operand(inner.next().expect("hash assignment must have a key"))
        .expect("validated hash key");
    let expr = build_expr(inner.next().expect("hash assignment must have an expression"))
        .expect("validated hash assignment expression");
    Stmt::HashAssign { name, key, expr }
}

fn parse_list_assign(pair: Pair<'_, Rule>) -> Vec<Stmt> {
    let mut declaration = false;
    let mut vars = Vec::new();
    let mut exprs = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::declaration => declaration = true,
            Rule::var_list => {
                vars = inner
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::var)
                    .map(parse_variable)
                    .collect();
            }
            Rule::expr_list => {
                exprs = inner
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::expr)
                    .map(|p| build_expr(p).expect("validated list expression"))
                    .collect();
            }
            _ => {}
        }
    }

    // To get swap semantics right, evaluate all RHS into temporaries first,
    // then assign from temporaries to the target variables.
    let mut stmts = Vec::new();
    let temp_names: Vec<String> = (0..exprs.len())
        .map(|i| format!("__list_tmp_{}", i))
        .collect();

    // Step 1: Assign each RHS expression to a temp variable
    for (i, expr) in exprs.into_iter().enumerate() {
        stmts.push(Stmt::Assign {
            name: temp_names[i].clone(),
            expr,
            declaration: true,
        });
    }

    // Step 2: Assign from temps to the target variables
    for (i, var) in vars.into_iter().enumerate() {
        if i < temp_names.len() {
            stmts.push(Stmt::Assign {
                name: var,
                expr: Expr::Variable(temp_names[i].clone()),
                declaration,
            });
        }
    }

    stmts
}

fn parse_declare(pair: Pair<'_, Rule>) -> Stmt {
    let name = pair
        .into_inner()
        .find(|inner| inner.as_rule() == Rule::var)
        .map(parse_variable)
        .expect("declaration must contain a variable");

    Stmt::Declare { name }
}

fn parse_if(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Stmt {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("if must have a condition"))
        .expect("validated condition expression");
    let then_branch = parse_block(inner.next().expect("if must have a then block"), max_loop_unroll);
    let mut elsif_clauses = Vec::new();
    let mut final_else = Vec::new();

    for clause in inner {
        match clause.as_rule() {
            Rule::elsif_clause => elsif_clauses.push(parse_elsif_clause(clause, max_loop_unroll)),
            Rule::else_clause => final_else = parse_else_clause(clause, max_loop_unroll),
            other => unreachable!("unexpected if clause rule: {other:?}"),
        }
    }

    let else_branch = elsif_clauses.into_iter().rev().fold(
        final_else,
        |else_branch, (condition, then_branch)| {
            vec![Stmt::If {
                condition,
                then_branch,
                else_branch,
            }]
        },
    );

    Stmt::If {
        condition,
        then_branch,
        else_branch,
    }
}

fn parse_unless(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Stmt {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("unless must have a condition"))
        .expect("validated unless condition expression");
    let then_block = parse_block(inner.next().expect("unless must have a block"), max_loop_unroll);
    let else_branch = inner
        .find(|p| p.as_rule() == Rule::else_clause)
        .map(|clause| parse_else_clause(clause, max_loop_unroll))
        .unwrap_or_default();

    // Desugar: unless (COND) { A } else { B } => if (!(COND)) { A } else { B }
    let negated_condition = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(condition),
    };

    Stmt::If {
        condition: negated_condition,
        then_branch: then_block,
        else_branch,
    }
}

fn parse_return(pair: Pair<'_, Rule>) -> Stmt {
    let mut inner = pair.into_inner();
    let expr = build_expr(inner.next().expect("return must contain an expression"))
        .expect("validated return expression");

    // Check for statement modifier (return EXPR if/unless COND)
    if let Some(modifier) = inner.next() {
        match modifier.as_rule() {
            Rule::return_if => {
                let condition = build_expr(
                    modifier.into_inner().next().expect("return_if must have a condition"),
                )
                .expect("validated return_if condition");
                return Stmt::If {
                    condition,
                    then_branch: vec![Stmt::Return(expr)],
                    else_branch: Vec::new(),
                };
            }
            Rule::return_unless => {
                let condition = build_expr(
                    modifier.into_inner().next().expect("return_unless must have a condition"),
                )
                .expect("validated return_unless condition");
                let negated = Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(condition),
                };
                return Stmt::If {
                    condition: negated,
                    then_branch: vec![Stmt::Return(expr)],
                    else_branch: Vec::new(),
                };
            }
            _ => {}
        }
    }

    Stmt::Return(expr)
}

fn parse_die(pair: Pair<'_, Rule>) -> Stmt {
    let mut expr = None;
    let mut modifier: Option<Pair<'_, Rule>> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expr if expr.is_none() => {
                expr = Some(build_expr(inner).expect("validated die expression"));
            }
            Rule::die_if | Rule::die_unless => {
                modifier = Some(inner);
            }
            _ => {}
        }
    }

    let die_expr = expr.unwrap_or_else(|| Expr::String(String::new()));
    let die_stmt = Stmt::Die(die_expr);

    // Handle statement modifier (die EXPR if/unless COND)
    if let Some(mod_pair) = modifier {
        match mod_pair.as_rule() {
            Rule::die_if => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("die_if must have a condition"),
                )
                .expect("validated die_if condition");
                return Stmt::If {
                    condition,
                    then_branch: vec![die_stmt],
                    else_branch: Vec::new(),
                };
            }
            Rule::die_unless => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("die_unless must have a condition"),
                )
                .expect("validated die_unless condition");
                let negated = Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(condition),
                };
                return Stmt::If {
                    condition: negated,
                    then_branch: vec![die_stmt],
                    else_branch: Vec::new(),
                };
            }
            _ => {}
        }
    }

    die_stmt
}

fn parse_last(pair: Pair<'_, Rule>) -> Stmt {
    let mut modifier: Option<Pair<'_, Rule>> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::last_if | Rule::last_unless => {
                modifier = Some(inner);
            }
            _ => {}
        }
    }

    if let Some(mod_pair) = modifier {
        match mod_pair.as_rule() {
            Rule::last_if => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("last_if must have a condition"),
                )
                .expect("validated last_if condition");
                return Stmt::If {
                    condition,
                    then_branch: vec![Stmt::Last],
                    else_branch: Vec::new(),
                };
            }
            Rule::last_unless => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("last_unless must have a condition"),
                )
                .expect("validated last_unless condition");
                let negated = Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(condition),
                };
                return Stmt::If {
                    condition: negated,
                    then_branch: vec![Stmt::Last],
                    else_branch: Vec::new(),
                };
            }
            _ => {}
        }
    }

    Stmt::Last
}

fn parse_next(pair: Pair<'_, Rule>) -> Stmt {
    let mut modifier: Option<Pair<'_, Rule>> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::next_if | Rule::next_unless => {
                modifier = Some(inner);
            }
            _ => {}
        }
    }

    if let Some(mod_pair) = modifier {
        match mod_pair.as_rule() {
            Rule::next_if => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("next_if must have a condition"),
                )
                .expect("validated next_if condition");
                return Stmt::If {
                    condition,
                    then_branch: vec![Stmt::Next],
                    else_branch: Vec::new(),
                };
            }
            Rule::next_unless => {
                let condition = build_expr(
                    mod_pair.into_inner().next().expect("next_unless must have a condition"),
                )
                .expect("validated next_unless condition");
                let negated = Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(condition),
                };
                return Stmt::If {
                    condition: negated,
                    then_branch: vec![Stmt::Next],
                    else_branch: Vec::new(),
                };
            }
            _ => {}
        }
    }

    Stmt::Next
}

fn parse_push(pair: Pair<'_, Rule>) -> Stmt {
    let mut inner = pair.into_inner();
    let array = inner
        .find(|p| p.as_rule() == Rule::ident)
        .map(parse_bare_ident)
        .expect("push must have an array name");
    let value = inner
        .find(|p| p.as_rule() == Rule::expr)
        .map(build_expr)
        .expect("push must have a value expression")
        .expect("validated push value expression");
    Stmt::Push { array, value }
}

fn parse_inc(pair: Pair<'_, Rule>) -> Stmt {
    let name = pair
        .into_inner()
        .find(|inner| inner.as_rule() == Rule::var)
        .map(parse_variable)
        .expect("inc_stmt must have a variable");
    Stmt::Assign {
        name: name.clone(),
        expr: Expr::Binary {
            left: Box::new(Expr::Variable(name)),
            op: BinaryOp::Add,
            right: Box::new(Expr::Int(1)),
        },
        declaration: false,
    }
}

fn parse_dec(pair: Pair<'_, Rule>) -> Stmt {
    let name = pair
        .into_inner()
        .find(|inner| inner.as_rule() == Rule::var)
        .map(parse_variable)
        .expect("dec_stmt must have a variable");
    Stmt::Assign {
        name: name.clone(),
        expr: Expr::Binary {
            left: Box::new(Expr::Variable(name)),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Int(1)),
        },
        declaration: false,
    }
}

fn parse_block(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    pair.into_inner()
        .filter(|inner| {
            matches!(
                inner.as_rule(),
                Rule::do_while_stmt
                    | Rule::do_until_stmt
                    | Rule::while_stmt
                    | Rule::until_stmt
                    | Rule::for_stmt
                    | Rule::foreach_stmt
                    | Rule::assign_stmt
                    | Rule::array_assign_stmt
                    | Rule::hash_assign_stmt
                    | Rule::list_assign_stmt
                    | Rule::declare_stmt
                    | Rule::if_stmt
                    | Rule::unless_stmt
                    | Rule::return_stmt
                    | Rule::die_stmt
                    | Rule::warn_stmt
                    | Rule::print_stmt
                    | Rule::say_stmt
                    | Rule::last_stmt
                    | Rule::next_stmt
                    | Rule::push_stmt
                    | Rule::inc_stmt
                    | Rule::dec_stmt
            )
        })
        .flat_map(|pair| parse_stmt(pair, max_loop_unroll))
        .collect()
}

fn parse_do_while(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let body = parse_block(inner.next().expect("do-while must have a body"), max_loop_unroll);
    let condition = build_expr(inner.next().expect("do-while must have a condition"))
        .expect("validated do-while condition");

    let has_last = contains_last(&body);
    let has_next = contains_next(&body);

    if has_last && has_next {
        // Both last and next: declare both flags, transform first body, then unroll rest
        static BREAK_FLAG: &str = "__broke";
        static SKIP_FLAG: &str = "__skipped";
        let declare_broke = Stmt::Assign {
            name: BREAK_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let declare_skipped = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let reset_skip = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: false,
        };
        let transformed_first = transform_body_for_last_and_next(body.clone(), BREAK_FLAG, SKIP_FLAG);
        let mut result = vec![declare_broke, declare_skipped, reset_skip];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_both_flags(
                condition, body, max_loop_unroll - 1, BREAK_FLAG, SKIP_FLAG,
            ));
        }
        result
    } else if has_last {
        // Only last: declare break flag, transform first body, then unroll rest
        static BREAK_FLAG: &str = "__broke";
        let declare = Stmt::Assign {
            name: BREAK_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let transformed_first = transform_body_for_last(body.clone(), BREAK_FLAG);
        let mut result = vec![declare];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_flag(
                condition, body, max_loop_unroll - 1, BREAK_FLAG,
            ));
        }
        result
    } else if has_next {
        // Only next: declare skip flag, transform first body, then unroll rest
        static SKIP_FLAG: &str = "__skipped";
        let declare = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let reset_skip = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: false,
        };
        let transformed_first = transform_body_for_next(body.clone(), SKIP_FLAG);
        let mut result = vec![declare, reset_skip];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_skip_flag(
                condition, body, max_loop_unroll - 1, SKIP_FLAG,
            ));
        }
        result
    } else {
        // No last or next: execute body once unconditionally, then unroll as while
        let mut result = body.clone();
        if max_loop_unroll > 0 {
            result.extend(unroll_while_simple(condition, body, max_loop_unroll - 1));
        }
        result
    }
}

fn parse_do_until(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let body = parse_block(inner.next().expect("do-until must have a body"), max_loop_unroll);
    let condition = build_expr(inner.next().expect("do-until must have a condition"))
        .expect("validated do-until condition");
    // Desugar: do { BODY } until (C) => do { BODY } while (!(C))
    let negated_condition = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(condition),
    };

    let has_last = contains_last(&body);
    let has_next = contains_next(&body);

    if has_last && has_next {
        static BREAK_FLAG: &str = "__broke";
        static SKIP_FLAG: &str = "__skipped";
        let declare_broke = Stmt::Assign {
            name: BREAK_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let declare_skipped = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let reset_skip = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: false,
        };
        let transformed_first = transform_body_for_last_and_next(body.clone(), BREAK_FLAG, SKIP_FLAG);
        let mut result = vec![declare_broke, declare_skipped, reset_skip];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_both_flags(
                negated_condition, body, max_loop_unroll - 1, BREAK_FLAG, SKIP_FLAG,
            ));
        }
        result
    } else if has_last {
        static BREAK_FLAG: &str = "__broke";
        let declare = Stmt::Assign {
            name: BREAK_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let transformed_first = transform_body_for_last(body.clone(), BREAK_FLAG);
        let mut result = vec![declare];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_flag(
                negated_condition, body, max_loop_unroll - 1, BREAK_FLAG,
            ));
        }
        result
    } else if has_next {
        static SKIP_FLAG: &str = "__skipped";
        let declare = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let reset_skip = Stmt::Assign {
            name: SKIP_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: false,
        };
        let transformed_first = transform_body_for_next(body.clone(), SKIP_FLAG);
        let mut result = vec![declare, reset_skip];
        result.extend(transformed_first);
        if max_loop_unroll > 0 {
            result.extend(unroll_while_with_skip_flag(
                negated_condition, body, max_loop_unroll - 1, SKIP_FLAG,
            ));
        }
        result
    } else {
        let mut result = body.clone();
        if max_loop_unroll > 0 {
            result.extend(unroll_while_simple(negated_condition, body, max_loop_unroll - 1));
        }
        result
    }
}

fn parse_while(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("while must have a condition"))
        .expect("validated while condition");
    let body = parse_block(inner.next().expect("while must have a body"), max_loop_unroll);
    unroll_while(condition, body, max_loop_unroll)
}

fn parse_until(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("until must have a condition"))
        .expect("validated until condition");
    let body = parse_block(inner.next().expect("until must have a body"), max_loop_unroll);

    // Desugar: until (COND) { BODY } => while (!(COND)) { BODY }
    let negated_condition = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(condition),
    };
    unroll_while(negated_condition, body, max_loop_unroll)
}

fn parse_for(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let inner = pair.into_inner();
    let mut init = Vec::new();
    let mut condition = None;
    let mut step = Vec::new();
    let mut body = Vec::new();

    for part in inner {
        match part.as_rule() {
            Rule::for_assign if condition.is_none() => init.push(parse_for_assign(part)),
            Rule::expr => condition = Some(build_expr(part).expect("validated for condition")),
            Rule::for_assign => step.push(parse_for_assign(part)),
            Rule::block => body = parse_block(part, max_loop_unroll),
            _ => {}
        }
    }

    let condition = condition.expect("for must have a condition");

    // If the body contains `next`, we need to keep the step separate so it
    // isn't guarded by the skip flag (Perl's `next` still executes the for-loop step).
    if contains_next(&body) {
        init.extend(unroll_for_with_next(condition, body, step, max_loop_unroll));
    } else {
        let mut loop_body = body;
        loop_body.extend(step);
        init.extend(unroll_while(condition, loop_body, max_loop_unroll));
    }
    init
}

fn parse_foreach(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let loop_var = parse_variable(inner.next().expect("foreach must have a loop variable"));
    let array_name = inner
        .next()
        .expect("foreach must have an array name")
        .as_str()
        .to_string();
    let user_body = parse_block(inner.next().expect("foreach must have a body"), max_loop_unroll);

    // Desugar: foreach my $x (@arr) { BODY }
    // =>
    // my $__foreach_i = 0;
    // while ($__foreach_i < scalar(@arr)) {
    //     my $x = $arr[$__foreach_i];
    //     BODY
    //     $__foreach_i = $__foreach_i + 1;
    // }

    let idx = "__foreach_i".to_string();

    // Init: my $__foreach_i = 0;
    let init = Stmt::Assign {
        name: idx.clone(),
        expr: Expr::Int(0),
        declaration: true,
    };

    // Condition: $__foreach_i < scalar(@arr)
    let condition = Expr::Binary {
        left: Box::new(Expr::Variable(idx.clone())),
        op: BinaryOp::Lt,
        right: Box::new(Expr::Builtin {
            function: Builtin::Scalar,
            args: vec![Expr::Variable(array_name.clone())],
        }),
    };

    // Body prepend: my $x = $arr[$__foreach_i];
    let element_assign = Stmt::Assign {
        name: loop_var,
        expr: Expr::Access {
            kind: AccessKind::Array,
            collection: array_name,
            index: Box::new(Expr::Variable(idx.clone())),
        },
        declaration: true,
    };

    // Step: $__foreach_i = $__foreach_i + 1;
    let step = Stmt::Assign {
        name: idx.clone(),
        expr: Expr::Binary {
            left: Box::new(Expr::Variable(idx)),
            op: BinaryOp::Add,
            right: Box::new(Expr::Int(1)),
        },
        declaration: false,
    };

    let mut full_body = vec![element_assign];
    full_body.extend(user_body);

    let step_stmts = vec![step];

    // Reuse the same for-loop unrolling logic as parse_for
    let mut result = vec![init];
    if contains_next(&full_body) {
        result.extend(unroll_for_with_next(condition, full_body, step_stmts, max_loop_unroll));
    } else {
        full_body.extend(step_stmts);
        result.extend(unroll_while(condition, full_body, max_loop_unroll));
    }
    result
}

fn parse_for_assign(pair: Pair<'_, Rule>) -> Stmt {
    let inner = pair
        .into_inner()
        .next()
        .expect("for assignment must contain an assignment");
    match inner.as_rule() {
        Rule::for_scalar_assign => {
            let mut inner = inner.into_inner();
            let name = parse_variable(inner.next().expect("for scalar assignment must have a var"));
            let op_pair = inner.next().expect("for scalar assignment must have an assign_op");
            let compound_op = match op_pair.as_str() {
                "+=" => Some(BinaryOp::Add),
                "-=" => Some(BinaryOp::Sub),
                "**=" => Some(BinaryOp::Pow),
                "*=" => Some(BinaryOp::Mul),
                "/=" => Some(BinaryOp::Div),
                "%=" => Some(BinaryOp::Mod),
                ".=" => Some(BinaryOp::Concat),
                "&=" => Some(BinaryOp::BitAnd),
                "|=" => Some(BinaryOp::BitOr),
                "^=" => Some(BinaryOp::BitXor),
                "<<=" => Some(BinaryOp::Shl),
                ">>=" => Some(BinaryOp::Shr),
                _ => None,
            };
            let rhs = build_expr(inner.next().expect("for scalar assignment must have an expr"))
                .expect("validated for scalar assignment");
            let expr = match compound_op {
                Some(op) => Expr::Binary {
                    left: Box::new(Expr::Variable(name.clone())),
                    op,
                    right: Box::new(rhs),
                },
                None => rhs,
            };
            Stmt::Assign {
                name,
                expr,
                declaration: false,
            }
        }
        Rule::for_array_assign => {
            let mut inner = inner.into_inner();
            let name = parse_variable(inner.next().expect("for array assignment must have a var"));
            let index =
                parse_access_operand(inner.next().expect("for array assignment must have an index"))
                    .expect("validated for array index");
            let expr = build_expr(inner.next().expect("for array assignment must have an expr"))
                .expect("validated for array assignment");
            Stmt::ArrayAssign { name, index, expr }
        }
        Rule::for_hash_assign => {
            let mut inner = inner.into_inner();
            let name = parse_variable(inner.next().expect("for hash assignment must have a var"));
            let key =
                parse_access_operand(inner.next().expect("for hash assignment must have a key"))
                    .expect("validated for hash key");
            let expr = build_expr(inner.next().expect("for hash assignment must have an expr"))
                .expect("validated for hash assignment");
            Stmt::HashAssign { name, key, expr }
        }
        Rule::for_inc => {
            let name = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::var)
                .map(parse_variable)
                .expect("for_inc must have a variable");
            Stmt::Assign {
                name: name.clone(),
                expr: Expr::Binary {
                    left: Box::new(Expr::Variable(name)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Int(1)),
                },
                declaration: false,
            }
        }
        Rule::for_dec => {
            let name = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::var)
                .map(parse_variable)
                .expect("for_dec must have a variable");
            Stmt::Assign {
                name: name.clone(),
                expr: Expr::Binary {
                    left: Box::new(Expr::Variable(name)),
                    op: BinaryOp::Sub,
                    right: Box::new(Expr::Int(1)),
                },
                declaration: false,
            }
        }
        other => unreachable!("unexpected for assignment rule: {other:?}"),
    }
}

fn unroll_while(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    let has_last = contains_last(&body);
    let has_next = contains_next(&body);
    if has_last && has_next {
        // Both last and next: use both flags
        unroll_while_with_last_and_next(condition, body, remaining)
    } else if has_last {
        unroll_while_with_last(condition, body, remaining)
    } else if has_next {
        unroll_while_with_next(condition, body, remaining)
    } else {
        unroll_while_simple(condition, body, remaining)
    }
}

fn unroll_while_simple(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    if remaining == 0 {
        return vec![Stmt::If {
            condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    let mut then_branch = body.clone();
    then_branch.extend(unroll_while_simple(condition.clone(), body, remaining - 1));
    vec![Stmt::If {
        condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

/// Unroll a while loop whose body contains `last`.
///
/// Produces:
///   my $__broke = 0;
///   if (C && $__broke == 0) {
///       <body with last replaced by $__broke=1, and subsequent stmts guarded>
///       if (C && $__broke == 0) { ... recurse ... }
///   }
fn unroll_while_with_last(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    static BREAK_FLAG: &str = "__broke";

    // Declare and initialize the break flag
    let declare = Stmt::Assign {
        name: BREAK_FLAG.to_string(),
        expr: Expr::Int(0),
        declaration: true,
    };

    let mut result = vec![declare];
    result.extend(unroll_while_with_flag(condition, body, remaining, BREAK_FLAG));
    result
}

fn unroll_while_with_flag(
    condition: Expr,
    body: Vec<Stmt>,
    remaining: usize,
    flag: &str,
) -> Vec<Stmt> {
    // The effective condition: original_condition && $flag == 0
    let flag_check = Expr::Binary {
        left: Box::new(Expr::Variable(flag.to_string())),
        op: BinaryOp::Eq,
        right: Box::new(Expr::Int(0)),
    };
    let effective_condition = Expr::Binary {
        left: Box::new(condition.clone()),
        op: BinaryOp::And,
        right: Box::new(flag_check),
    };

    if remaining == 0 {
        return vec![Stmt::If {
            condition: effective_condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    // Transform the body: replace `last` with flag assignment, guard subsequent stmts
    let transformed_body = transform_body_for_last(body.clone(), flag);

    let mut then_branch = transformed_body;
    then_branch.extend(unroll_while_with_flag(
        condition,
        body,
        remaining - 1,
        flag,
    ));

    vec![Stmt::If {
        condition: effective_condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

/// Transform a loop body to handle `last`:
/// - Replace `Stmt::Last` with `$flag = 1`
/// - After any statement that might set the flag (i.e., an if-block containing last),
///   wrap remaining statements in `if ($flag == 0) { ... }`
fn transform_body_for_last(stmts: Vec<Stmt>, flag: &str) -> Vec<Stmt> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < stmts.len() {
        let stmt = stmts[i].clone();
        i += 1;

        match stmt {
            Stmt::Last => {
                // Replace with flag = 1
                result.push(Stmt::Assign {
                    name: flag.to_string(),
                    expr: Expr::Int(1),
                    declaration: false,
                });
                // Guard all remaining statements
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_last(remaining, flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } if contains_last(&then_branch) || contains_last(&else_branch) => {
                // Recursively transform branches
                let new_then = transform_body_for_last(then_branch, flag);
                let new_else = transform_body_for_last(else_branch, flag);
                result.push(Stmt::If {
                    condition,
                    then_branch: new_then,
                    else_branch: new_else,
                });
                // Guard remaining statements since an if-branch may have set the flag
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_last(remaining, flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            other => {
                result.push(other);
            }
        }
    }

    result
}

/// Recursively check whether any statement in the slice is `Stmt::Last`.
fn contains_last(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Last => true,
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => contains_last(then_branch) || contains_last(else_branch),
        _ => false,
    })
}

/// Unroll a for loop whose body contains `next`, keeping the step outside
/// the skip-flag guard so that it always executes (matching Perl semantics).
fn unroll_for_with_next(
    condition: Expr,
    body: Vec<Stmt>,
    step: Vec<Stmt>,
    remaining: usize,
) -> Vec<Stmt> {
    static SKIP_FLAG: &str = "__skipped";

    let declare = Stmt::Assign {
        name: SKIP_FLAG.to_string(),
        expr: Expr::Int(0),
        declaration: true,
    };

    let has_last = contains_last(&body);

    if has_last {
        static BREAK_FLAG: &str = "__broke";
        let declare_broke = Stmt::Assign {
            name: BREAK_FLAG.to_string(),
            expr: Expr::Int(0),
            declaration: true,
        };
        let mut result = vec![declare_broke, declare];
        result.extend(unroll_for_with_both_flags(
            condition, body, step, remaining, BREAK_FLAG, SKIP_FLAG,
        ));
        result
    } else {
        let mut result = vec![declare];
        result.extend(unroll_for_with_skip_flag(
            condition, body, step, remaining, SKIP_FLAG,
        ));
        result
    }
}

fn unroll_for_with_skip_flag(
    condition: Expr,
    body: Vec<Stmt>,
    step: Vec<Stmt>,
    remaining: usize,
    flag: &str,
) -> Vec<Stmt> {
    if remaining == 0 {
        return vec![Stmt::If {
            condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    // Reset skip flag at start of each iteration
    let reset = Stmt::Assign {
        name: flag.to_string(),
        expr: Expr::Int(0),
        declaration: false,
    };

    // Transform body (NOT step) for next
    let transformed_body = transform_body_for_next(body.clone(), flag);

    let mut then_branch = vec![reset];
    then_branch.extend(transformed_body);
    // Step always executes (outside the skip guard)
    then_branch.extend(step.clone());
    then_branch.extend(unroll_for_with_skip_flag(
        condition.clone(),
        body,
        step,
        remaining - 1,
        flag,
    ));

    vec![Stmt::If {
        condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

fn unroll_for_with_both_flags(
    condition: Expr,
    body: Vec<Stmt>,
    step: Vec<Stmt>,
    remaining: usize,
    break_flag: &str,
    skip_flag: &str,
) -> Vec<Stmt> {
    let flag_check = Expr::Binary {
        left: Box::new(Expr::Variable(break_flag.to_string())),
        op: BinaryOp::Eq,
        right: Box::new(Expr::Int(0)),
    };
    let effective_condition = Expr::Binary {
        left: Box::new(condition.clone()),
        op: BinaryOp::And,
        right: Box::new(flag_check),
    };

    if remaining == 0 {
        return vec![Stmt::If {
            condition: effective_condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    // Reset skip flag at start of each iteration
    let reset_skip = Stmt::Assign {
        name: skip_flag.to_string(),
        expr: Expr::Int(0),
        declaration: false,
    };

    // Transform body for both last and next
    let transformed_body = transform_body_for_last_and_next(body.clone(), break_flag, skip_flag);

    let mut then_branch = vec![reset_skip];
    then_branch.extend(transformed_body);
    // Step always executes (outside the skip guard) but only if not broke
    let break_check = Expr::Binary {
        left: Box::new(Expr::Variable(break_flag.to_string())),
        op: BinaryOp::Eq,
        right: Box::new(Expr::Int(0)),
    };
    if !step.is_empty() {
        then_branch.push(Stmt::If {
            condition: break_check,
            then_branch: step.clone(),
            else_branch: Vec::new(),
        });
    }
    then_branch.extend(unroll_for_with_both_flags(
        condition,
        body,
        step,
        remaining - 1,
        break_flag,
        skip_flag,
    ));

    vec![Stmt::If {
        condition: effective_condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

/// Recursively check whether any statement in the slice is `Stmt::Next`.
fn contains_next(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Next => true,
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => contains_next(then_branch) || contains_next(else_branch),
        _ => false,
    })
}

/// Unroll a while loop whose body contains `next` (but not `last`).
///
/// Produces:
///   if (C) {
///       $__skipped = 0;
///       <body with next replaced by $__skipped=1, and subsequent stmts guarded>
///       if (C) {
///           $__skipped = 0;
///           ...
///       }
///   }
fn unroll_while_with_next(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    static SKIP_FLAG: &str = "__skipped";

    // Declare the skip flag once before the loop
    let declare = Stmt::Assign {
        name: SKIP_FLAG.to_string(),
        expr: Expr::Int(0),
        declaration: true,
    };

    let mut result = vec![declare];
    result.extend(unroll_while_with_skip_flag(
        condition, body, remaining, SKIP_FLAG,
    ));
    result
}

fn unroll_while_with_skip_flag(
    condition: Expr,
    body: Vec<Stmt>,
    remaining: usize,
    flag: &str,
) -> Vec<Stmt> {
    if remaining == 0 {
        return vec![Stmt::If {
            condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    // Reset the skip flag at the start of each iteration
    let reset = Stmt::Assign {
        name: flag.to_string(),
        expr: Expr::Int(0),
        declaration: false,
    };

    // Transform the body: replace `next` with flag assignment, guard subsequent stmts
    let transformed_body = transform_body_for_next(body.clone(), flag);

    let mut then_branch = vec![reset];
    then_branch.extend(transformed_body);
    then_branch.extend(unroll_while_with_skip_flag(
        condition.clone(),
        body,
        remaining - 1,
        flag,
    ));

    vec![Stmt::If {
        condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

/// Unroll a while loop whose body contains both `last` and `next`.
fn unroll_while_with_last_and_next(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    static BREAK_FLAG: &str = "__broke";
    static SKIP_FLAG: &str = "__skipped";

    // Declare both flags
    let declare_broke = Stmt::Assign {
        name: BREAK_FLAG.to_string(),
        expr: Expr::Int(0),
        declaration: true,
    };
    let declare_skipped = Stmt::Assign {
        name: SKIP_FLAG.to_string(),
        expr: Expr::Int(0),
        declaration: true,
    };

    let mut result = vec![declare_broke, declare_skipped];
    result.extend(unroll_while_with_both_flags(
        condition, body, remaining, BREAK_FLAG, SKIP_FLAG,
    ));
    result
}

fn unroll_while_with_both_flags(
    condition: Expr,
    body: Vec<Stmt>,
    remaining: usize,
    break_flag: &str,
    skip_flag: &str,
) -> Vec<Stmt> {
    // The effective condition: original_condition && $break_flag == 0
    let flag_check = Expr::Binary {
        left: Box::new(Expr::Variable(break_flag.to_string())),
        op: BinaryOp::Eq,
        right: Box::new(Expr::Int(0)),
    };
    let effective_condition = Expr::Binary {
        left: Box::new(condition.clone()),
        op: BinaryOp::And,
        right: Box::new(flag_check),
    };

    if remaining == 0 {
        return vec![Stmt::If {
            condition: effective_condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    // Reset skip flag at the start of each iteration
    let reset_skip = Stmt::Assign {
        name: skip_flag.to_string(),
        expr: Expr::Int(0),
        declaration: false,
    };

    // Transform the body for both last and next
    let transformed_body = transform_body_for_last_and_next(body.clone(), break_flag, skip_flag);

    let mut then_branch = vec![reset_skip];
    then_branch.extend(transformed_body);
    then_branch.extend(unroll_while_with_both_flags(
        condition,
        body,
        remaining - 1,
        break_flag,
        skip_flag,
    ));

    vec![Stmt::If {
        condition: effective_condition,
        then_branch,
        else_branch: Vec::new(),
    }]
}

/// Transform a loop body to handle `next`:
/// - Replace `Stmt::Next` with `$flag = 1`
/// - After any statement that might set the flag (i.e., an if-block containing next),
///   wrap remaining statements in `if ($flag == 0) { ... }`
fn transform_body_for_next(stmts: Vec<Stmt>, flag: &str) -> Vec<Stmt> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < stmts.len() {
        let stmt = stmts[i].clone();
        i += 1;

        match stmt {
            Stmt::Next => {
                // Replace with flag = 1
                result.push(Stmt::Assign {
                    name: flag.to_string(),
                    expr: Expr::Int(1),
                    declaration: false,
                });
                // Guard all remaining statements
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_next(remaining, flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } if contains_next(&then_branch) || contains_next(&else_branch) => {
                // Recursively transform branches
                let new_then = transform_body_for_next(then_branch, flag);
                let new_else = transform_body_for_next(else_branch, flag);
                result.push(Stmt::If {
                    condition,
                    then_branch: new_then,
                    else_branch: new_else,
                });
                // Guard remaining statements since an if-branch may have set the flag
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_next(remaining, flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            other => {
                result.push(other);
            }
        }
    }

    result
}

/// Transform a loop body to handle both `last` and `next`:
fn transform_body_for_last_and_next(stmts: Vec<Stmt>, break_flag: &str, skip_flag: &str) -> Vec<Stmt> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < stmts.len() {
        let stmt = stmts[i].clone();
        i += 1;

        match stmt {
            Stmt::Last => {
                result.push(Stmt::Assign {
                    name: break_flag.to_string(),
                    expr: Expr::Int(1),
                    declaration: false,
                });
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_last_and_next(remaining, break_flag, skip_flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(break_flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            Stmt::Next => {
                result.push(Stmt::Assign {
                    name: skip_flag.to_string(),
                    expr: Expr::Int(1),
                    declaration: false,
                });
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_last_and_next(remaining, break_flag, skip_flag);
                    let flag_check = Expr::Binary {
                        left: Box::new(Expr::Variable(skip_flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    result.push(Stmt::If {
                        condition: flag_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } if contains_last(&then_branch) || contains_last(&else_branch)
                || contains_next(&then_branch) || contains_next(&else_branch) => {
                let new_then = transform_body_for_last_and_next(then_branch, break_flag, skip_flag);
                let new_else = transform_body_for_last_and_next(else_branch, break_flag, skip_flag);
                result.push(Stmt::If {
                    condition,
                    then_branch: new_then,
                    else_branch: new_else,
                });
                if i < stmts.len() {
                    let remaining: Vec<Stmt> = stmts[i..].to_vec();
                    let guarded = transform_body_for_last_and_next(remaining, break_flag, skip_flag);
                    // Guard with both flags: remaining runs only if neither broke nor skipped
                    let break_check = Expr::Binary {
                        left: Box::new(Expr::Variable(break_flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    let skip_check = Expr::Binary {
                        left: Box::new(Expr::Variable(skip_flag.to_string())),
                        op: BinaryOp::Eq,
                        right: Box::new(Expr::Int(0)),
                    };
                    let combined_check = Expr::Binary {
                        left: Box::new(break_check),
                        op: BinaryOp::And,
                        right: Box::new(skip_check),
                    };
                    result.push(Stmt::If {
                        condition: combined_check,
                        then_branch: guarded,
                        else_branch: Vec::new(),
                    });
                }
                break;
            }
            other => {
                result.push(other);
            }
        }
    }

    result
}
fn parse_elsif_clause(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> (Expr, Vec<Stmt>) {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("elsif must have a condition"))
        .expect("validated elsif condition expression");
    let block = parse_block(inner.next().expect("elsif must have a block"), max_loop_unroll);
    (condition, block)
}

fn parse_else_clause(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    parse_block(
        pair.into_inner()
            .next()
            .expect("else clause must contain a block"),
        max_loop_unroll,
    )
}

fn parse_variable(pair: Pair<'_, Rule>) -> String {
    pair.as_str().trim_start_matches('$').to_string()
}

fn parse_bare_ident(pair: Pair<'_, Rule>) -> String {
    pair.as_str().to_string()
}

fn build_expr(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    match pair.as_rule() {
        Rule::ternary_expr => {
            let mut inner = pair.into_inner();
            let condition_pair = inner
                .next()
                .expect("ternary_expr must start with simple_expr");
            let condition = build_simple_expr(condition_pair)?;

            if let Some(then_pair) = inner.next() {
                let then_expr = build_simple_expr(then_pair)?;
                let else_expr = build_expr(inner.next().expect("ternary must have else branch"))?;
                Ok(Expr::Ternary {
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                })
            } else {
                Ok(condition)
            }
        }
        Rule::expr => {
            let ternary_pair = pair
                .into_inner()
                .next()
                .expect("expr must contain ternary_expr");
            build_expr(ternary_pair)
        }
        Rule::simple_expr => build_simple_expr(pair),
        other => Err(format!("unexpected expression rule: {other:?}")),
    }
}

fn build_simple_expr(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    PrattParser::new()
        .op(pest::pratt_parser::Op::infix(
            Rule::op_low_or,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_low_and,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_or,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_and,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_bitor,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_bitxor,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_bitand,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(
            pest::pratt_parser::Op::infix(Rule::op_eq, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_ne, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_seq, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sne, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_slt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sgt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sle, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sge, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_cmp, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_spaceship, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_lt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_le, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_gt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_ge, pest::pratt_parser::Assoc::Left),
        )
        .op(
            pest::pratt_parser::Op::infix(Rule::op_shl, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_shr, pest::pratt_parser::Assoc::Left),
        )
        .op(
            pest::pratt_parser::Op::infix(Rule::op_add, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sub, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_concat, pest::pratt_parser::Assoc::Left),
        )
        .op(
            pest::pratt_parser::Op::infix(Rule::op_repeat, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_mul, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_div, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_mod, pest::pratt_parser::Assoc::Left),
        )
        .op(pest::pratt_parser::Op::infix(
            Rule::op_pow,
            pest::pratt_parser::Assoc::Right,
        ))
        .op(pest::pratt_parser::Op::prefix(Rule::op_not)
            | pest::pratt_parser::Op::prefix(Rule::op_neg)
            | pest::pratt_parser::Op::prefix(Rule::op_bitnot)
            | pest::pratt_parser::Op::prefix(Rule::op_low_not))
        .map_primary(|primary| match primary.as_rule() {
            Rule::int => {
                let s = primary.as_str();
                let stripped: String = s.chars().filter(|&c| c != '_').collect();
                let value = if let Some(hex) = stripped.strip_prefix("0x") {
                    i64::from_str_radix(hex, 16).map_err(|_| {
                        format!("invalid hex integer: {}", s)
                    })?
                } else if let Some(oct) = stripped.strip_prefix("0o") {
                    i64::from_str_radix(oct, 8).map_err(|_| {
                        format!("invalid octal integer: {}", s)
                    })?
                } else if let Some(bin) = stripped.strip_prefix("0b") {
                    i64::from_str_radix(bin, 2).map_err(|_| {
                        format!("invalid binary integer: {}", s)
                    })?
                } else {
                    stripped.parse().map_err(|_| {
                        format!("invalid integer: {}", s)
                    })?
                };
                Ok(Expr::Int(value))
            }
            Rule::string => Ok(Expr::String(parse_string_literal(primary.as_str())?)),
            Rule::var => Ok(Expr::Variable(parse_variable(primary))),
            Rule::array_access => parse_collection_access(primary, AccessKind::Array),
            Rule::hash_access => parse_collection_access(primary, AccessKind::Hash),
            Rule::call_expr => parse_call_expr(primary),
            Rule::expr => build_expr(primary),
            Rule::scalar_call => parse_scalar_call(primary),
            Rule::pop_call => parse_pop_call(primary),
            Rule::length_call => parse_builtin_call(primary, Builtin::Length),
            Rule::substr_call => parse_substr_call(primary),
            Rule::index_call => parse_builtin_call(primary, Builtin::Index),
            Rule::abs_call => parse_builtin_call(primary, Builtin::Abs),
            Rule::min_call => parse_builtin_call(primary, Builtin::Min),
            Rule::max_call => parse_builtin_call(primary, Builtin::Max),
            Rule::ord_call => parse_builtin_call(primary, Builtin::Ord),
            Rule::chr_call => parse_builtin_call(primary, Builtin::Chr),
            Rule::chomp_call => parse_builtin_call(primary, Builtin::Chomp),
            Rule::reverse_call => parse_builtin_call(primary, Builtin::Reverse),
            Rule::int_call => parse_builtin_call(primary, Builtin::Int),
            Rule::contains_call => parse_builtin_call(primary, Builtin::Contains),
            Rule::starts_with_call => parse_builtin_call(primary, Builtin::StartsWith),
            Rule::ends_with_call => parse_builtin_call(primary, Builtin::EndsWith),
            Rule::replace_call => parse_builtin_call(primary, Builtin::Replace),
            Rule::char_at_call => parse_builtin_call(primary, Builtin::CharAt),
            other => Err(format!("unexpected primary rule: {other:?}")),
        })
        .map_prefix(|op, rhs| {
            Ok(Expr::Unary {
                op: match op.as_rule() {
                    Rule::op_not | Rule::op_low_not => UnaryOp::Not,
                    Rule::op_neg => UnaryOp::Neg,
                    Rule::op_bitnot => UnaryOp::BitNot,
                    other => return Err(format!("unexpected prefix operator: {other:?}")),
                },
                expr: Box::new(rhs?),
            })
        })
        .map_infix(|lhs, op, rhs| {
            Ok(Expr::Binary {
                left: Box::new(lhs?),
                op: match op.as_rule() {
                    Rule::op_add => BinaryOp::Add,
                    Rule::op_sub => BinaryOp::Sub,
                    Rule::op_mul => BinaryOp::Mul,
                    Rule::op_div => BinaryOp::Div,
                    Rule::op_mod => BinaryOp::Mod,
                    Rule::op_pow => BinaryOp::Pow,
                    Rule::op_concat => BinaryOp::Concat,
                    Rule::op_lt => BinaryOp::Lt,
                    Rule::op_le => BinaryOp::Le,
                    Rule::op_gt => BinaryOp::Gt,
                    Rule::op_ge => BinaryOp::Ge,
                    Rule::op_eq => BinaryOp::Eq,
                    Rule::op_ne => BinaryOp::Ne,
                    Rule::op_seq => BinaryOp::StrEq,
                    Rule::op_sne => BinaryOp::StrNe,
                    Rule::op_slt => BinaryOp::StrLt,
                    Rule::op_sgt => BinaryOp::StrGt,
                    Rule::op_sle => BinaryOp::StrLe,
                    Rule::op_sge => BinaryOp::StrGe,
                    Rule::op_cmp => BinaryOp::Cmp,
                    Rule::op_and | Rule::op_low_and => BinaryOp::And,
                    Rule::op_or | Rule::op_low_or => BinaryOp::Or,
                    Rule::op_bitand => BinaryOp::BitAnd,
                    Rule::op_bitor => BinaryOp::BitOr,
                    Rule::op_bitxor => BinaryOp::BitXor,
                    Rule::op_shl => BinaryOp::Shl,
                    Rule::op_shr => BinaryOp::Shr,
                    Rule::op_spaceship => BinaryOp::Spaceship,
                    Rule::op_repeat => BinaryOp::Repeat,
                    other => return Err(format!("unexpected infix operator: {other:?}")),
                },
                right: Box::new(rhs?),
            })
        })
        .parse(pair.into_inner())
}

fn parse_scalar_call(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    let mut inner = pair.into_inner();
    let ident = inner
        .next()
        .ok_or_else(|| "scalar call must have an identifier".to_string())?;
    let name = parse_bare_ident(ident);
    Ok(Expr::Builtin {
        function: Builtin::Scalar,
        args: vec![Expr::Variable(name)],
    })
}

fn parse_pop_call(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    let mut inner = pair.into_inner();
    let ident = inner
        .next()
        .ok_or_else(|| "pop call must have an identifier".to_string())?;
    let name = parse_bare_ident(ident);
    Ok(Expr::Pop { array: name })
}

fn parse_builtin_call(
    pair: Pair<'_, Rule>,
    function: Builtin,
) -> std::result::Result<Expr, String> {
    let args = pair
        .into_inner()
        .filter(|inner| inner.as_rule() == Rule::expr)
        .map(build_expr)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Expr::Builtin { function, args })
}

/// Parse substr with 2 or 3 arguments.
/// 2-arg form `substr($s, $off)` is desugared to `substr($s, $off, length($s) - $off)`.
fn parse_substr_call(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    let mut args = pair
        .into_inner()
        .filter(|inner| inner.as_rule() == Rule::expr)
        .map(build_expr)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if args.len() == 2 {
        let str_expr = args[0].clone();
        let offset_expr = args[1].clone();
        let length_expr = Expr::Binary {
            left: Box::new(Expr::Builtin {
                function: Builtin::Length,
                args: vec![str_expr],
            }),
            op: BinaryOp::Sub,
            right: Box::new(offset_expr),
        };
        args.push(length_expr);
    }
    Ok(Expr::Builtin {
        function: Builtin::Substr,
        args,
    })
}

fn parse_collection_access(
    pair: Pair<'_, Rule>,
    kind: AccessKind,
) -> std::result::Result<Expr, String> {
    let mut inner = pair.into_inner();
    let collection = parse_variable(inner.next().expect("collection access must have a variable"));
    let index = parse_access_operand(inner.next().expect("collection access must have an index"))?;
    Ok(Expr::Access {
        kind,
        collection,
        index: Box::new(index),
    })
}

fn parse_access_operand(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    match pair.as_rule() {
        Rule::access_index | Rule::access_key => {
            let inner = pair
                .into_inner()
                .next()
                .expect("access operand must contain a value");
            parse_access_operand(inner)
        }
        Rule::bare_ident => Ok(Expr::Variable(parse_bare_ident(pair))),
        Rule::expr => build_expr(pair),
        Rule::var => Ok(Expr::Variable(parse_variable(pair))),
        Rule::int => {
            let s = pair.as_str();
            let stripped: String = s.chars().filter(|&c| c != '_').collect();
            let value = if let Some(hex) = stripped.strip_prefix("0x") {
                i64::from_str_radix(hex, 16).map_err(|_| {
                    format!("invalid hex integer: {}", s)
                })?
            } else if let Some(oct) = stripped.strip_prefix("0o") {
                i64::from_str_radix(oct, 8).map_err(|_| {
                    format!("invalid octal integer: {}", s)
                })?
            } else if let Some(bin) = stripped.strip_prefix("0b") {
                i64::from_str_radix(bin, 2).map_err(|_| {
                    format!("invalid binary integer: {}", s)
                })?
            } else {
                stripped.parse().map_err(|_| {
                    format!("invalid integer: {}", s)
                })?
            };
            Ok(Expr::Int(value))
        }
        Rule::string => Ok(Expr::String(parse_string_literal(pair.as_str())?)),
        other => Err(format!("unexpected access operand: {other:?}")),
    }
}

fn parse_call_expr(pair: Pair<'_, Rule>) -> std::result::Result<Expr, String> {
    let mut inner = pair.into_inner();
    let function = parse_bare_ident(inner.next().expect("call must have a function name"));
    let mut args = Vec::new();
    for pair in inner {
        match pair.as_rule() {
            Rule::call_args => {
                for arg in pair.into_inner().filter(|arg| arg.as_rule() == Rule::expr) {
                    args.push(build_expr(arg)?);
                }
            }
            Rule::expr => args.push(build_expr(pair)?),
            _ => {}
        }
    }
    Ok(Expr::Call { function, args })
}

fn parse_string_literal(raw: &str) -> std::result::Result<String, String> {
    let mut chars = raw.chars();
    let quote = chars.next();
    let is_single = match quote {
        Some('\'') => {
            if chars.next_back() != Some('\'') {
                return Err(format!("invalid string literal: {raw}"));
            }
            true
        }
        Some('"') => {
            if chars.next_back() != Some('"') {
                return Err(format!("invalid string literal: {raw}"));
            }
            false
        }
        _ => return Err(format!("invalid string literal: {raw}")),
    };

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            if is_single {
                // Single-quoted: only \\ and \' are escape sequences
                match ch {
                    '\'' | '\\' => value.push(ch),
                    other => {
                        // Not a recognized escape — emit the backslash literally
                        value.push('\\');
                        value.push(other);
                    }
                }
            } else {
                // Double-quoted: full escape processing
                match ch {
                    '"' | '\\' => value.push(ch),
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    other => return Err(format!("unsupported escape sequence: \\{other}")),
                }
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            value.push(ch);
        }
    }

    if escaped {
        return Err("unterminated string escape".to_string());
    }

    Ok(value)
}

fn render_expr_error(error: pest::error::Error<Rule>) -> String {
    let (line, column) = match error.line_col {
        pest::error::LineColLocation::Pos((line, column)) => (line, column),
        pest::error::LineColLocation::Span((line, column), _) => (line, column),
    };
    format!("syntax error at {line}:{column}: {error}")
}

fn render_body_error(error: pest::error::Error<Rule>) -> ((usize, usize), String) {
    let (line, column) = match error.line_col {
        pest::error::LineColLocation::Pos((line, column)) => (line, column),
        pest::error::LineColLocation::Span((line, column), _) => (line, column),
    };
    ((line, column), error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{AccessKind, BinaryOp, Builtin, Expr, FunctionAst, Stmt},
        extractor::ExtractedFunction,
    };

    use super::{parse_expr, parse_function_ast};

    #[test]
    fn parses_valid_program_into_function_ast() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($x, $y) = @_;
    my $z = $x + $y;
    if ($z > 0) {
        return $z;
    } else {
        return $x;
    }
"#
            .to_string(),
            start_line: 1,
        };

        let ast = parse_function_ast(&function).unwrap();

        assert_eq!(
            ast,
            FunctionAst {
                name: "foo".to_string(),
                params: vec!["x".to_string(), "y".to_string()],
                body: vec![
                    Stmt::Assign {
                        name: "z".to_string(),
                        declaration: true,
                        expr: Expr::Binary {
                            left: Box::new(Expr::Variable("x".to_string())),
                            op: BinaryOp::Add,
                            right: Box::new(Expr::Variable("y".to_string())),
                        },
                    },
                    Stmt::If {
                        condition: Expr::Binary {
                            left: Box::new(Expr::Variable("z".to_string())),
                            op: BinaryOp::Gt,
                            right: Box::new(Expr::Int(0)),
                        },
                        then_branch: vec![Stmt::Return(Expr::Variable("z".to_string()))],
                        else_branch: vec![Stmt::Return(Expr::Variable("x".to_string()))],
                    },
                ],
            }
        );
    }

    #[test]
    fn parses_elsif_chain_as_nested_if() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($x) = @_;
    if ($x < 0) {
        return 0;
    } elsif ($x == 0) {
        return 1;
    } else {
        return 2;
    }
"#
            .to_string(),
            start_line: 1,
        };

        let ast = parse_function_ast(&function).unwrap();

        assert!(matches!(
            &ast.body[0],
            Stmt::If {
                else_branch,
                ..
            } if matches!(&else_branch[0], Stmt::If { .. })
        ));
    }

    #[test]
    fn parses_string_builtins_and_concat() {
        let expr = parse_expr(r#"substr($x . "!", 0, length($y)) eq $y"#).unwrap();
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::StrEq,
                ..
            }
        ));
    }

    #[test]
    fn parses_string_literal_and_builtin_calls() {
        let expr = parse_expr(r#"index("hello", "he")"#).unwrap();
        assert_eq!(
            expr,
            Expr::Builtin {
                function: Builtin::Index,
                args: vec![
                    Expr::String("hello".to_string()),
                    Expr::String("he".to_string())
                ],
            }
        );
    }

    #[test]
    fn parses_modulo_with_multiplicative_precedence() {
        let expr = parse_expr("$x + $y % 2").unwrap();
        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::Add,
                right,
                ..
            } if matches!(
                right.as_ref(),
                Expr::Binary {
                    op: BinaryOp::Mod,
                    ..
                }
            )
        ));
    }

    #[test]
    fn parses_array_and_hash_access_forms() {
        let array_expr = parse_expr("$arr[i]").unwrap();
        assert!(matches!(
            array_expr,
            Expr::Access {
                kind: AccessKind::Array,
                collection,
                ..
            } if collection == "arr"
        ));

        let hash_expr = parse_expr(r#"$h{$k}"#).unwrap();
        assert!(matches!(
            hash_expr,
            Expr::Access {
                kind: AccessKind::Hash,
                collection,
                ..
            } if collection == "h"
        ));
    }

    #[test]
    fn parses_function_calls() {
        let expr = parse_expr("foo($x, 1)").unwrap();
        assert!(matches!(
            expr,
            Expr::Call { function, args } if function == "foo" && args.len() == 2
        ));
    }

    #[test]
    fn lowers_while_and_for_into_conditionals() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($x) = @_;
    my $i;
    while ($x > 0) {
        $x = $x - 1;
    }
    for ($i = 0; $i < 2; $i = $i + 1) {
        $x = $x + 1;
    }
    return $x;
"#
            .to_string(),
            start_line: 1,
        };

        let ast = parse_function_ast(&function).unwrap();
        assert!(matches!(ast.body[1], Stmt::If { .. }));
        assert!(matches!(ast.body[3], Stmt::If { .. }));
    }

    #[test]
    fn rejects_else_if_alias() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($x) = @_;
    if ($x < 0) {
        return 0;
    } else if ($x == 0) {
        return 1;
    }
"#
            .to_string(),
            start_line: 1,
        };

        assert!(parse_function_ast(&function).is_err());
    }

    #[test]
    fn parses_scalar_call() {
        let function = ExtractedFunction {
            name: "test_scalar".to_string(),
            annotations: Vec::new(),
            body: r#"
    my ($arr) = @_;
    my $len = scalar(@arr);
    return $len;
"#
            .to_string(),
            start_line: 1,
        };

        let ast = parse_function_ast(&function).unwrap();
        assert_eq!(ast.name, "test_scalar");
        assert_eq!(ast.params, vec!["arr"]);
        // Check that we have some statements (assignment and return)
        assert!(ast.body.len() >= 2);
    }
}
