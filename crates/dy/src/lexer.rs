/// The parser is responsible of the syntax analysis by cutting the content into lines, and lines into parts
use crate::spec::{KeySpec, ValidDYSpec, all_valid_keys};
use std::collections::HashMap;

pub const COMMENT_PREFIX: &str = "//";
const MARKDOWN_CODE_SNIPPETS_SEPARATORS: &[&str; 2] = &["```", "~~~"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineType<'a> {
    /// A line with a valid key at the start. The key can only be valid if it is in the spec and it is in the right level
    WithKey(&'a KeySpec<'a>),
    /// Just a comment outside of a valid code block, it will be completely ignored then
    Comment,
    /// We don't really know for now it it's a line of content after a WithKey or an invalid line that should not exist
    Unknown,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Line<'a> {
    pub(crate) index: usize,
    pub(crate) slice: &'a str,
    pub(crate) lt: LineType<'a>,
}

#[derive(Debug, Eq, PartialEq)]
/// A line part is a key, or any other value (after a key or not)
pub enum LinePart<'a> {
    Key(&'a str),
    Value(&'a str),
}

impl<'a> Line<'a> {
    pub(crate) fn tokenize_parts(&self) -> Vec<LinePart<'a>> {
        match self.lt {
            LineType::WithKey(key_spec) => {
                vec![
                    LinePart::Key(&self.slice[..key_spec.id.len()]),
                    LinePart::Value(self.slice[key_spec.id.len()..].trim()),
                ]
            }
            _ => vec![LinePart::Value(self.slice)],
        }
    }
}

/// Take all the lines of `content`, take a flat list of all valid keys in `spec`
/// and categorize lines between comments, starting with a key and put all the others in the `unknown` category.
/// A WithKey Line is not verified to be at a valid position !
pub fn tokenize_into_lines<'a>(spec: &'a ValidDYSpec, content: &'a str) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    let all_keys = all_valid_keys(spec.get());
    // For faster access to the correct key, we group them by length so when extracting the first
    // word, we can only look at keys with the same length
    let mut all_keys_grouped_by_len: HashMap<usize, Vec<&KeySpec>> = HashMap::new();
    all_keys.iter().for_each(|k| {
        all_keys_grouped_by_len
            .entry(k.id.len())
            .or_default()
            .push(k);
    });

    let mut inside_a_markdown_code_snippet = false;

    for (index, line_text) in content.lines().enumerate() {
        let mut lt = LineType::Unknown;

        for code_separator in MARKDOWN_CODE_SNIPPETS_SEPARATORS {
            if line_text.starts_with(code_separator) {
                inside_a_markdown_code_snippet = !inside_a_markdown_code_snippet;
            }
        }

        if inside_a_markdown_code_snippet {
            // just keep it as Unknown, we skill all lines inside markdown code snippets
        } else if line_text.starts_with(COMMENT_PREFIX) {
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
        common::tests::{
            CODE_SPEC, COURSE_SPEC, EXO_SPEC, GOAL_SPEC, TESTING_COURSE_SPEC, TESTING_EXOS_SPEC,
        },
        parser::{Line, LinePart, LineType, line_starts_with_key, tokenize_into_lines},
        spec::ValidDYSpec,
    };
    use pretty_assertions::assert_eq;

    #[test]
    #[ntest::timeout(50)]
    fn test_line_into_parts() {
        assert_eq!(
            Line {
                index: 0,
                slice: "course AB C D",
                lt: LineType::WithKey(COURSE_SPEC)
            }
            .tokenize_parts(),
            vec![LinePart::Key("course"), LinePart::Value("AB C D")]
        );
        assert_eq!(
            Line {
                index: 0,
                slice: "// something here",
                lt: LineType::Comment
            }
            .tokenize_parts(),
            vec![LinePart::Value("// something here")]
        );
        assert_eq!(
            Line {
                index: 0,
                slice: "something here",
                lt: LineType::Unknown
            }
            .tokenize_parts(),
            vec![LinePart::Value("something here")]
        );
    }

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
            tokenize_into_lines(&ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap(), text),
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
 // not a comment
";
        assert_eq!(
            tokenize_into_lines(&ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap(), text),
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
                Line {
                    index: 7,
                    slice: " // not a comment",
                    lt: LineType::Unknown,
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
        let binding = ValidDYSpec::new(TESTING_COURSE_SPEC).unwrap();
        let lines = tokenize_into_lines(&binding, text);
        assert_eq!(lines.len(), 5);
        dbg!(&lines);
        assert!(&lines.iter().all(|l| l.lt == LineType::Unknown));
    }

    #[test]
    #[ntest::timeout(50)]
    fn test_can_tokenize_and_ignore_anything_inside_code_blocks() {
        let text = "
// hey there
exo hey there
some instruction
~~~rust
// super function
// ?????
fn main() {
    // hey yooo
}

// ignored prefix
exo hey
see something
~~~
// ignored

```
// super css
h1{
color:blue; // included
}
/* included */
```
// ignored again!
";
        let binding = ValidDYSpec::new(TESTING_EXOS_SPEC).unwrap();
        let lines = tokenize_into_lines(&binding, text);
        assert_eq!(
            lines,
            vec![
                Line {
                    index: 0,
                    slice: "",
                    lt: LineType::Unknown
                },
                Line {
                    index: 1,
                    slice: "// hey there",
                    lt: LineType::Comment,
                },
                Line {
                    index: 2,
                    slice: "exo hey there",
                    lt: LineType::WithKey(EXO_SPEC)
                },
                Line {
                    index: 3,
                    slice: "some instruction",
                    lt: LineType::Unknown
                },
                Line {
                    index: 4,
                    slice: "~~~rust",
                    lt: LineType::Unknown
                },
                Line {
                    index: 5,
                    slice: "// super function",
                    lt: LineType::Unknown,
                },
                Line {
                    index: 6,
                    slice: "// ?????",
                    lt: LineType::Unknown,
                },
                Line {
                    index: 7,
                    slice: "fn main() {",
                    lt: LineType::Unknown
                },
                Line {
                    index: 8,
                    slice: "    // hey yooo",
                    lt: LineType::Unknown
                },
                Line {
                    index: 9,
                    slice: "}",
                    lt: LineType::Unknown
                },
                Line {
                    index: 10,
                    slice: "",
                    lt: LineType::Unknown
                },
                Line {
                    index: 11,
                    slice: "// ignored prefix",
                    lt: LineType::Unknown,
                },
                Line {
                    index: 12,
                    slice: "exo hey",
                    lt: LineType::Unknown
                },
                Line {
                    index: 13,
                    slice: "see something",
                    lt: LineType::Unknown
                },
                Line {
                    index: 14,
                    slice: "~~~",
                    lt: LineType::Unknown
                },
                Line {
                    index: 15,
                    slice: "// ignored",
                    lt: LineType::Comment,
                },
                Line {
                    index: 16,
                    slice: "",
                    lt: LineType::Unknown
                },
                Line {
                    index: 17,
                    slice: "```",
                    lt: LineType::Unknown
                },
                Line {
                    index: 18,
                    slice: "// super css",
                    lt: LineType::Unknown,
                },
                Line {
                    index: 19,
                    slice: "h1{",
                    lt: LineType::Unknown
                },
                Line {
                    index: 20,
                    slice: "color:blue; // included",
                    lt: LineType::Unknown
                },
                Line {
                    index: 21,
                    slice: "}",
                    lt: LineType::Unknown
                },
                Line {
                    index: 22,
                    slice: "/* included */",
                    lt: LineType::Unknown
                },
                Line {
                    index: 23,
                    slice: "```",
                    lt: LineType::Unknown
                },
                Line {
                    index: 24,
                    slice: "// ignored again!",
                    lt: LineType::Comment,
                },
            ]
        );
    }
}
