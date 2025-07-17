/// The DY spec for parsing PLX course. Given a string, parse it into `DYCourse` and generate some errors
use dy::{
    FromDYBlock, ParseResult,
    error::ParseError,
    parse_with_spec,
    semantic::Block,
    spec::{DYSpec, KeySpec, KeyType, ValidDYSpec},
};

#[derive(Default, Debug, PartialEq)]
pub struct DYCourse {
    name: String,
    instruction: String,
    code: String,
    goal: String,
}

pub const GOAL_SPEC: &KeySpec = &KeySpec {
    id: "goal",
    desc: "The goal key describes the learning goals of this course.",
    subkeys: &[],
    kt: KeyType::Multiline,
    once: true,
    required: true,
};
pub const CODE_SPEC: &KeySpec = &KeySpec {
    id: "code",
    desc: "The code of the course is a shorter name of the course, under 10 letters usually.",
    subkeys: &[],
    kt: KeyType::SingleLine,
    once: true,
    required: true,
};
pub const COURSE_SPEC: &KeySpec = &KeySpec {
    id: "course",
    desc: "A PLX course is grouping skills and exos related to a common set of learning goals.",
    subkeys: &[CODE_SPEC, GOAL_SPEC],
    kt: KeyType::SingleLine,
    once: true,
    required: true,
};
// Note: to avoid double definition of COURSE_SPEC we use the plural form
// even though only one course can be extracted
pub const COURSES_SPEC: &DYSpec = &[COURSE_SPEC];

impl<'a> FromDYBlock<'a> for DYCourse {
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, DYCourse) {
        let mut errors = Vec::new();
        let mut course = DYCourse {
            name: block.get_joined_text(),
            ..Default::default()
        };
        // The first line is the name, the following ones are the instruction
        for subblock in block.subblocks.iter() {
            let id = subblock.key.id;
            if id == CODE_SPEC.id {
                course.code = subblock.get_joined_text();
            }
            if id == GOAL_SPEC.id {
                course.goal = subblock.text.join("\n");
            }
        }
        (errors, course)
    }
}

pub fn parse_course(content: &str) -> ParseResult<DYCourse> {
    parse_with_spec::<DYCourse>(
        &ValidDYSpec::new(COURSES_SPEC).expect("TESTING_COURSE_SPEC is invalid !"),
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
        assert_eq!(
            parse_course(text),
            ParseResult {
                items: vec![DYCourse {
                    name: "Programmation 1".to_string(),
                    instruction: "".to_string(),
                    code: "PRG1".to_string(),
                    goal: "Apprendre des bases solides du C++".to_string()
                }],
                errors: vec![]
            }
        )
    }
}
