use std::fmt::Display;

use lsp_types::Range;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub enum ParseErrorType {
    #[error("The '{0}' key can be only used under a `{1}`")]
    WrongKeyPosition(String, String),
    #[error("The '{0}' key can only be used once {msg}", msg = if *.1 == 0 {"in document root"} else {"at this level"})]
    DuplicatedKey(String, u8),
}

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub struct ParseError {
    pub range: Range,
    pub some_file: Option<String>,
    pub error: ParseErrorType,
}

// TODO: improve this console display structure ? add colors ? add context of the line and add `^^^^^`
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at {}: {}",
            match &self.some_file {
                Some(file) => format!(
                    "{file}:{}:{}",
                    self.range.start.line, self.range.start.character
                ),
                None => format!(
                    "At line {}, char {}",
                    self.range.start.line, self.range.start.character
                ),
            },
            self.error
        )
    }
}
