use std::collections::HashMap;

/// The parser engine is responsible to parse a given piece of text, with a given spec
use crate::spec::{all_valid_keys, KeySpec, ValidDYSpec};

const COMMENT_PREFIX: &str = "//";

#[derive(Clone, Debug, Eq, PartialEq)]
enum LineType<'a> {
    /// A line with a valid key at the start. The key can only be valid if it is in the spec and it is in the right level
    WithKey(&'a KeySpec<'a>),
    /// Just a comment outside of a valid code block, it will be completely ignored then
    Comment,
    /// We don't really know for now it it's a line of content after a WithKey or an invalid line that should not exist
    Unknown,
}

#[derive(Debug, Eq, PartialEq)]
struct Line<'a> {
    index: usize,
    slice: &'a str,
    lt: LineType<'a>,
}

/// Take all the lines of `content`, take a flat list of all valid keys in `spec`
/// and categorize lines between comments, starting with a key and put all the others in the `unknown` category.
/// A WithKey Line is not verified to be at a valid position !
fn tokenize_into_lines<'a>(spec: &'a ValidDYSpec, content: &'a str) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    let all_keys = all_valid_keys(spec.get());
    let mut all_keys_grouped_by_len: HashMap<usize, Vec<&KeySpec>> = HashMap::new();
    all_keys.iter().for_each(|k| {
        all_keys_grouped_by_len
            .entry(k.id.len())
            .or_default()
            .push(k);
    });

    for (index, line_text) in content.lines().enumerate() {
        let mut lt = LineType::Unknown;

        if line_text.starts_with(COMMENT_PREFIX) {
            lt = LineType::Comment;
        } else {
            // Extract the first word before the first space, if there is no space, the first word is the entire line
            let first_word = line_text.split(" ").next().unwrap_or(line_text);

            // If there is a key with the same length as the first word, that is equal
            if let Some(possible_keys) = all_keys_grouped_by_len.get(&first_word.len()) {
                for key in possible_keys {
                    if line_starts_with_key(line_text, key.id) {
                        lt = LineType::WithKey(key);
                        break;
                    }
                }
            }
        }

        // Finally push the line, it might be in LineType::Unknown yet
        lines.push(Line {
            index,
            slice: line_text,
            lt: lt.clone(),
        });
    }

    lines
}

/// Make sure the given line starts with a prefix and is followed by nothing or a space or a \n
#[inline(always)]
fn line_starts_with_key(line: &str, prefix: &str) -> bool {
    if !line.starts_with(prefix) {
        return false;
    }

    if line.len() > prefix.len()
        && line.chars().nth(prefix.len()) != Some(' ')
        && line.chars().nth(prefix.len()) != Some('\n')
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::{
        common::tests::{CODE_SPEC, COURSE_SPEC, GOAL_SPEC, PLX_COURSE_SPEC},
        parser::{line_starts_with_key, tokenize_into_lines, Line, LineType},
        spec::ValidDYSpec,
    };
    use pretty_assertions::assert_eq;

    #[test]
    #[ntest::timeout(50)]
    fn test_line_starts_with_key() {
        assert!(line_starts_with_key("course hey there", "course"));
        assert!(line_starts_with_key("course", "course"));
        assert!(line_starts_with_key("course ", "course"));
        assert!(line_starts_with_key("course\n", "course"));
        assert!(!line_starts_with_key("coursea", "course"));
        assert!(!line_starts_with_key("course$", "course"));
        assert!(!line_starts_with_key("cour", "course"));
        assert!(!line_starts_with_key("cour", "course"));
    }
    #[test]
    #[ntest::timeout(50)]
    fn test_can_tokenize_basic_lines() {
        let text = "course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        assert_eq!(
            tokenize_into_lines(&ValidDYSpec::new(PLX_COURSE_SPEC).unwrap(), text),
            vec![
                Line {
                    index: 0,
                    slice: "course Programmation 1",
                    lt: LineType::WithKey(COURSE_SPEC)
                },
                Line {
                    index: 1,
                    slice: "code PRG1",
                    lt: LineType::WithKey(CODE_SPEC)
                },
                Line {
                    index: 2,
                    slice: "goal Apprendre des bases solides du C++",
                    lt: LineType::WithKey(GOAL_SPEC)
                }
            ]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_tokenize_comments_and_empty() {
        let text = "// just a comment
course Programmation 1
code PRG1
// another comment
goal Apprendre des bases solides du C++

// yet another one
";
        assert_eq!(
            tokenize_into_lines(&ValidDYSpec::new(PLX_COURSE_SPEC).unwrap(), text),
            vec![
                Line {
                    index: 0,
                    slice: "// just a comment",
                    lt: LineType::Comment,
                },
                Line {
                    index: 1,
                    slice: "course Programmation 1",
                    lt: LineType::WithKey(COURSE_SPEC)
                },
                Line {
                    index: 2,
                    slice: "code PRG1",
                    lt: LineType::WithKey(CODE_SPEC)
                },
                Line {
                    index: 3,
                    slice: "// another comment",
                    lt: LineType::Comment,
                },
                Line {
                    index: 4,
                    slice: "goal Apprendre des bases solides du C++",
                    lt: LineType::WithKey(GOAL_SPEC)
                },
                Line {
                    index: 5,
                    slice: "",
                    lt: LineType::Unknown,
                },
                Line {
                    index: 6,
                    slice: "// yet another one",
                    lt: LineType::Comment,
                },
            ]
        );
    }

    #[test]
    #[ntest::timeout(50)]
    // All empty lines are considered Unknown for the tokenizer
    fn test_can_tokenize_lines_with_invalid_keys_and_empty_lines() {
        let text = "courseo Programmation 1
codehey
goalApprendre slaut

blabla";
        let binding = ValidDYSpec::new(PLX_COURSE_SPEC).unwrap();
        let lines = tokenize_into_lines(&binding, text);
        assert_eq!(lines.len(), 5);
        dbg!(&lines);
        assert!(&lines.iter().all(|l| l.lt == LineType::Unknown));
    }
}
