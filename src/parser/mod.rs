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
                Rule::while_stmt
                    | Rule::for_stmt
                    | Rule::assign_stmt
                    | Rule::array_assign_stmt
                    | Rule::hash_assign_stmt
                    | Rule::declare_stmt
                    | Rule::if_stmt
                    | Rule::return_stmt
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
        Rule::declare_stmt => vec![parse_declare(pair)],
        Rule::if_stmt => vec![parse_if(pair, max_loop_unroll)],
        Rule::while_stmt => parse_while(pair, max_loop_unroll),
        Rule::for_stmt => parse_for(pair, max_loop_unroll),
        Rule::return_stmt => vec![parse_return(pair)],
        other => unreachable!("unexpected statement rule: {other:?}"),
    }
}

fn parse_assign(pair: Pair<'_, Rule>) -> Stmt {
    let mut declaration = false;
    let mut name = None;
    let mut expr = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::declaration => declaration = true,
            Rule::var => name = Some(parse_variable(inner)),
            Rule::expr => expr = Some(build_expr(inner).expect("validated expression")),
            _ => {}
        }
    }

    Stmt::Assign {
        name: name.expect("assignment must have a variable"),
        expr: expr.expect("assignment must have an expression"),
        declaration,
    }
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

fn parse_return(pair: Pair<'_, Rule>) -> Stmt {
    let expr = pair
        .into_inner()
        .find(|inner| inner.as_rule() == Rule::expr)
        .map(build_expr)
        .expect("return must contain an expression")
        .expect("validated return expression");

    Stmt::Return(expr)
}

fn parse_block(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    pair.into_inner()
        .filter(|inner| {
            matches!(
                inner.as_rule(),
                Rule::while_stmt
                    | Rule::for_stmt
                    | Rule::assign_stmt
                    | Rule::array_assign_stmt
                    | Rule::hash_assign_stmt
                    | Rule::declare_stmt
                    | Rule::if_stmt
                    | Rule::return_stmt
            )
        })
        .flat_map(|pair| parse_stmt(pair, max_loop_unroll))
        .collect()
}

fn parse_while(pair: Pair<'_, Rule>, max_loop_unroll: usize) -> Vec<Stmt> {
    let mut inner = pair.into_inner();
    let condition = build_expr(inner.next().expect("while must have a condition"))
        .expect("validated while condition");
    let body = parse_block(inner.next().expect("while must have a body"), max_loop_unroll);
    unroll_while(condition, body, max_loop_unroll)
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
    let mut loop_body = body;
    loop_body.extend(step);
    init.extend(unroll_while(condition, loop_body, max_loop_unroll));
    init
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
            let expr = build_expr(inner.next().expect("for scalar assignment must have an expr"))
                .expect("validated for scalar assignment");
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
        other => unreachable!("unexpected for assignment rule: {other:?}"),
    }
}

fn unroll_while(condition: Expr, body: Vec<Stmt>, remaining: usize) -> Vec<Stmt> {
    if remaining == 0 {
        return vec![Stmt::If {
            condition,
            then_branch: vec![Stmt::LoopBoundExceeded],
            else_branch: Vec::new(),
        }];
    }

    let mut then_branch = body.clone();
    then_branch.extend(unroll_while(condition.clone(), body, remaining - 1));
    vec![Stmt::If {
        condition,
        then_branch,
        else_branch: Vec::new(),
    }]
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
    PrattParser::new()
        .op(pest::pratt_parser::Op::infix(
            Rule::op_or,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(pest::pratt_parser::Op::infix(
            Rule::op_and,
            pest::pratt_parser::Assoc::Left,
        ))
        .op(
            pest::pratt_parser::Op::infix(Rule::op_eq, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_ne, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_seq, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sne, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_lt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_le, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_gt, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_ge, pest::pratt_parser::Assoc::Left),
        )
        .op(
            pest::pratt_parser::Op::infix(Rule::op_add, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_sub, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_concat, pest::pratt_parser::Assoc::Left),
        )
        .op(
            pest::pratt_parser::Op::infix(Rule::op_mul, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_div, pest::pratt_parser::Assoc::Left)
                | pest::pratt_parser::Op::infix(Rule::op_mod, pest::pratt_parser::Assoc::Left),
        )
        .op(pest::pratt_parser::Op::prefix(Rule::op_not)
            | pest::pratt_parser::Op::prefix(Rule::op_neg))
        .map_primary(|primary| match primary.as_rule() {
            Rule::int => {
                Ok(Expr::Int(primary.as_str().parse().map_err(|_| {
                    format!("invalid integer: {}", primary.as_str())
                })?))
            }
            Rule::string => Ok(Expr::String(parse_string_literal(primary.as_str())?)),
            Rule::var => Ok(Expr::Variable(parse_variable(primary))),
            Rule::array_access => parse_collection_access(primary, AccessKind::Array),
            Rule::hash_access => parse_collection_access(primary, AccessKind::Hash),
            Rule::call_expr => parse_call_expr(primary),
            Rule::expr => build_expr(primary),
            Rule::scalar_call => parse_scalar_call(primary),
            Rule::length_call => parse_builtin_call(primary, Builtin::Length),
            Rule::substr_call => parse_builtin_call(primary, Builtin::Substr),
            Rule::index_call => parse_builtin_call(primary, Builtin::Index),
            other => Err(format!("unexpected primary rule: {other:?}")),
        })
        .map_prefix(|op, rhs| {
            Ok(Expr::Unary {
                op: match op.as_rule() {
                    Rule::op_not => UnaryOp::Not,
                    Rule::op_neg => UnaryOp::Neg,
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
                    Rule::op_concat => BinaryOp::Concat,
                    Rule::op_lt => BinaryOp::Lt,
                    Rule::op_le => BinaryOp::Le,
                    Rule::op_gt => BinaryOp::Gt,
                    Rule::op_ge => BinaryOp::Ge,
                    Rule::op_eq => BinaryOp::Eq,
                    Rule::op_ne => BinaryOp::Ne,
                    Rule::op_seq => BinaryOp::StrEq,
                    Rule::op_sne => BinaryOp::StrNe,
                    Rule::op_and => BinaryOp::And,
                    Rule::op_or => BinaryOp::Or,
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
        Rule::int => Ok(Expr::Int(
            pair.as_str()
                .parse()
                .map_err(|_| format!("invalid integer: {}", pair.as_str()))?,
        )),
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
    if chars.next() != Some('"') || chars.next_back() != Some('"') {
        return Err(format!("invalid string literal: {raw}"));
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' | '\\' => value.push(ch),
                other => return Err(format!("unsupported escape sequence: \\{other}")),
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
