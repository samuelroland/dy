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
/// A block represents the instance of a key found in the text, including subblocks for subkeys.
/// A block has a textual value for its key under field `text`
pub struct Block<'a> {
    pub key: &'a KeySpec<'a>,
    /// The text contained in the value of this block, when multiline it can contains several &str
    /// This doesn't contain the key
    pub text: Vec<&'a str>,
    /// The full range of all lines used to describe this block, including subblocks
    pub range: Range,
    /// The sub blocks
    pub subblocks: Vec<Block<'a>>,
}

impl<'a> Block<'a> {
    /// Push a new line of text, with given line and the line index where it was found
    /// The line_index is necessary because comments could be present in the middle of the text
    fn push_text(&mut self, line: &'a str, line_index: usize) {
        self.text.push(line);
        self.range.end.line = line_index as u32;
        self.range.end.character = line.len() as u32;
    }

    /// Get the different recolted lines into a single String, after triming the final text
    pub fn get_joined_text(&self) -> String {
        self.text.join("\n").trim().to_string()
    }

    /// Split joined text with at split the text after `split_after_lines` lines and returns a tuple of both trim results
    pub fn get_text_with_joined_splits_at(&self, split_after_lines: usize) -> (String, String) {
        let (first, second) = self.text.split_at(split_after_lines);
        (
            first.join("\n").trim().to_string(),
            second.join("\n").trim().to_string(),
        )
    }
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
/// It groups Unknown content after a multiline prefix in a single block for the associated key
/// On each line WithKey we try to determine whether the key is valid at this position
///
/// Here is the example of the algo to better understand it before reading the code
/// EXAMPLE with this spec tree
/// - exo
///   - check
///     - args
///     - see
///     - type
///
/// and these exo lines:
///
/// random text
/// exo hey there
/// some content
/// // just a comment
/// see not good because incorrect level
/// check yes
/// args 1
/// see good
/// args duplicated !
/// type good
/// check 2
///
/// line 0: Unknown -> no block created before so it's an ContentOutOfKey error
/// line 1: WithKey(exo) -> testing "exo" -> ok -> new block
/// line 2: Unknown -> append its text to the last created block (for "exo")
/// line 3: Comment -> ignore the comment
/// line 4: WithKey(see) -> recursive call with subkeys spec -> testing "check" KO, level--, testing "exo" KO -> level == 0 so report the WrongKeyPosition error!
/// line 5: WithKey(check) -> recursive call. testing "check" -> found immediately
/// line 6: WithKey(args) -> recursive call with subkeys spec -> testing "args" OK new block
/// line 7: WithKey(see) -> testing "args", testing "see" OK new block
/// line 8: WithKey(args) -> testing "args" OK new block (duplicate key removed later)
/// line 9: WithKey(type) -> testing "args", testing "see", testing "type" OK new block
/// line 10: WithKey(check) -> testing "args", then "see", then "type", not found so remove
/// duplicates in blocks (this creates the DuplicatedKey for "args") and return -> testing "check" OK new block
///
/// See result of structure in test_strange_exo_parsing_can_correctly_ignore_error()
///
/// FIXME: there is a very special edge case of error recovering that is not managed. The above
/// example shows that after the WrongKeyPosition of "see" we continue with "exo", but that's only
/// because that's the root spec, not because that's the right place to start. If had a
/// WrongKeyPosition at level 2 and it will go up at level 0 to generate the error, a key at level
/// 3 right after that will not be correctly extracted...
pub fn build_blocks_tree<'a>(
    spec: &ValidDYSpec,
    lines: Vec<Line<'a>>,
) -> (Vec<Block<'a>>, Vec<ParseError>) {
    let (blocks, mut errors) =
        build_blocks_subtree_recursive(&mut lines.iter().peekable(), spec.get(), 0, None);

    // TODO: that's useful for future errors generated from entities
    // is it still useful or are the errors naturally already sorted ?
    errors.sort();

    (blocks, errors)
}

/// Recursive function to build a subtree of blocks
fn build_blocks_subtree_recursive<'a>(
    lines: &mut Peekable<std::slice::Iter<'_, Line<'a>>>,
    specs: &DYSpec,
    level: u8,
    parent_spec: Option<&KeySpec>,
) -> (Vec<Block<'a>>, Vec<ParseError>) {
    eprintln!(
        "\n>> build_blocks_subtree_recursive: line: {:?}, specs: {specs:?}, level {level}",
        lines.peek()
    );
    let mut errors: Vec<ParseError> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut blocks_starting_line_indexes: Vec<usize> = Vec::new();

    while let Some(line) = lines.peek() {
        match line.lt {
            LineType::WithKey(associated_spec) => {
                eprintln!(
                        "Checking if {associated_spec:?} is present inside specs list {specs:?} with level {level}"
                    );
                if specs.iter().any(|s| s.id == associated_spec.id) {
                    // eprintln!("Found {}", associated_spec.id);
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
                    let new_block = Block {
                        key: associated_spec,
                        text,
                        range: Range::new(
                            Position::new(line.index as u32, 0),
                            Position::new(line.index as u32, line.slice.len() as u32),
                        ),
                        subblocks: vec![],
                    };
                    eprintln!("New block: {new_block:?}");
                    blocks.push(new_block);
                    blocks_starting_line_indexes.push(line.index);

                    // The line was valid, we can move to the next line
                    lines.next();
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
                    eprintln!("No found at this level, going up\n");
                    break; // break the while, so we return from this function
                }
            }
            LineType::Comment => {
                eprintln!("SKipping comment: {}!", line.slice);
                lines.next();
            }
            LineType::Unknown => {
                if let Some(existing_block) = blocks.last_mut() {
                    if matches!(existing_block.key.kt, crate::spec::KeyType::SingleLine) {
                        if !line.slice.trim().is_empty() {
                            eprintln!("Found InvalidMultilineContent on line: {}", line.slice);
                            errors.push(ParseError {
                                range: range_on_line_with_length(
                                    line.index as u32,
                                    line.slice.len() as u32,
                                ),
                                some_file: None,
                                error: ParseErrorType::InvalidMultilineContent(
                                    existing_block.key.id.to_string(),
                                ),
                            });
                        }
                    } else {
                        existing_block.push_text(line.slice, line.index);
                    }
                } else if !line.slice.trim().is_empty() {
                    eprintln!("Found ContentOutOfKey on line: {}", line.slice);
                    // non empty lines without an existing block are ContentOutOfKey
                    errors.push(ParseError {
                        range: range_on_line_with_length(
                            line.index as u32,
                            line.slice.len() as u32,
                        ),
                        some_file: None,
                        error: ParseErrorType::ContentOutOfKey,
                    });
                }
                lines.next();
            }
        }

        // As the line is WithKey, we may need to go check the subkeys
        if matches!(
            lines.peek(),
            Some(Line {
                lt: LineType::WithKey(_),
                ..
            })
        ) {
            // If there is an existing block and it's key spec contains subkeys, we have to go check if they match
            if let Some(existing_block) = blocks.last_mut() {
                if !existing_block.key.subkeys.is_empty() {
                    let (subblocks, suberrors) = build_blocks_subtree_recursive(
                        lines,
                        existing_block.key.subkeys,
                        level + 1,
                        Some(existing_block.key),
                    );
                    errors.extend(suberrors);
                    existing_block.subblocks = subblocks;
                }
            }
        }
    }

    // Once the blocks have been entirely extracted at this level (with possible subkeys)
    // there are ready to be removed in case they are duplicates !
    let mut once_keys_found: HashSet<&str> = HashSet::new(); // TODO: change this to a normal vec with an index access, to improve performance
    let mut non_duplicated_blocks = Vec::with_capacity(blocks.len());
    for (idx, block) in blocks.into_iter().enumerate() {
        // Make sure keys with once=true are not inserted more than once !
        if block.key.once && !once_keys_found.insert(block.key.id) {
            errors.push(ParseError {
                range: range_on_line_with_length(
                    blocks_starting_line_indexes[idx] as u32,
                    block.key.id.len() as u32,
                ),
                some_file: None,
                error: ParseErrorType::DuplicatedKey(block.key.id.to_string(), level),
            });
        } else {
            non_duplicated_blocks.push(block);
        }
    }

    (non_duplicated_blocks, errors)
}

/// Util function to create a new range on a single line, at given line index, from position 0 to given length
fn range_on_line_with_length(line: u32, length: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: length,
        },
    }
}
/// Util function to create a new range on given line indexes from start of first line to to given length in last line
fn range_on_lines(line: u32, line2: u32, length: u32) -> Range {
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line: line2,
            character: length,
        },
    }
}

#[cfg(test)]
mod tests {

    use crate::common::tests::{
        ARGS_SPEC, CHECK_SPEC, EXIT_SPEC, EXO_SPEC, SEE_SPEC, SKILL_SPEC, SUBSKILL_SPEC,
        TESTING_EXOS_SPEC, TESTING_SKILLS_SPEC, TYPE_SPEC,
    };
    use crate::error::{ParseError, ParseErrorType};
    use crate::semantic::{range_on_line_with_length, range_on_lines};
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
        build_blocks_tree(spec, lines)
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_build_blocks_for_simple_course() {
        let text = "course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        let spec = &ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let lines = tokenize_into_lines(spec, text);
        let (blocks, errors) = build_blocks_tree(spec, lines);

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

    #[test]
    #[ntest::timeout(50)]
    fn test_can_build_blocks_with_multiline_keys_ignoring_comments() {
        let text = "// amazing file
// just ignored
skill A
// yop
A desc
// yop in middle
A desc 2
subskill AA
skill B
// just ignored
B desc
// just ignored
subskill C
// just ignored
// just ignored
// just ignored
C desc
C desc 2
// just ignored
// just ignored
// just ignored
";
        let binding = ValidDYSpec::new(TESTING_SKILLS_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            blocks,
            vec![
                Block {
                    key: SKILL_SPEC,
                    text: vec!["A", "A desc", "A desc 2"],
                    range: range_on_lines(2, 6, 8),
                    subblocks: vec![Block {
                        key: SUBSKILL_SPEC,
                        text: vec!["AA",],
                        range: range_on_line_with_length(7, 11),
                        subblocks: vec![],
                    },],
                },
                Block {
                    key: SKILL_SPEC,
                    text: vec!["B", "B desc"],
                    range: range_on_lines(8, 10, 6),
                    subblocks: vec![Block {
                        key: SUBSKILL_SPEC,
                        text: vec!["C", "C desc", "C desc 2",],
                        range: range_on_lines(12, 17, 8),
                        subblocks: vec![],
                    },],
                },
            ]
        );
        assert_eq!(errors, vec![]);
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_detect_invalid_multiline_content() {
        let text = "course Programmation 1
some multiline content oups
code PRG1
goal Apprendre des bases solides du C++";
        let binding = ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            errors,
            vec![ParseError {
                range: range_on_line_with_length(1, 27),
                some_file: None,
                error: ParseErrorType::InvalidMultilineContent("course".to_string())
            }]
        );
        assert_eq!(
            blocks,
            vec![Block {
                key: COURSE_SPEC,
                text: vec!["Programmation 1"],
                range: range_on_line_with_length(0, 22),
                subblocks: vec![
                    Block {
                        key: CODE_SPEC,
                        text: vec!["PRG1"],
                        range: range_on_line_with_length(2, 9),
                        subblocks: vec![],
                    },
                    Block {
                        key: GOAL_SPEC,
                        text: vec!["Apprendre des bases solides du C++"],
                        range: range_on_line_with_length(3, 39),
                        subblocks: vec![],
                    },
                ],
            }]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_detect_content_out_of_key() {
        let text = "
some random content

course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        let binding = ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            errors,
            vec![ParseError {
                range: range_on_line_with_length(1, 19),
                some_file: None,
                error: ParseErrorType::ContentOutOfKey
            }]
        );
        assert_eq!(
            blocks,
            vec![Block {
                key: COURSE_SPEC,
                text: vec!["Programmation 1"],
                range: range_on_line_with_length(3, 22),
                subblocks: vec![
                    Block {
                        key: CODE_SPEC,
                        text: vec!["PRG1"],
                        range: range_on_line_with_length(4, 9),
                        subblocks: vec![],
                    },
                    Block {
                        key: GOAL_SPEC,
                        text: vec!["Apprendre des bases solides du C++"],
                        range: range_on_line_with_length(5, 39),
                        subblocks: vec![],
                    },
                ],
            }]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_extract_complex_exos_blocks_with_errors_ignorance() {
        let text = "// great exo
exo hey
a great instruction
on several lines

check validate it
args John
see Hello John
type Doe
see Hello John Doe
exit 0

check error
args john doe
args invalid duplicated args !
see too many arguments
exit 1
exit double exit !

// Another one !
exo duplicated invalid exo !
check error with duplicate
"; // the challenge is to be able to ignore the check here as the exo key was ignored
        let binding = ValidDYSpec::new(TESTING_EXOS_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            errors,
            vec![
                ParseError {
                    range: range_on_line_with_length(14, 4),
                    some_file: None,
                    error: ParseErrorType::DuplicatedKey("args".to_string(), 2),
                },
                ParseError {
                    range: range_on_line_with_length(17, 4),
                    some_file: None,
                    error: ParseErrorType::DuplicatedKey("exit".to_string(), 2),
                },
                ParseError {
                    range: range_on_line_with_length(20, 3),
                    some_file: None,
                    error: ParseErrorType::DuplicatedKey("exo".to_string(), 0),
                },
            ]
        );
        assert_eq!(
            blocks,
            vec![
                Block {
                    key: EXO_SPEC,
                    text: vec!["hey", "a great instruction", "on several lines", ""],
                    range: range_on_lines(1, 4, 0),
                    subblocks: vec![
                        Block {
                            key: CHECK_SPEC,
                            text: vec!["validate it",],
                            range: range_on_line_with_length(5, 17),
                            subblocks: vec![
                                Block {
                                    key: ARGS_SPEC,
                                    text: vec!["John",],
                                    range: range_on_line_with_length(6, 9),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: SEE_SPEC,
                                    text: vec!["Hello John",],
                                    range: range_on_line_with_length(7, 14),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: TYPE_SPEC,
                                    text: vec!["Doe",],
                                    range: range_on_line_with_length(8, 8),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: SEE_SPEC,
                                    text: vec!["Hello John Doe",],
                                    range: range_on_line_with_length(9, 18),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: EXIT_SPEC,
                                    text: vec!["0",],
                                    range: range_on_line_with_length(10, 6),
                                    subblocks: vec![],
                                },
                            ],
                        },
                        Block {
                            key: CHECK_SPEC,
                            text: vec!["error",],
                            range: range_on_line_with_length(12, 11),
                            subblocks: vec![
                                Block {
                                    key: ARGS_SPEC,
                                    text: vec!["john doe",],
                                    range: range_on_line_with_length(13, 13),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: SEE_SPEC,
                                    text: vec!["too many arguments",],
                                    range: range_on_line_with_length(15, 22),
                                    subblocks: vec![],
                                },
                                Block {
                                    key: EXIT_SPEC,
                                    text: vec!["1",],
                                    range: range_on_line_with_length(16, 6),
                                    subblocks: vec![],
                                },
                            ],
                        },
                    ],
                },
                // no exo as a duplicate !
            ]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_strange_exo_parsing_can_correctly_ignore_error() {
        let text = "random text
exo hey there
some content
// just a comment
see not good because incorrect level
check yes
args 1
see good
args duplicated !
type good
check 2
";
        let binding = ValidDYSpec::new(TESTING_EXOS_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(
            errors,
            vec![
                ParseError {
                    range: range_on_line_with_length(0, 11),
                    some_file: None,
                    error: ParseErrorType::ContentOutOfKey
                },
                ParseError {
                    range: range_on_line_with_length(4, 3),
                    some_file: None,
                    error: ParseErrorType::WrongKeyPosition("see".to_string(), "??".to_string()),
                },
                ParseError {
                    range: range_on_line_with_length(8, 4),
                    some_file: None,
                    error: ParseErrorType::DuplicatedKey("args".to_string(), 2),
                },
            ]
        );
        assert_eq!(
            blocks,
            vec![Block {
                key: EXO_SPEC,
                text: vec!["hey there", "some content",],
                range: range_on_lines(1, 2, 12),
                subblocks: vec![
                    Block {
                        key: CHECK_SPEC,
                        text: vec!["yes",],
                        range: range_on_line_with_length(5, 9),
                        subblocks: vec![
                            Block {
                                key: ARGS_SPEC,
                                text: vec!["1",],
                                range: range_on_line_with_length(6, 6),
                                subblocks: vec![],
                            },
                            Block {
                                key: SEE_SPEC,
                                text: vec!["good",],
                                range: range_on_line_with_length(7, 8),
                                subblocks: vec![],
                            },
                            Block {
                                key: TYPE_SPEC,
                                text: vec!["good",],
                                range: range_on_line_with_length(9, 9),
                                subblocks: vec![],
                            },
                        ],
                    },
                    Block {
                        key: CHECK_SPEC,
                        text: vec!["2",],
                        range: range_on_line_with_length(10, 7),
                        subblocks: vec![],
                    },
                ],
            },]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_empty_lines_are_present_in_block_text() {
        let text = "exo hey there
some instruction

~~~

some code

~~~
";
        let binding = ValidDYSpec::new(TESTING_EXOS_SPEC).unwrap();
        let (blocks, errors) = get_blocks(&binding, text);
        assert_eq!(errors, vec![]);
        assert_eq!(
            blocks,
            vec![Block {
                key: EXO_SPEC,
                text: vec![
                    "hey there",
                    "some instruction",
                    "",
                    "~~~",
                    "",
                    "some code",
                    "",
                    "~~~",
                ],
                range: range_on_lines(0, 7, 3),
                subblocks: vec![],
            },]
        );
    }
}
