use dy::{
    FromDYBlock, ParseResult,
    error::{ParseError, ParseErrorType},
    parse_with_spec, range_on_line_part,
    semantic::Block,
    spec::{DYSpec, KeySpec, ValidDYSpec, ValueType},
};
use serde::Serialize;

/// This describes the automation of an action that would normally be done manually in the terminal
#[derive(Serialize, Debug, PartialEq)]
pub enum TermAction {
    /// Make sure there is the given string in the program stdout. It doesn't need to be exact.
    /// This string is trimed itself to avoid any missing invisible space causing check failure
    See(String),
    /// Type something in the terminal, by injecting content into stdin at once,
    /// including an additionnal new line \n at the end
    Type(String),
}

#[derive(Serialize, Default, Debug, PartialEq)]
pub struct Check {
    pub name: String,
    /// The argument to pass to exo program when executing it
    pub args: Vec<String>,
    /// The expected exit code of the exo program
    pub exit: Option<i32>, // why i32 ? because std::process::ExitStatus::code() -> Option<i32>
    /// The test sequence containing assertions to verify the behavior of the exo program
    pub sequence: Vec<TermAction>,
}

#[derive(Serialize, Default, Debug, PartialEq)]
pub struct DYExo {
    pub name: String,
    pub instruction: String,
    pub checks: Vec<Check>,
}

const ARGS_KEYSPEC: &KeySpec = &KeySpec {
    id: "args",
    desc: "The command line arguments passed to the exo program, the space is used to split the list of arguments. No quotes or space inside argument is supported at the moment.",
    // TODO: support a way to have arguments with space !
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: false,
};
const SEE_KEYSPEC: &KeySpec = &KeySpec {
    id: "see",
    desc: "The `see` assertion asserts that the standard output of the exo program contains the given text. Values around that text are permitted.",
    subkeys: &[],
    vt: ValueType::Multiline,
    once: false,
    required: true,
};
const TYPE_KEYSPEC: &KeySpec = &KeySpec {
    id: "type",
    desc: "The `type` action simulate typing in the terminal and hitting enter. It inject the given text in the standard input at once after appending a `\\n` at the end of the text.",
    subkeys: &[],
    vt: ValueType::SingleLine, // we can only type a single line of text. The type value can be empty, it just means we type enter without anything before.
    once: false,
    required: false,
};
const EXIT_KEYSPEC: &KeySpec = &KeySpec {
    desc: "Assert the value of the exit code (also named exit status). By default, this is checked to be 0, you can define another value to assert the program has failed with a specific exit code.",
    id: "exit",
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: false,
};
const CHECK_KEYSPEC: &KeySpec = &KeySpec {
    id: "check",
    desc: "Describe a `check`, which is a basic automated test.",
    subkeys: &[ARGS_KEYSPEC, SEE_KEYSPEC, TYPE_KEYSPEC, EXIT_KEYSPEC],
    vt: ValueType::SingleLine,
    once: false,
    required: true,
};
const EXO_KEYSPEC: &KeySpec = &KeySpec {
    id: "exo",
    desc: "Define a new exercise (exo is shortcut for exercise) with a name and optionnal instruction.",
    subkeys: &[CHECK_KEYSPEC],
    vt: ValueType::Multiline,
    once: true, // for now, only one exo per file
    required: true,
};

pub const EXO_SPEC: &DYSpec = &[EXO_KEYSPEC];

// Error texts
const ERROR_CANNOT_PARSE_EXIT_CODE: &str =
    "Couldn't parse the given value as the program's exit code (signed 32bits integer)";

impl<'a> FromDYBlock<'a> for DYExo {
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, DYExo) {
        let mut errors = Vec::new();
        let mut exo = DYExo::default();
        // The first line is the name, the following ones are the description
        (exo.name, exo.instruction) = block.get_text_with_joined_splits_at(1);
        for exo_subblock in block.subblocks.iter() {
            let id = exo_subblock.key.id;
            if id == CHECK_KEYSPEC.id {
                let mut check = Check {
                    name: exo_subblock.get_joined_text(),
                    ..Default::default()
                };
                for check_subblock in exo_subblock.subblocks.iter() {
                    let check_subblock_id = check_subblock.key.id;
                    if check_subblock_id == ARGS_KEYSPEC.id {
                        let args_text = &check_subblock.get_joined_text();
                        if args_text.is_empty() {
                            errors.push(ParseError {
                                // Note: the range is pointing just after the key as it's where the value need to come
                                range: range_on_line_part(
                                    check_subblock.range.start.line,
                                    ARGS_KEYSPEC.id.len() as u32,
                                    ARGS_KEYSPEC.id.len() as u32,
                                ),
                                error: ParseErrorType::MissingRequiredValue(
                                    check_subblock_id.to_string(),
                                ),
                            });
                        } else {
                            check.args = split_args_string(args_text);
                        }
                    }
                    if check_subblock_id == EXIT_KEYSPEC.id {
                        check.exit = None;
                        match check_subblock.get_joined_text().parse::<i32>() {
                            Ok(code) => check.exit = Some(code),
                            Err(_) => {
                                errors.push(ParseError {
                                    range: range_on_line_part(
                                        check_subblock.range.start.line,
                                        check_subblock.range.start.character
                                            + check_subblock_id.len() as u32
                                            + 1,
                                        check_subblock.range.end.character,
                                    ),
                                    error: ParseErrorType::ValidationError(
                                        ERROR_CANNOT_PARSE_EXIT_CODE.to_string(),
                                    ),
                                });
                            }
                        }
                    }
                    if check_subblock_id == TYPE_KEYSPEC.id {
                        check
                            .sequence
                            .push(TermAction::Type(check_subblock.get_joined_text()));
                    }
                    if check_subblock_id == SEE_KEYSPEC.id {
                        check
                            .sequence
                            .push(TermAction::See(check_subblock.get_joined_text()));
                    }
                }
                exo.checks.push(check);
            }
        }
        (errors, exo)
    }
}

// For now we only break on space, that's a bit limited if we need to have args that include space
// in them. This will be fixed in the future when needed.
fn split_args_string(line: &str) -> Vec<String> {
    if line.is_empty() {
        vec![]
    } else {
        line.split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }
}

pub fn parse_exos(some_file: &Option<String>, content: &str) -> ParseResult<DYExo> {
    parse_with_spec(
        &ValidDYSpec::new(EXO_SPEC).expect("EXOS_SPEC is invalid !"),
        some_file,
        content,
    )
}

#[cfg(test)]
mod tests {
    use dy::{
        ParseResult,
        error::{ParseError, ParseErrorType},
        range_on_line_part,
    };

    use crate::exo::{Check, DYExo, ERROR_CANNOT_PARSE_EXIT_CODE, TermAction, parse_exos};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_can_parse_a_simple_exo() {
        let text = "
// the basic just greet me exo !
exo Just greet me
A simple hello program that **asks your firstname and lastname and greets you**.
Make sure to validate firstname and lastname content. They must contain only A-Z and a-z chars. 
Do not use a regex. Try to avoid repeating the validation logic.

The goal is to train input/output with `printf` and `scanf`.

check Can enter the full name and be greeted
args kinda
see What is your firstname ?
type John
see Hello John, what's your lastname ?
type Doe
see Have a nice day John Doe !
exit 0

check It validates the firstname text
see What is your firstname ?
type John23
see This doesn't look like a firstname...
exit 2
";
        let some_file = &Some("exo.dy".to_string());
        assert_eq!(
            parse_exos(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: None,
                items: vec![DYExo {
                    name: "Just greet me".to_string(),
                    instruction: "A simple hello program that **asks your firstname and lastname and greets you**.\nMake sure to validate firstname and lastname content. They must contain only A-Z and a-z chars. \nDo not use a regex. Try to avoid repeating the validation logic.\n\nThe goal is to train input/output with `printf` and `scanf`.".to_string(),
                    checks: vec![
                        Check {
                            name: "Can enter the full name and be greeted".to_string(),
                            args: vec!["kinda".to_string(),],
                            exit: Some(0,),
                            sequence: vec![
                                TermAction::See("What is your firstname ?".to_string(),),
                                TermAction::Type("John".to_string(),),
                                TermAction::See("Hello John, what's your lastname ?".to_string(),),
                                TermAction::Type("Doe".to_string(),),
                                TermAction::See("Have a nice day John Doe !".to_string(),),
                            ],
                        },
                        Check {
                            name: "It validates the firstname text".to_string(),
                            args: vec![],
                            exit: Some(2,),
                            sequence: vec![
                                TermAction::See("What is your firstname ?".to_string(),),
                                TermAction::Type("John23".to_string(),),
                                TermAction::See("This doesn't look like a firstname...".to_string(),),
                            ],
                        },
                    ],
                },],
                errors: vec![]
            }
        )
    }

    #[test]
    fn test_can_error_on_invalid_exit_code() {
        let text = "exo test
check test
see hello
exit blabla
";
        let some_file = &Some("exo.dy".to_string());
        assert_eq!(
            parse_exos(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: Some(text.to_string()),
                items: vec![DYExo {
                    name: "test".to_string(),
                    instruction: "".to_string(),
                    checks: vec![Check {
                        name: "test".to_string(),
                        args: vec![],
                        exit: None,
                        sequence: vec![TermAction::See("hello".to_string(),),],
                    },],
                }],
                errors: vec![ParseError {
                    range: range_on_line_part(3, 5, 11),
                    error: ParseErrorType::ValidationError(
                        ERROR_CANNOT_PARSE_EXIT_CODE.to_string()
                    )
                }]
            }
        )
    }

    #[test]
    fn test_can_extract_args_by_space_split() {
        let text = "exo test
check test
args 1 2 3 hey there
see hello
";
        let some_file = &Some("exo.dy".to_string());
        assert_eq!(
            parse_exos(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: None,
                items: vec![DYExo {
                    name: "test".to_string(),
                    instruction: "".to_string(),
                    checks: vec![Check {
                        name: "test".to_string(),
                        args: vec![
                            "1".to_string(),
                            "2".to_string(),
                            "3".to_string(),
                            "hey".to_string(),
                            "there".to_string()
                        ],
                        exit: None,
                        sequence: vec![TermAction::See("hello".to_string(),),],
                    },],
                }],
                errors: vec![]
            }
        )
    }

    #[test]
    fn test_detect_empty_args_error_but_ignores_empty_type() {
        let text = "exo test
check test
see hello
// that's an error
args
// that's okay
type
";
        let some_file = &Some("exo.dy".to_string());
        assert_eq!(
            parse_exos(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: Some(text.to_string()),
                items: vec![DYExo {
                    name: "test".to_string(),
                    instruction: "".to_string(),
                    checks: vec![Check {
                        name: "test".to_string(),
                        args: vec![],
                        exit: None,
                        sequence: vec![
                            TermAction::See("hello".to_string(),),
                            TermAction::Type("".to_string())
                        ],
                    },],
                }],
                errors: vec![ParseError {
                    range: range_on_line_part(4, 4, 4),
                    error: ParseErrorType::MissingRequiredValue("args".to_string()),
                }]
            }
        )
    }
}
