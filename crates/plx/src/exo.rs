use dy::{
    FromDYBlock, ParseResult,
    error::{ParseError, ParseErrorType},
    parse_with_spec, range_on_line_part,
    semantic::Block,
    spec::{DYSpec, KeySpec, ValidDYSpec, ValueType},
};

/// This describes the automation of an action that would normally be done manually in the terminal
#[derive(Debug, PartialEq)]
enum TermAction {
    /// Make sure there is the given string in the program stdout. It doesn't need to be exact.
    /// This string is trimed itself to avoid any missing invisible space causing check failure
    See(String),
    /// Type something in the terminal, by injecting content into stdin at once,
    /// including an additionnal new line \n at the end
    Type(String),
}

#[derive(Default, Debug, PartialEq)]
struct Check {
    name: String,
    /// The argument to pass to exo program when executing it
    args: Vec<String>,
    /// The expected exit code of the exo program
    exit: Option<i32>, // why i32 ? because std::process::ExitStatus::code() -> Option<i32>
    /// The test sequence containing assertions to verify the behavior of the exo program
    sequence: Vec<TermAction>,
}

#[derive(Default, Debug, PartialEq)]
pub struct DYExo {
    name: String,
    instruction: String,
    checks: Vec<Check>,
}

pub const ARGS_SPEC: &KeySpec = &KeySpec {
    id: "args",
    desc: "The command line arguments passed to the exo program, the space is used to split the list of arguments. No quotes or space inside argument is supported at the moment.",
    // TODO: support a way to have arguments with space !
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: false,
};
pub const SEE_SPEC: &KeySpec = &KeySpec {
    id: "see",
    desc: "The `see` assertion asserts that the standard output of the exo program contains the given text. Values around that text are permitted.",
    subkeys: &[],
    vt: ValueType::Multiline,
    once: false,
    required: true,
};
pub const TYPE_SPEC: &KeySpec = &KeySpec {
    id: "type",
    desc: "The `type` action simulate typing in the terminal and hitting enter. It inject the given text in the standard input at once after appending a `\\n` at the end of the text.",
    subkeys: &[],
    vt: ValueType::SingleLine, // we can only type a single line of text
    once: false,
    required: false,
};
pub const EXIT_SPEC: &KeySpec = &KeySpec {
    desc: "Assert the value of the exit code (also named exit status). By default, this is checked to be 0, you can define another value to assert the program has failed with a specific exit code.",
    id: "exit",
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: false,
};
pub const CHECK_SPEC: &KeySpec = &KeySpec {
    id: "check",
    desc: "Describe a `check`, which is a basic automated test.",
    subkeys: &[ARGS_SPEC, SEE_SPEC, TYPE_SPEC, EXIT_SPEC],
    vt: ValueType::SingleLine,
    once: false,
    required: true,
};
pub const EXO_SPEC: &KeySpec = &KeySpec {
    id: "exo",
    desc: "Define a new exercise (exo is shortcut for exercise) with a name and optionnal instruction.",
    subkeys: &[CHECK_SPEC],
    vt: ValueType::Multiline,
    once: true, // for now, only one exo per file
    required: true,
};

// Note: to avoid double definition of EXO_SPEC we use the plural form
// even though only one course can be extracted
pub const EXOS_SPEC: &DYSpec = &[EXO_SPEC];

// Error texts
const ERROR_CANNOT_PARSE_EXIT_CODE: &str = "Couldn't parse the given value as the program's exit code, which is an unsigned 32 bits integer.";

impl<'a> FromDYBlock<'a> for DYExo {
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, DYExo) {
        let mut errors = Vec::new();
        let mut exo = DYExo::default();
        // The first line is the name, the following ones are the description
        (exo.name, exo.instruction) = block.get_text_with_joined_splits_at(1);
        for exo_subblock in block.subblocks.iter() {
            let id = exo_subblock.key.id;
            if id == CHECK_SPEC.id {
                let mut check = Check {
                    name: exo_subblock.get_joined_text(),
                    ..Default::default()
                };
                for check_subblock in exo_subblock.subblocks.iter() {
                    let check_subblock_id = check_subblock.key.id;
                    if check_subblock_id == ARGS_SPEC.id {
                        check.args = check_subblock.text.iter().map(|s| s.to_string()).collect()
                    }
                    if check_subblock_id == EXIT_SPEC.id {
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
                                    some_file: None,
                                    error: ParseErrorType::ValidationError(
                                        ERROR_CANNOT_PARSE_EXIT_CODE.to_string(),
                                    ),
                                });
                            }
                        }
                    }
                    if check_subblock_id == TYPE_SPEC.id {
                        check
                            .sequence
                            .push(TermAction::Type(check_subblock.get_joined_text()));
                    }
                    if check_subblock_id == SEE_SPEC.id {
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

pub fn parse_exos(content: &str) -> ParseResult<DYExo> {
    parse_with_spec(
        &ValidDYSpec::new(EXOS_SPEC).expect("EXOS_SPEC is invalid !"),
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
args kinda useless
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
        assert_eq!(
            parse_exos(text),
            ParseResult {
                items: vec![DYExo {
                    name: "Just greet me".to_string(),
                    instruction: "A simple hello program that **asks your firstname and lastname and greets you**.\nMake sure to validate firstname and lastname content. They must contain only A-Z and a-z chars. \nDo not use a regex. Try to avoid repeating the validation logic.\n\nThe goal is to train input/output with `printf` and `scanf`.".to_string(),
                    checks: vec![
                        Check {
                            name: "Can enter the full name and be greeted".to_string(),
                            args: vec!["kinda useless".to_string(),],
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
        assert_eq!(
            parse_exos(text),
            ParseResult {
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
                    some_file: None,
                    error: ParseErrorType::ValidationError(
                        ERROR_CANNOT_PARSE_EXIT_CODE.to_string()
                    )
                }]
            }
        )
    }
}
