use std::collections::BTreeSet;

use thiserror::Error;
use tracing::debug;

use crate::{
    ast::{Expr, Type},
    extractor::ExtractedFunction,
    parser,
};

pub const SIG_PREFIX: &str = "# sig:";
pub const PRE_PREFIX: &str = "# pre:";
pub const POST_PREFIX: &str = "# post:";
pub const POS_PREFIX: &str = "# pos:";
pub const EXTERN_PREFIX: &str = "# extern:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpec {
    pub name: String,
    pub arg_types: Vec<Type>,
    pub ret_type: Type,
    pub pre: Expr,
    pub post: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternSpec {
    pub name: String,
    pub param_types: Vec<Type>,
    pub return_type: Type,
    pub precondition: Expr,
    pub postcondition: Expr,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnnotationError {
    #[error("function `{function}` is missing a `# sig:` annotation")]
    MissingSignature { function: String },
    #[error("function `{function}` is missing a `# post:` annotation")]
    MissingPost { function: String },
    #[error("function `{function}` has duplicate `{directive}` annotations")]
    DuplicateDirective {
        function: String,
        directive: &'static str,
    },
    #[error("function `{function}` has an unknown annotation: {annotation}")]
    UnknownAnnotation {
        function: String,
        annotation: String,
    },
    #[error("function `{function}` has an invalid signature: {signature}")]
    InvalidSignature { function: String, signature: String },
    #[error("function `{function}` uses unsupported type `{type_name}`")]
    UnsupportedType { function: String, type_name: String },
    #[error(
        "function `{function}` declares {type_count} signature arguments but extracts {param_count} parameters"
    )]
    ParameterCountMismatch {
        function: String,
        type_count: usize,
        param_count: usize,
    },
    #[error("function `{function}` cannot infer parameters from the body")]
    InvalidParameterBinding { function: String },
    #[error("function `{function}` references unknown variable `${variable}`")]
    MissingVariable { function: String, variable: String },
    #[error("function `{function}` has an invalid {directive} expression: {expression}")]
    InvalidExpression {
        function: String,
        directive: &'static str,
        expression: String,
    },
    #[error("invalid extern declaration: {declaration}")]
    InvalidExternDeclaration { declaration: String },
    #[error("extern `{name}` uses unsupported type `{type_name}`")]
    UnsupportedExternType { name: String, type_name: String },
    #[error("extern `{name}` has an invalid expression: {expression}")]
    InvalidExternExpression { name: String, expression: String },
}

pub fn parse_function_spec(
    function: &ExtractedFunction,
) -> std::result::Result<FunctionSpec, AnnotationError> {
    let function_name = function.name.clone();
    let param_names = infer_param_names(&function.body).ok_or_else(|| {
        AnnotationError::InvalidParameterBinding {
            function: function_name.clone(),
        }
    })?;

    let mut signature = None;
    let mut pre = None;
    let mut post = None;

    for annotation in &function.annotations {
        let trimmed = annotation.trim();
        if let Some(raw_signature) = trimmed.strip_prefix(SIG_PREFIX) {
            if signature.is_some() {
                return Err(AnnotationError::DuplicateDirective {
                    function: function_name.clone(),
                    directive: SIG_PREFIX,
                });
            }
            signature = Some(parse_signature(function, raw_signature.trim())?);
        } else if let Some(raw_pre) = trimmed.strip_prefix(PRE_PREFIX) {
            if pre.is_some() {
                return Err(AnnotationError::DuplicateDirective {
                    function: function_name.clone(),
                    directive: PRE_PREFIX,
                });
            }
            pre = Some(parse_expression(
                &function_name,
                PRE_PREFIX,
                raw_pre.trim(),
            )?);
        } else if let Some(raw_post) = trimmed.strip_prefix(POST_PREFIX) {
            if post.is_some() {
                return Err(AnnotationError::DuplicateDirective {
                    function: function_name.clone(),
                    directive: POST_PREFIX,
                });
            }
            post = Some(parse_expression(
                &function_name,
                POST_PREFIX,
                raw_post.trim(),
            )?);
        } else if let Some(raw_post) = trimmed.strip_prefix(POS_PREFIX) {
            if post.is_some() {
                return Err(AnnotationError::DuplicateDirective {
                    function: function_name.clone(),
                    directive: POST_PREFIX,
                });
            }
            post = Some(parse_expression(
                &function_name,
                POST_PREFIX,
                raw_post.trim(),
            )?);
        } else {
            return Err(AnnotationError::UnknownAnnotation {
                function: function_name.clone(),
                annotation: annotation.clone(),
            });
        }
    }

    let (arg_types, ret_type) = signature.ok_or_else(|| AnnotationError::MissingSignature {
        function: function_name.clone(),
    })?;
    let post = post.ok_or_else(|| AnnotationError::MissingPost {
        function: function_name.clone(),
    })?;
    let pre = pre.unwrap_or(Expr::Bool(true));

    if arg_types.len() != param_names.len() {
        return Err(AnnotationError::ParameterCountMismatch {
            function: function_name.clone(),
            type_count: arg_types.len(),
            param_count: param_names.len(),
        });
    }

    validate_variables(&function_name, &param_names, &pre)?;
    validate_variables(&function_name, &param_names, &post)?;

    let spec = FunctionSpec {
        name: function_name,
        arg_types,
        ret_type,
        pre,
        post,
    };
    debug!(
        function = spec.name,
        arg_count = spec.arg_types.len(),
        "parsed function annotations"
    );
    Ok(spec)
}

fn parse_signature(
    function: &ExtractedFunction,
    signature: &str,
) -> std::result::Result<(Vec<Type>, Type), AnnotationError> {
    let (raw_args, raw_ret) =
        signature
            .split_once("->")
            .ok_or_else(|| AnnotationError::InvalidSignature {
                function: function.name.clone(),
                signature: signature.to_string(),
            })?;

    let args = raw_args.trim();
    if !args.starts_with('(') || !args.ends_with(')') {
        return Err(AnnotationError::InvalidSignature {
            function: function.name.clone(),
            signature: signature.to_string(),
        });
    }

    let arg_types = split_signature_types(&args[1..args.len() - 1])
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .map(|part| parse_type(&function.name, &part))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let ret_type = parse_type(&function.name, raw_ret.trim())?;

    Ok((arg_types, ret_type))
}

fn split_signature_types(raw: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (index, ch) in raw.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(raw[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < raw.len() {
        parts.push(raw[start..].trim());
    }
    parts
}

fn parse_type(function: &str, raw_type: &str) -> std::result::Result<Type, AnnotationError> {
    match raw_type {
        "Int" => Ok(Type::Int),
        "Str" => Ok(Type::Str),
        "Array<Int>" => Ok(Type::ArrayInt),
        "Array<Str>" => Ok(Type::ArrayStr),
        "Hash<Str, Int>" => Ok(Type::HashInt),
        "Hash<Str, Str>" => Ok(Type::HashStr),
        other => Err(AnnotationError::UnsupportedType {
            function: function.to_string(),
            type_name: other.to_string(),
        }),
    }
}

fn infer_param_names(body: &str) -> Option<Vec<String>> {
    let first_stmt = body.lines().map(str::trim).find(|line| !line.is_empty())?;
    let without_prefix = first_stmt.strip_prefix("my (")?;
    let (vars, suffix) = without_prefix.split_once(')')?;
    if suffix.trim() != "= @_;" {
        return None;
    }

    let mut names = Vec::new();
    for variable in vars.split(',') {
        let variable = variable.trim();
        let name = variable.strip_prefix('$')?;
        if name.is_empty() {
            return None;
        }
        names.push(name.to_string());
    }

    Some(names)
}

fn validate_variables(
    function: &str,
    params: &[String],
    expr: &Expr,
) -> std::result::Result<(), AnnotationError> {
    let allowed = params
        .iter()
        .map(String::as_str)
        .chain(std::iter::once("result"))
        .collect::<BTreeSet<_>>();

    for variable in collect_variables(expr) {
        if !allowed.contains(variable.as_str()) {
            return Err(AnnotationError::MissingVariable {
                function: function.to_string(),
                variable,
            });
        }
    }

    Ok(())
}

fn collect_variables(expr: &Expr) -> Vec<String> {
    let mut variables = BTreeSet::new();
    collect_variables_inner(expr, &mut variables);
    variables.into_iter().collect()
}

fn collect_variables_inner(expr: &Expr, variables: &mut BTreeSet<String>) {
    match expr {
        Expr::Variable(name) => {
            variables.insert(name.clone());
        }
        Expr::Unary { expr, .. } => collect_variables_inner(expr, variables),
        Expr::Binary { left, right, .. } => {
            collect_variables_inner(left, variables);
            collect_variables_inner(right, variables);
        }
        Expr::Ternary { condition, then_expr, else_expr } => {
            collect_variables_inner(condition, variables);
            collect_variables_inner(then_expr, variables);
            collect_variables_inner(else_expr, variables);
        }
        Expr::Access {
            collection, index, ..
        } => {
            variables.insert(collection.clone());
            collect_variables_inner(index, variables);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_variables_inner(arg, variables);
            }
        }
        Expr::Builtin { args, .. } => {
            for arg in args {
                collect_variables_inner(arg, variables);
            }
        }
        Expr::Pop { array } => {
            variables.insert(array.clone());
        }
        Expr::Exists { hash, key } => {
            variables.insert(hash.clone());
            collect_variables_inner(key, variables);
        }
        Expr::Ref(target) | Expr::RefArray(target) | Expr::RefHash(target) => {
            variables.insert(target.clone());
        }
        Expr::Deref(ref_name) => {
            variables.insert(ref_name.clone());
        }
        Expr::ArrowArrayAccess { ref_var, index } => {
            variables.insert(ref_var.clone());
            collect_variables_inner(index, variables);
        }
        Expr::ArrowHashAccess { ref_var, key } => {
            variables.insert(ref_var.clone());
            collect_variables_inner(key, variables);
        }
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) => {}
    }
}

fn parse_expression(
    function: &str,
    directive: &'static str,
    expression: &str,
) -> std::result::Result<Expr, AnnotationError> {
    parser::parse_expr(expression).map_err(|_| AnnotationError::InvalidExpression {
        function: function.to_string(),
        directive,
        expression: expression.to_string(),
    })
}

/// Parse a `# extern:` annotation line into an `ExternSpec`.
///
/// Format: `# extern: NAME (Type1, Type2) -> RetType pre: EXPR post: EXPR`
/// The `pre:` and `post:` clauses are optional (default to `true`).
pub fn parse_extern_line(line: &str) -> std::result::Result<ExternSpec, AnnotationError> {
    let raw = line
        .trim()
        .strip_prefix(EXTERN_PREFIX)
        .ok_or_else(|| AnnotationError::InvalidExternDeclaration {
            declaration: line.to_string(),
        })?
        .trim();

    // Split off optional `post:` clause first (so we don't confuse it with sig content)
    let (before_post, postcondition) = if let Some(idx) = raw.find(" post:") {
        let post_expr = raw[idx + " post:".len()..].trim();
        (&raw[..idx], post_expr)
        } else {
        (raw, "")
    };

    // Split off optional `pre:` clause
    let (before_pre, precondition) = if let Some(idx) = before_post.find(" pre:") {
        let pre_expr = before_post[idx + " pre:".len()..].trim();
        (&before_post[..idx], pre_expr)
    } else {
        (before_post, "")
    };

    // Now `before_pre` should be: NAME (Type1, Type2, ...) -> RetType
    let sig_part = before_pre.trim();

    // Extract function name (first token before '(')
    let paren_start = sig_part.find('(').ok_or_else(|| {
        AnnotationError::InvalidExternDeclaration {
            declaration: line.to_string(),
        }
    })?;
    let name = sig_part[..paren_start].trim().to_string();
    if name.is_empty() {
        return Err(AnnotationError::InvalidExternDeclaration {
            declaration: line.to_string(),
        });
    }

    // Parse signature: (Type1, Type2, ...) -> RetType
    let sig_remainder = &sig_part[paren_start..];
    let (raw_args, raw_ret) =
        sig_remainder
            .split_once("->")
            .ok_or_else(|| AnnotationError::InvalidExternDeclaration {
                declaration: line.to_string(),
            })?;

    let args = raw_args.trim();
    if !args.starts_with('(') || !args.ends_with(')') {
        return Err(AnnotationError::InvalidExternDeclaration {
            declaration: line.to_string(),
        });
    }

    let param_types = split_signature_types(&args[1..args.len() - 1])
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .map(|part| {
            parse_type(&name, &part).map_err(|_| AnnotationError::UnsupportedExternType {
                name: name.clone(),
                type_name: part,
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let return_type =
        parse_type(&name, raw_ret.trim()).map_err(|_| AnnotationError::UnsupportedExternType {
            name: name.clone(),
            type_name: raw_ret.trim().to_string(),
        })?;

    let pre = if precondition.is_empty() {
        Expr::Bool(true)
    } else {
        parser::parse_expr(precondition).map_err(|_| AnnotationError::InvalidExternExpression {
            name: name.clone(),
            expression: precondition.to_string(),
        })?
    };

    let post = if postcondition.is_empty() {
        Expr::Bool(true)
    } else {
        parser::parse_expr(postcondition).map_err(|_| AnnotationError::InvalidExternExpression {
            name: name.clone(),
            expression: postcondition.to_string(),
        })?
    };

    debug!(name = name, param_count = param_types.len(), "parsed extern declaration");

    Ok(ExternSpec {
        name,
        param_types,
        return_type,
        precondition: pre,
        postcondition: post,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{BinaryOp, Builtin, Expr, Type},
        extractor::ExtractedFunction,
    };

    use super::{AnnotationError, parse_extern_line, parse_function_spec};

    #[test]
    fn parses_extern_with_postcondition_only() {
        let ext = parse_extern_line("# extern: abs_val (Int) -> Int post: $result >= 0").unwrap();
        assert_eq!(ext.name, "abs_val");
        assert_eq!(ext.param_types, vec![Type::Int]);
        assert_eq!(ext.return_type, Type::Int);
        assert_eq!(ext.precondition, Expr::Bool(true));
        assert_eq!(
            ext.postcondition,
            Expr::Binary {
                left: Box::new(Expr::Variable("result".to_string())),
                op: BinaryOp::Ge,
                right: Box::new(Expr::Int(0)),
            }
        );
    }

    #[test]
    fn parses_extern_with_pre_and_post() {
        let ext = parse_extern_line(
            "# extern: clamp (Int, Int, Int) -> Int pre: $b <= $c post: $result >= $b && $result <= $c",
        )
        .unwrap();
        assert_eq!(ext.name, "clamp");
        assert_eq!(ext.param_types, vec![Type::Int, Type::Int, Type::Int]);
        assert_eq!(ext.return_type, Type::Int);
        // precondition: $b <= $c
        assert!(ext.precondition != Expr::Bool(true));
        // postcondition: $result >= $b && $result <= $c
        assert!(ext.postcondition != Expr::Bool(true));
    }

    #[test]
    fn parses_extern_with_no_conditions() {
        let ext = parse_extern_line("# extern: noop (Int) -> Int").unwrap();
        assert_eq!(ext.name, "noop");
        assert_eq!(ext.param_types, vec![Type::Int]);
        assert_eq!(ext.return_type, Type::Int);
        assert_eq!(ext.precondition, Expr::Bool(true));
        assert_eq!(ext.postcondition, Expr::Bool(true));
    }

    #[test]
    fn rejects_extern_with_missing_arrow() {
        let err = parse_extern_line("# extern: bad (Int) Int").unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidExternDeclaration { .. }));
    }

    #[test]
    fn parses_valid_function_spec() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int, Int) -> Int".to_string(),
                "# pre: $x > 0 && $y > 0".to_string(),
                "# pos: $result > $x + $y * 2".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    return $x + $y;\n".to_string(),
            start_line: 4,
        };

        let spec = parse_function_spec(&function).unwrap();

        assert_eq!(spec.name, "foo");
        assert_eq!(spec.arg_types, vec![Type::Int, Type::Int]);
        assert_eq!(spec.ret_type, Type::Int);
        assert_eq!(
            spec.post,
            Expr::Binary {
                left: Box::new(Expr::Variable("result".to_string())),
                op: BinaryOp::Gt,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Variable("y".to_string())),
                        op: BinaryOp::Mul,
                        right: Box::new(Expr::Int(2)),
                    }),
                }),
            }
        );
    }

    #[test]
    fn parses_string_signature_and_postcondition() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Str) -> Str".to_string(),
                "# post: length($result) == length($x)".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();

        assert_eq!(spec.arg_types, vec![Type::Str]);
        assert_eq!(spec.ret_type, Type::Str);
        assert_eq!(
            spec.post,
            Expr::Binary {
                left: Box::new(Expr::Builtin {
                    function: Builtin::Length,
                    args: vec![Expr::Variable("result".to_string())],
                }),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Builtin {
                    function: Builtin::Length,
                    args: vec![Expr::Variable("x".to_string())],
                }),
            }
        );
    }

    #[test]
    fn rejects_invalid_signature_format() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: Int, Int -> Int".to_string(),
                "# post: $result > 0".to_string(),
            ],
            body: "\n    my ($x, $y) = @_;\n    return $x + $y;\n".to_string(),
            start_line: 1,
        };

        let error = parse_function_spec(&function).unwrap_err();

        assert_eq!(
            error,
            AnnotationError::InvalidSignature {
                function: "foo".to_string(),
                signature: "Int, Int -> Int".to_string(),
            }
        );
    }

    #[test]
    fn rejects_missing_variables() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# pre: $z > 0".to_string(),
                "# post: $result > $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let error = parse_function_spec(&function).unwrap_err();

        assert_eq!(
            error,
            AnnotationError::MissingVariable {
                function: "foo".to_string(),
                variable: "z".to_string(),
            }
        );
    }

    #[test]
    fn defaults_missing_precondition_to_true() {
        let function = ExtractedFunction {
            name: "foo".to_string(),
            annotations: vec![
                "# sig: (Int) -> Int".to_string(),
                "# post: $result >= $x".to_string(),
            ],
            body: "\n    my ($x) = @_;\n    return $x;\n".to_string(),
            start_line: 1,
        };

        let spec = parse_function_spec(&function).unwrap();

        assert_eq!(spec.pre, Expr::Bool(true));
    }
}
