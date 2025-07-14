/// The semantic analyzer is responsible for building tree of blocks, building and verifying the hierarchy as the lines
/// starting with a key are found.
use lsp_types::{Position, Range};

use crate::{
    error::Error,
    parser::{Line, LinePart, LineType},
    spec::{DYSpec, KeySpec, ValidDYSpec},
};

#[derive(Debug, PartialEq)]
struct Block<'a> {
    key: &'a KeySpec<'a>,
    /// The text contained in the value of this block, when multiline it can contains several &str
    /// This doesn't contain the key
    text: Vec<&'a str>,
    /// The full range of all lines used to describe this block, including subblocks
    range: Range,
    /// The sub blocks
    subblocks: Vec<Block<'a>>,
}

/// The recursive style search to find if the current key is valid consists of the following algorightm
/// This is starting at the root spec, we try to find if the found key is part of the spec at root
/// if yes, we go a lever deeper into the spec of the found key, because there is a higher chance of finding the next key in subkeys for the next line
/// We continue with the next lines. If we don't find anything in this level, we go in level - 1 if
/// possible. When we reached the root again, we stop and report the error.
///
/// EXAMPLE with this spec tree
/// exo
/// - check
///   - args
///   - see
///   - type
///
/// and these exo lines:
///
/// exo hey there
/// check yes
/// see good
/// exo yop
/// see not good
///
/// first line: testing "exo" -> found immediately
/// second line: testing "check" -> found immediately
/// third line: testing "args", then "see" -> ok
/// fourth line: testing "args", "see", "type", level--, testing "check", level--, testing "exo" -> ok
/// fifth line: testing "check", level--, testing "exo", cannot level-- so report the error!
///
/// TODO: clean up this algo
///
/// ALGORITHM OVERVIEW
/// parents_spec_by_depth = [] as an stack of [&KeySpec], at each depth we store the parent spec to
/// be able to continue with other keys on the same levels by looking at the subkeys of the parent
/// current_spec = &root_spec // the given spec
/// for line in lines
///     if the line is a key
///         loop
///             if this current_spec contains this key
///                 all good, the key is valid, create a new_block for it without subblocks
///                 if key has subkeys
///                     parents_blocks push new_block
///                     parents_spec_by_depth.push(current_spec)
///                     current_spec = key.subkeys
///                 else
///                     blocks last() push new_block // as t
///             else if parents_spec_by_depth is not empty // not found at this level, going up to see in childrens of parents
///                 let merged = parents_blocks pop()
///                 parents_blocks last() push merge // no more child blocks to find in merged
///                 current_spec = parents_spec_by_depth pop back
///             else break // if there is no parents_spec_by_depth, we reached the top of the spec tree, the key is not valid
///
///     else if the line is a comment, just ignore
///     else if the line is a unknown
///         if there is an existing block, append the text into int
///         else that's an error, report it
///
fn build_blocks_tree<'a>(lines: Vec<Line<'a>>, spec: &ValidDYSpec) -> Vec<Block<'a>> {
    let errors: Vec<Error> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut current_spec = spec.get();
    let mut parents_spec: Vec<&DYSpec> = Vec::new();
    let mut parents_blocks: Vec<Block> = Vec::new();
    for line in lines {
        match line.lt {
            LineType::WithKey(key_spec) => {
                loop {
                    if let Some(found) = current_spec.iter().find(|s| s.id == key_spec.id) {
                        let parts = line.tokenize_parts();
                        let text = parts
                            .iter()
                            .filter_map(|f| {
                                if let LinePart::Value(a) = f {
                                    Some(*a)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let new_block = Block {
                            key: key_spec,
                            text,
                            range: Range::new(
                                Position::new(line.index as u32, 0),
                                Position::new(line.index as u32, line.slice.len() as u32),
                            ),
                            subblocks: vec![],
                        };

                        if !key_spec.subkeys.is_empty() {
                            parents_blocks.push(new_block);
                            parents_spec.push(current_spec);
                            current_spec = key_spec.subkeys;
                        } else {
                            if let Some(existing_block) = &mut parent_block {
                                existing_block.subblocks.push(new_block);
                            }
                        }
                    }

                    break;
                    // If we reach this point, the key_spec.id is not a valid key at this level !
                    // TODO store the error
                }
            }
            LineType::Comment => continue,
            LineType::Unknown => {
                if let Some(existing_block) = &mut parent_block {
                    existing_block.text.push(line.slice);
                    existing_block.range.end.line = line.index as u32;
                } else {
                    // todo manage errors
                }
            }
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use crate::{
        common::tests::PLX_COURSE_SPEC, parser::tokenize_into_lines, semantic::build_blocks_tree,
        spec::ValidDYSpec,
    };
    use pretty_assertions::assert_eq;

    #[test]
    #[ntest::timeout(50)]
    // All empty lines are considered Unknown for the tokenizer
    fn test_can_build_blocks_for_course() {
        let text = "course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        let spec = &ValidDYSpec::new(PLX_COURSE_SPEC).unwrap();
        let lines = tokenize_into_lines(spec, text);
        let blocks = build_blocks_tree(lines, spec);

        assert_eq!(blocks, vec![]);
    }
}
