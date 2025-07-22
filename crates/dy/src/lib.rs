use std::fmt::Display;

use colored::Colorize;
use error::ParseError;
use lsp_types::{Position, Range};
use parser::tokenize_into_lines;
use semantic::{Block, build_blocks_tree};
use serde::Serialize;
use spec::ValidDYSpec;

pub mod error;
pub mod parser;
pub mod semantic;
pub mod spec;

mod common;

// DY files must be stored inside something.dy
pub const FILE_EXTENSION: &str = "dy";

/// The result of a parsing, with the vector of items and potentially some errors
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ParseResult<T> {
    pub items: Vec<T>,
    pub errors: Vec<ParseError>,
    /// If provided during parse_with_spec, the file path will be provided, for better error display
    pub some_file_path: Option<String>,
    /// If the `errors` vec is not empty, it will also includes the file content so the error can be displayed
    pub some_file_content: Option<String>,
}

impl<T> Display for ParseResult<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            write!(
                f,
                "{}",
                format!(
                    "Found {} items {}with no error!",
                    self.items.len(),
                    self.some_file_path
                        .as_ref()
                        .map(|path| format!("in {path} "))
                        .unwrap_or_default()
                )
                .green()
            )
        } else {
            let _ = write!(
                f,
                "{}",
                format!(
                    "Found {} item{} {}with {} error{}.\n",
                    self.items.len(),
                    if self.items.len() > 1 { "s" } else { "" },
                    self.some_file_path
                        .as_ref()
                        .map(|path| format!("in {path} "))
                        .unwrap_or_default(),
                    self.errors.len(),
                    if self.errors.len() > 1 { "s" } else { "" },
                )
                .red()
            );

            for error in self.errors.iter() {
                let range = error.range;
                let position = match &self.some_file_path {
                    Some(file) => format!("{file}:{}:{}", range.start.line, range.start.character),
                    None => format!("line {}, char {}", range.start.line, range.start.character),
                };
                let _ = write!(f, "{}", format!("\nError at {position}\n").cyan().bold());

                let context_line = match &self.some_file_content {
                    Some(content) => content.lines().collect::<Vec<_>>()
                        [error.range.start.line as usize..range.end.line as usize + 1]
                        .join("\n"),
                    None => String::default(),
                };
                let _ = writeln!(f, "{context_line}");
                let underlined_chars_count = range.end.character - range.start.character;
                let shifter = range.start.character;
                let repeated_markers = if underlined_chars_count == 0 {
                    "|"
                } else {
                    &"^".repeat(underlined_chars_count as usize)
                };
                let _ = write!(
                    f,
                    "{}{}",
                    " ".repeat(shifter as usize),
                    repeated_markers.red()
                );
                let _ = writeln!(f, "{}", format!(" {}", error.error).red().bold());
            }
            Ok(())
        }
    }
}

/// Make sure we can create this type from a Block and validate it's content once created
pub trait FromDYBlock<'a> {
    /// Get a block representing the same object as Self but in a blocks tree
    /// Subblocks must be taken into account, there is call of this method for them
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, Self);
}

/// Given a ValidDYSpec and a content, generate a ParseResult with all the items of type T that
/// have been extracted. This T needs to implement the mapping from a given Block and validation
/// after the mapping, via the FromDYBlock trait.
/// The ParseResult.some_file_content is filled with an owned copy of the content only if they are some errors
pub fn parse_with_spec<'a, T>(
    spec: &'a ValidDYSpec,
    some_file: &Option<String>,
    content: &'a str,
) -> ParseResult<T>
where
    T: FromDYBlock<'a>,
{
    let lines = tokenize_into_lines(spec, content);
    let (blocks, mut errors) = build_blocks_tree(spec, lines);

    let mut items: Vec<T> = Vec::with_capacity(blocks.len());

    // Note: we only call from_block_with_validation() on the block at level 0
    // because they are the only one we know they implement this method, due to the T: FromDYBlock
    // Subblocks are managed by this method
    for block in blocks {
        let (new_errors, entity) = T::from_block_with_validation(&block);
        errors.extend(new_errors);
        items.push(entity);
    }

    let some_file_content = if errors.is_empty() {
        None
    } else {
        Some(content.to_string())
    };

    ParseResult {
        items,
        some_file_path: some_file.clone(),
        errors,
        some_file_content,
    }
}

// Helpers functions

/// Util function to create a new range on a single line, at given line index, from position 0 to given length
pub fn range_on_line_with_length(line: u32, length: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: length,
        },
    }
}
/// Util function to create a new range on given line indexes from start of first line to to given length in last line
pub fn range_on_lines(line: u32, line2: u32, length: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line: line2,
            character: length,
        },
    }
}

pub fn range_on_line_part(line: u32, start: u32, end: u32) -> Range {
    Range {
        start: Position {
            line,
            character: start,
        },
        end: Position {
            line,
            character: end,
        },
    }
}
