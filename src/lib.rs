#![forbid(unsafe_code)]

pub mod annotations;
pub mod ast;
pub mod cli;
pub mod extractor;
pub mod ir;
pub mod limits;
pub mod parser;
pub mod smt;
pub mod symexec;

use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, PerlcheckerError>;

#[derive(Debug, Error)]
pub enum PerlcheckerError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("verification failed")]
    VerificationFailed,
    #[error(transparent)]
    Annotation(#[from] annotations::AnnotationError),
    #[error(transparent)]
    TypeCheck(#[from] ast::TypeCheckError),
    #[error(transparent)]
    Extraction(#[from] extractor::ExtractorError),
    #[error(transparent)]
    Ir(#[from] ir::IrError),
    #[error(transparent)]
    Parse(#[from] parser::ParseError),
    #[error(transparent)]
    Smt(#[from] smt::SmtError),
    #[error(transparent)]
    SymExec(#[from] symexec::SymExecError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageSubset {
    pub supported_types: &'static [&'static str],
    pub supported_control_flow: &'static [&'static str],
    pub supported_expressions: &'static [&'static str],
    pub forbidden_features: &'static [&'static str],
}

pub const V1_LANGUAGE_SUBSET: LanguageSubset = LanguageSubset {
    supported_types: &["Int", "Str"],
    supported_control_flow: &["if", "return", "assignment"],
    supported_expressions: &[
        "+", "-", "*", "/", ".", "<", "<=", ">", ">=", "==", "!=", "eq", "ne", "&&", "||", "!",
        "length", "substr", "index",
    ],
    forbidden_features: &[
        "loops",
        "arrays",
        "hashes",
        "function calls",
        "globals",
        "implicit variables ($_)",
        "regex",
        "implicit type coercions",
    ],
};

#[cfg(test)]
mod tests {
    use super::V1_LANGUAGE_SUBSET;

    #[test]
    fn phase_zero_locks_the_v1_subset() {
        assert_eq!(V1_LANGUAGE_SUBSET.supported_types, ["Int", "Str"]);
        assert!(V1_LANGUAGE_SUBSET.supported_control_flow.contains(&"if"));
        assert!(V1_LANGUAGE_SUBSET.supported_expressions.contains(&"length"));
        assert!(V1_LANGUAGE_SUBSET.forbidden_features.contains(&"regex"));
    }
}
