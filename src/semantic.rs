use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::Peekable;

/// The semantic analyzer is responsible for building tree of blocks, building and verifying the hierarchy as the lines
/// starting with a key are found.
use lsp_types::{Position, Range};

use crate::{
    error::{ParseError, ParseErrorType},
    parser::{Line, LinePart, LineType},
    spec::{DYSpec, KeySpec, ValidDYSpec},
};

#[derive(PartialEq)]
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

// Implement Debug so we can have a shorter display of Range
impl<'a> Debug for Block<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NiceRange<'a>(&'a Range);
        impl<'a> Debug for NiceRange<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}:{}-{}:{}",
                    &self.0.start.line,
                    &self.0.start.character,
                    &self.0.end.line,
                    &self.0.end.character,
                )
            }
        }
        f.debug_struct("Block")
            .field("key", &self.key)
            .field("text", &self.text)
            .field("range", &NiceRange(&self.range))
            .field("subblocks", &self.subblocks)
            .finish()
    }
}

/// Given a flat list of Line, build a blocks tree, with a tree's hierarchy respecting the given tree spec. Return possible hierarchy errors.
/// This group Unknown content after a multiline prefix in a single block of the associated key
/// On each line WithKey we try to determine whether the key is valid at this position
///
/// Instead of a recursive approach, we use non-recursive algorithm that is going deeper in the spec tree
/// if the key is valid, and upper in the spec tree otherwise
///
/// TODO REFACTOR THIS COMMENT
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
/// check okay
///
/// first line: testing "exo" -> found immediately
/// second line: testing "check" -> found immediately
/// third line: testing "args", then "see" -> ok
/// fourth line: testing "args", "see", "type", level--, testing "check", level--, testing "exo" -> ok
/// fifth line: testing "check", level--, testing "exo", cannot level-- so report the error!
/// sixth line: testing with the sub spec of the last valid keys (at start it is the root spec, then the spec.subkeys of the last valid key)
///
/// TODO: clean up this algo
///
/// ALGORITHM OVERVIEW
/// parents_spec = [] as an stack of [&KeySpec], at each depth we store the parent spec to
/// be able to continue with other keys on the same levels by looking at the subkeys of the parent
/// current_spec = &root_spec // the given spec
/// for line in lines
///     if the line is a key
///         loop
///             if this current_spec contains this key
///                 all good, the key is valid, create a new_block for it without subblocks
///                 if key has subkeys
///                     parents_blocks push new_block
///                     parents_spec.push(current_spec)
///                     current_spec = key.subkeys
///                 else
///                     blocks last() push new_block // as there will be no children block, we can push it
///             else if parents_spec_by_depth is not empty // not found at this level, going up to see in childrens of parents
///                 let merged = parents_blocks pop()
///                 parents_blocks last() push merge // no more child blocks to find in merged
///                 current_spec = parents_spec pop back
///             else break // if there is no element in parents_spec, we reached the top of the spec tree, the key is not valid
///
///     else if the line is a comment, just ignore
///     else if the line is a unknown
///         if there is an existing block, append the text into int
///         else that's an error, report it
///
fn build_blocks_tree<'a>(
    lines: Vec<Line<'a>>,
    spec: &ValidDYSpec,
) -> (Vec<Block<'a>>, Vec<ParseError>) {
    build_blocks_subtree_recursive(&mut lines.iter().peekable(), spec.get(), 0, None)
}

/// Recursive function to build a subtree of blocks
fn build_blocks_subtree_recursive<'a>(
    lines: &mut Peekable<std::slice::Iter<'_, Line<'a>>>,
    specs: &DYSpec,
    level: u8,
    parent_spec: Option<&KeySpec>,
) -> (Vec<Block<'a>>, Vec<ParseError>) {
    let mut errors: Vec<ParseError> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();

    // TODO: change this to a normal vec with an index access, to improve performance
    let mut once_keys_found: HashSet<&str> = HashSet::new();

    while let Some(line) = lines.peek() {
        match line.lt {
            LineType::WithKey(associated_spec) => {
                // eprintln!("Checking line WithKey: {}", line.slice);
                eprintln!(
                        "Checking if {associated_spec:?} is present inside specs list {specs:?} with level {level}"
                    );
                if specs.iter().any(|s| s.id == associated_spec.id) {
                    // Make sure keys with once=true are not inserted more than once !
                    if associated_spec.once {
                        let already_inserted = !once_keys_found.insert(associated_spec.id);
                        if already_inserted {
                            errors.push(ParseError {
                                range: range_on_line_with_length(
                                    line.index as u32,
                                    associated_spec.id.len() as u32,
                                ),
                                some_file: None,
                                error: ParseErrorType::DuplicatedKey(
                                    associated_spec.id.to_string(),
                                    level,
                                ),
                            });
                            lines.next();
                            break;
                        }
                    }

                    // Build the new block as it is valid
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
                    let mut new_block = Block {
                        key: associated_spec,
                        text,
                        range: Range::new(
                            Position::new(line.index as u32, 0),
                            Position::new(line.index as u32, line.slice.len() as u32),
                        ),
                        subblocks: vec![],
                    };

                    println!("New block: {new_block:?}");

                    // The line was valid, we can move to the next line
                    lines.next();

                    // If there are subkeys and the next li
                    if !associated_spec.subkeys.is_empty()
                        && matches!(
                            lines.peek(),
                            Some(Line {
                                lt: LineType::WithKey(_),
                                ..
                            })
                        )
                    {
                        eprintln!("Recursive call to level {}", level + 1);
                        let (subblocks, suberrors) = build_blocks_subtree_recursive(
                            lines,
                            associated_spec.subkeys,
                            level + 1,
                            Some(associated_spec),
                        );
                        dbg!(&subblocks, &suberrors);
                        new_block.subblocks = subblocks;
                        errors.extend(suberrors);
                        // if !new_block.subblocks.is_empty() {
                        //     blocks.push(new_block); // we can save the block at this point
                        //     current_new_block = None; // no more Unknown lines can be added to block.text after children blocks have been found
                        //                               // block.
                        // }
                    } else {
                        // current_new_block = Some(new_block);
                    }
                    blocks.push(new_block); // TODO fix
                } else if level == 0 {
                    eprintln!("Found WrongKeyPosition !");
                    errors.push(ParseError {
                        range: range_on_line_with_length(
                            line.index as u32,
                            associated_spec.id.len() as u32,
                        ),
                        some_file: None,
                        error: ParseErrorType::WrongKeyPosition(
                            associated_spec.id.to_string(),
                            "??".to_string(), // how to get the parent ??
                        ),
                    });
                    lines.next();
                } else {
                    eprintln!("No found at this level, going up");
                    break;
                }
                // break;
                // If we reach this point, the key_spec.id is not a valid key at this level !
                // TODO store the error
            }
            LineType::Comment => continue,
            LineType::Unknown => {
                // if let Some(existing_block) = parents_blocks.last_mut() {
                //     existing_block.text.push(line.slice);
                //     existing_block.range.end.line = line.index as u32;
                // } else {
                //     // todo manage errors
                // }
            }
        }
    }
    (blocks, errors)
}

/// Util function to create a new range on a single line, at given line indes, from position 0 to given length
fn range_on_line_with_length(line: u32, length: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: length,
        },
    }
}

#[cfg(test)]
mod tests {

    use crate::common::tests::{SKILL_SPEC, SUBSKILL_SPEC, TESTING_SKILLS_SPEC};
    use crate::error::{ParseError, ParseErrorType};
    use crate::semantic::range_on_line_with_length;
    use crate::{
        common::tests::{CODE_SPEC, COURSE_SPEC, GOAL_SPEC, TESTING_COURSE_SPEC},
        parser::tokenize_into_lines,
        semantic::{build_blocks_tree, Block},
        spec::ValidDYSpec,
    };
    use pretty_assertions::assert_eq;

    fn get_blocks<'a>(
        spec: &'a ValidDYSpec,
        text: &'a str,
    ) -> (std::vec::Vec<Block<'a>>, std::vec::Vec<ParseError>) {
        let lines = tokenize_into_lines(spec, text);
        build_blocks_tree(lines, spec)
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_build_blocks_for_simple_course() {
        let text = "course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        let spec = &ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let lines = tokenize_into_lines(spec, text);
        let (blocks, errors) = build_blocks_tree(lines, spec);

        assert_eq!(
            blocks,
            vec![Block {
                key: COURSE_SPEC,
                text: vec!["Programmation 1",],
                range: range_on_line_with_length(0, 22),
                subblocks: vec![
                    Block {
                        key: CODE_SPEC,
                        text: vec!["PRG1",],
                        range: range_on_line_with_length(1, 9),
                        subblocks: vec![],
                    },
                    Block {
                        key: GOAL_SPEC,
                        text: vec!["Apprendre des bases solides du C++",],
                        range: range_on_line_with_length(2, 39),
                        subblocks: vec![],
                    },
                ],
            }]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_build_blocks_for_complex_skills() {
        let text = "skill A
subskill B
skill C
skill D
subskill E";
        let binding = ValidDYSpec::new(TESTING_SKILLS_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            blocks,
            vec![
                Block {
                    key: SKILL_SPEC,
                    text: vec!["A",],
                    range: range_on_line_with_length(0, 7),
                    subblocks: vec![Block {
                        key: SUBSKILL_SPEC,
                        text: vec!["B",],
                        range: range_on_line_with_length(1, 10),
                        subblocks: vec![],
                    },],
                },
                Block {
                    key: SKILL_SPEC,
                    text: vec!["C",],
                    range: range_on_line_with_length(2, 7),
                    subblocks: vec![],
                },
                Block {
                    key: SKILL_SPEC,
                    text: vec!["D",],
                    range: range_on_line_with_length(3, 7),
                    subblocks: vec![Block {
                        key: SUBSKILL_SPEC,
                        text: vec!["E",],
                        range: range_on_line_with_length(4, 10),
                        subblocks: vec![],
                    },],
                }
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_detect_wrong_key_positions() {
        let text = "goal learn c++
course Programmation 1
code hey";
        let (_, errors) = get_blocks(&ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap(), text);

        assert_eq!(
            errors,
            vec![ParseError {
                range: range_on_line_with_length(0, 4),
                some_file: None,
                error: ParseErrorType::WrongKeyPosition("goal".to_string(), "??".to_string()) // "course".to_string())
            }]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_detect_duplicated_key_error() {
        let text = "course Programmation 1
course oups";
        let binding = ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            blocks,
            vec![Block {
                key: COURSE_SPEC,
                text: vec!["Programmation 1",],
                range: range_on_line_with_length(0, 22),
                subblocks: vec![],
            }]
        );
        assert_eq!(
            errors,
            vec![ParseError {
                range: range_on_line_with_length(1, 6),
                some_file: None,
                error: ParseErrorType::DuplicatedKey("course".to_string(), 0)
            }]
        );
    }
}
