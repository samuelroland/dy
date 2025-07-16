use error::ParseError;
use parser::tokenize_into_lines;
use semantic::{build_blocks_tree, Block};
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
pub trait FromDYBlock<'a>
where
    Self: Default,
{
    fn from_block(block: &Block<'a>) -> Self;
    fn validate(&self) -> Vec<ParseError>;
}

/// Given a ValidDYSpec and a content, generate a ParseResult with all the items of type T that
/// have been extracted. This T needs to implement the mapping from a given Block and validation
/// after the mapping, via the FromDYBlock trait.
pub fn parse_with_spec<'a, T>(spec: &'a ValidDYSpec, content: &'a str) -> ParseResult<T>
where
    T: Default + FromDYBlock<'a>,
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
