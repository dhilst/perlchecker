use crate::annotations::SIG_PREFIX;
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFunction {
    pub name: String,
    pub annotations: Vec<String>,
    pub body: String,
    pub start_line: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExtractorError {
    #[error("annotation block at line {line} must be followed immediately by `sub NAME {{`")]
    MissingSub { line: usize },
    #[error("invalid sub declaration at line {line}: {header}")]
    InvalidSubHeader { line: usize, header: String },
    #[error("function `{function}` starting at line {line} has unmatched braces")]
    UnmatchedBraces { function: String, line: usize },
}

pub fn extract_annotated_functions(source: &str) -> Result<Vec<ExtractedFunction>, ExtractorError> {
    let lines: Vec<&str> = source.lines().collect();
    let line_offsets = line_start_offsets(source);
    let mut extracted = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if !lines[index].trim_start().starts_with(SIG_PREFIX) {
            index += 1;
            continue;
        }

        let annotation_start = index;
        let mut annotations = Vec::new();
        while index < lines.len() && lines[index].trim_start().starts_with('#') {
            annotations.push(lines[index].to_string());
            index += 1;
        }

        if index >= lines.len() {
            return Err(ExtractorError::MissingSub {
                line: annotation_start + 1,
            });
        }

        let header = lines[index];
        let function_name =
            parse_sub_header(header).ok_or_else(|| ExtractorError::InvalidSubHeader {
                line: index + 1,
                header: header.to_string(),
            })?;

        let open_brace_byte = line_offsets[index]
            + header
                .find('{')
                .expect("validated sub header must contain an opening brace");
        let close_brace_byte = find_matching_brace(source, open_brace_byte).ok_or_else(|| {
            ExtractorError::UnmatchedBraces {
                function: function_name.clone(),
                line: index + 1,
            }
        })?;

        let body = source[open_brace_byte + 1..close_brace_byte].to_string();
        let end_line = line_index_for_byte(source, close_brace_byte);

        debug!(
            name = function_name,
            start_line = index + 1,
            end_line = end_line + 1,
            "extracted annotated function"
        );

        extracted.push(ExtractedFunction {
            name: function_name,
            annotations,
            body,
            start_line: index + 1,
        });

        index = end_line + 1;
    }

    Ok(extracted)
}

fn parse_sub_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("sub ")?;
    let (name, trailing) = remainder.split_once('{')?;

    if !trailing.trim().is_empty() {
        return None;
    }

    let name = name.trim();
    if name.is_empty() || name.split_whitespace().count() != 1 {
        return None;
    }

    Some(name.to_string())
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut offset = 0;

    for line in source.lines() {
        offsets.push(offset);
        offset += line.len() + 1;
    }

    offsets
}

fn find_matching_brace(source: &str, open_brace_byte: usize) -> Option<usize> {
    let mut depth = 0usize;

    for (offset, ch) in source[open_brace_byte..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_brace_byte + offset);
                }
            }
            _ => {}
        }
    }

    None
}

fn line_index_for_byte(source: &str, byte_index: usize) -> usize {
    source[..byte_index]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
}

#[cfg(test)]
mod tests {
    use super::{ExtractedFunction, ExtractorError, extract_annotated_functions};
    use proptest::prelude::*;

    #[test]
    fn extracts_multiple_annotated_functions() {
        let source = r#"
sub plain {
    return 0;
}

# sig: (Int) -> Int
# post: $result >= $x
sub foo {
    my ($x) = @_;
    if ($x > 0) {
        return $x;
    }

    return 0;
}

# sig: (Int, Int) -> Int
# post: $result >= $x
sub bar {
    my ($x, $y) = @_;
    return $x + $y;
}
"#;

        let extracted = extract_annotated_functions(source).unwrap();

        assert_eq!(
            extracted,
            vec![
                ExtractedFunction {
                    name: "foo".to_string(),
                    annotations: vec![
                        "# sig: (Int) -> Int".to_string(),
                        "# post: $result >= $x".to_string(),
                    ],
                    body: "\n    my ($x) = @_;\n    if ($x > 0) {\n        return $x;\n    }\n\n    return 0;\n".to_string(),
                    start_line: 8,
                },
                ExtractedFunction {
                    name: "bar".to_string(),
                    annotations: vec![
                        "# sig: (Int, Int) -> Int".to_string(),
                        "# post: $result >= $x".to_string(),
                    ],
                    body: "\n    my ($x, $y) = @_;\n    return $x + $y;\n".to_string(),
                    start_line: 19,
                },
            ]
        );
    }

    #[test]
    fn rejects_annotation_block_without_immediate_sub() {
        let source = r#"
# sig: (Int) -> Int
# post: $result > 0

sub foo {
    return 1;
}
"#;

        let error = extract_annotated_functions(source).unwrap_err();

        assert_eq!(
            error,
            ExtractorError::InvalidSubHeader {
                line: 4,
                header: "".to_string()
            }
        );
    }

    #[test]
    fn rejects_unmatched_braces() {
        let source = r#"
# sig: (Int) -> Int
# post: $result >= $x
sub foo {
    if ($x > 0) {
        return $x;
}
"#;

        let error = extract_annotated_functions(source).unwrap_err();

        assert_eq!(
            error,
            ExtractorError::UnmatchedBraces {
                function: "foo".to_string(),
                line: 4,
            }
        );
    }

    proptest! {
        #[test]
        fn counts_generated_annotated_functions(count in 0usize..20) {
            let source = (0..count)
                .map(|index| {
                    format!(
                        "# sig: (Int) -> Int\n# post: $result >= $x\nsub f{index} {{\n    my ($x) = @_;\n    return $x;\n}}\n"
                    )
                })
                .collect::<String>();

            let extracted = extract_annotated_functions(&source).unwrap();

            prop_assert_eq!(extracted.len(), count);
        }
    }
}
