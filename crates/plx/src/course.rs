/// The DY spec for parsing PLX course. Given a string, parse it into `DYCourse` and generate some errors
use dy::{
    FromDYBlock, ParseResult,
    error::ParseError,
    parse_with_spec,
    parser::Block,
    spec::{DYSpec, KeySpec, ValidDYSpec, ValueType},
};
use serde::Serialize;

#[derive(Serialize, Default, Debug, PartialEq)]
pub struct DYCourse {
    pub name: String,
    pub code: String,
    pub goal: String,
}

const GOAL_KEYSPEC: &KeySpec = &KeySpec {
    id: "goal",
    desc: "The goal key describes the learning goals of this course.",
    subkeys: &[],
    vt: ValueType::Multiline,
    once: true,
    required: true,
};
const CODE_KEYSPEC: &KeySpec = &KeySpec {
    id: "code",
    desc: "The code of the course is a shorter name of the course, under 10 letters usually.",
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: true,
};
const COURSE_KEYSPEC: &KeySpec = &KeySpec {
    id: "course",
    desc: "A PLX course is grouping skills and exos related to a common set of learning goals.",
    subkeys: &[CODE_KEYSPEC, GOAL_KEYSPEC],
    vt: ValueType::SingleLine,
    once: true,
    required: true,
};
pub const COURSE_SPEC: &DYSpec = &[COURSE_KEYSPEC];

impl<'a> FromDYBlock<'a> for DYCourse {
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, DYCourse) {
        let errors = Vec::new();
        let mut course = DYCourse {
            name: block.get_joined_text(),
            ..Default::default()
        };
        for subblock in block.subblocks.iter() {
            let id = subblock.key.id;
            if id == CODE_KEYSPEC.id {
                course.code = subblock.get_joined_text();
            }
            if id == GOAL_KEYSPEC.id {
                course.goal = subblock.get_joined_text();
            }
        }
        (errors, course)
    }
}

pub fn parse_course(some_file: &Option<String>, content: &str) -> ParseResult<DYCourse> {
    parse_with_spec::<DYCourse>(
        &ValidDYSpec::new(COURSE_SPEC).expect("COURSE_SPEC is invalid !"),
        some_file,
        content,
    )
}

#[cfg(test)]
mod tests {
    use dy::ParseResult;

    use pretty_assertions::assert_eq;

    use crate::course::{DYCourse, parse_course};

    #[test]
    fn test_can_parse_simple_valid_course() {
        let text = "course Programmation 1
code PRG1
goal Apprendre des bases solides du C++";
        let some_file_path = Some("course.dy".to_string());
        assert_eq!(
            parse_course(&some_file_path, text),
            ParseResult {
                items: vec![DYCourse {
                    name: "Programmation 1".to_string(),
                    code: "PRG1".to_string(),
                    goal: "Apprendre des bases solides du C++".to_string()
                }],
                errors: vec![],
                some_file_path,
                some_file_content: None // on errors
            }
        )
    }

    #[test]
    fn test_parse_result_display_is_correct() {
        let text = "code YEP
course PRG1
goal Learn C++
course PRG2
goal hey";
        let expected_output = "Found 1 item in course.dy with 3 errors.

Error at course.dy:0:0
code YEP
^^^^ The 'code' key can be only used under a `??`

Error at course.dy:1:0
course PRG1
| Missing required key 'code'

Error at course.dy:3:0
course PRG2
^^^^^^ The 'course' key can only be used once in the document root
";

        let parse_result = parse_course(&Some("course.dy".to_string()), text);
        eprintln!("{parse_result}");
        assert_eq!(format!("{parse_result}"), expected_output);
    }

    #[test]
    fn test_parse_result_display_is_also_correct() {
        let text = "course
code PRG1
goal Learn C++
";
        let expected_output = "Found 1 item in course.dy with 1 error.

Error at course.dy:0:6
course
      | Missing a value for the required key 'course'
";
        // Hint: a course.dy file can only define a single course";

        let parse_result = parse_course(&Some("course.dy".to_string()), text);
        eprintln!("{parse_result}");
        assert_eq!(format!("{parse_result}"), expected_output);
    }

    #[test]
    fn test_parse_result_display_can_highlight_unknown_content() {
        let text = "// just a comment
what's this file ??
i don't know...

course Programmation 1
code PRG1
oupsii
goal hey";
        let expected_output = "Found 1 item in course.dy with 3 errors.

Error at course.dy:1:0
what's this file ??
^^^^^^^^^^^^^^^^^^^ This content is not associated to any valid key.
Hint: maybe this should be a comment starting with // or it needs a valid key as a prefix?

Error at course.dy:2:0
i don't know...
^^^^^^^^^^^^^^^ This content is not associated to any valid key.
Hint: maybe this should be a comment starting with // or it needs a valid key as a prefix?

Error at course.dy:6:0
oupsii
^^^^^^ Invalid multiline content found after the 'code' key which is single line
";
        // Hint: a course.dy file can only define a single course";

        let parse_result = parse_course(&Some("course.dy".to_string()), text);
        eprintln!("{parse_result}");
        assert_eq!(format!("{parse_result}"), expected_output);
    }
}
