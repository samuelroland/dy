use error::ParseError;
use lsp_types::{Position, Range};
use parser::tokenize_into_lines;
use semantic::{Block, build_blocks_tree};
use spec::ValidDYSpec;

pub mod error;
pub mod parser;
pub mod semantic;
pub mod spec;

mod common;

/// The result of a parsing, with the vector of items and potentially some errors
#[derive(Debug, PartialEq, Eq)]
pub struct ParseResult<T> {
    pub items: Vec<T>,
    pub errors: Vec<ParseError>,
}

/// Make sure we can create this type from a Block and validate it's content once created
pub trait FromDYBlock<'a> {
    fn from_block(block: &Block<'a>) -> Self;
    fn validate(&self) -> Vec<ParseError>;
}

/// Given a ValidDYSpec and a content, generate a ParseResult with all the items of type T that
/// have been extracted. This T needs to implement the mapping from a given Block and validation
/// after the mapping, via the FromDYBlock trait.
pub fn parse_with_spec<'a, T>(spec: &'a ValidDYSpec, content: &'a str) -> ParseResult<T>
where
    T: FromDYBlock<'a>,
{
    let lines = tokenize_into_lines(spec, content);
    let (blocks, mut errors) = build_blocks_tree(spec, lines);

    let mut items: Vec<T> = Vec::with_capacity(blocks.len());

    for block in blocks {
        let entity = T::from_block(&block);
        errors.extend(entity.validate());
        items.push(entity);
    }

    ParseResult { items, errors }
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
