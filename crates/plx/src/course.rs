// Testing the DY parser with the spec of PLX course
// use dy::parser::{parse_with_spec, ParseResult};
// use dy::spec::{DYSpec, KeySpec};

// How to define these 3 keys, with "course" as the entity prefix
// course single line but entity
//     code single line
//     goal -> multline

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
    subkeys: &[],
    kt: KeyType::Multiline,
    once: true,
};
pub const CODE_SPEC: &KeySpec = &KeySpec {
    id: "code",
    subkeys: &[],
    kt: KeyType::SingleLine,
    once: true,
};
pub const COURSE_SPEC: &KeySpec = &KeySpec {
    id: "course",
    subkeys: &[CODE_SPEC, GOAL_SPEC],
    kt: KeyType::SingleLine,
    once: true,
};
pub const TESTING_COURSE_SPEC: &DYSpec = &[COURSE_SPEC];

impl<'a> FromDYBlock<'a> for DYCourse {
    fn from_block(block: &Block<'a>) -> DYCourse {
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
        course
    }

    fn validate(&self) -> Vec<ParseError> {
        let mut errors = Vec::new();
        errors
    }
}

pub fn parse_course(content: &str) -> ParseResult<DYCourse> {
    parse_with_spec::<DYCourse>(
        &ValidDYSpec::new(TESTING_COURSE_SPEC).expect("TESTING_COURSE_SPEC is invalid !"),
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
