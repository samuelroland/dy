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
struct DYSkill {
    name: String,
    description: String,
    subskills: Vec<DYSkill>,
}

pub const SUBSKILL_SPEC: &KeySpec = &KeySpec {
    id: "subskill",
    subkeys: &[],
    kt: KeyType::Multiline,
    once: false,
};
pub const SKILL_SPEC: &KeySpec = &KeySpec {
    id: "skill",
    subkeys: &[SUBSKILL_SPEC],
    kt: KeyType::Multiline,
    once: false,
};
pub const TESTING_SKILLS_SPEC: &DYSpec = &[SKILL_SPEC];

impl<'a> FromDYBlock<'a> for DYSkill {
    fn from_block(block: &Block<'a>) -> DYSkill {
        dbg!(block);
        let mut skill = DYSkill::default();
        // The first line is the name, the following ones are the description
        (skill.name, skill.description) = block.get_text_with_joined_splits_at(1);
        for subblock in block.subblocks.iter() {
            let id = subblock.key.id;
            if id == SUBSKILL_SPEC.id {
                skill.subskills.push(DYSkill::from_block(subblock))
            }
        }
        skill
    }

    fn validate(&self) -> Vec<ParseError> {
        let mut errors = Vec::new();
        errors
    }
}

fn parse_skills(content: &str) -> ParseResult<DYSkill> {
    parse_with_spec(
        &ValidDYSpec::new(TESTING_SKILLS_SPEC).expect("TESTING_SKILLS_SPEC is invalid !"),
        content,
    )
}

#[cfg(test)]
mod tests {
    use dy::ParseResult;

    use crate::skill::{DYSkill, parse_skills};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_can_parse_simple_skills() {
        let text = "
// Great skills list
skill Classes

skill Opérateurs
Maitriser l'usage de **tous les opérateurs utiles**, pour manipuler
des nombres, des bits, ou encore des flux. Redéfinir les opérateurs
existants pour nos classes.
subskill Calculs simples
Juste des calculs de math !

subskill Calculs avancés et précision
subskill Opérateurs de flux

subskill Manipulation de bits
subskill Redéfinition d'opérateurs

";
        assert_eq!(
            parse_skills(text),
            ParseResult {
        items: vec![
            DYSkill {
                name: "Classes".to_string(),
                description: "".to_string(),
                subskills: vec![],
            },
            DYSkill {
                name: "Opérateurs".to_string(),
                description: "Maitriser l'usage de **tous les opérateurs utiles**, pour manipuler\ndes nombres, des bits, ou encore des flux. Redéfinir les opérateurs\nexistants pour nos classes.".to_string(),
                subskills: vec![
                    DYSkill {
                        name: "Calculs simples".to_string(),
                        description: "Juste des calculs de math !".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Calculs avancés et précision".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Opérateurs de flux".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Manipulation de bits".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Redéfinition d'opérateurs".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                ],
            },
        ],
                errors: vec![]
            }
        )
    }
}
