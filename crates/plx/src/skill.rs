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
    desc: "The subskill is the same as a skill but must be more specific and focused.",
    subkeys: &[],
    kt: KeyType::Multiline,
    once: false,
    required: false,
};
pub const SKILL_SPEC: &KeySpec = &KeySpec {
    id: "skill",
    desc: "The skill is describing what students are expected to be able to do. Subskills can be used to define more specific inner skills.\nThe first line is the skill name and following lines define the details of the skill.",
    subkeys: &[SUBSKILL_SPEC],
    kt: KeyType::Multiline,
    once: false,
    required: true,
};
pub const SKILLS_SPEC: &DYSpec = &[SKILL_SPEC];

impl<'a> FromDYBlock<'a> for DYSkill {
    fn from_block_with_validation(block: &Block<'a>) -> (Vec<ParseError>, DYSkill) {
        let mut errors = Vec::new();
        let mut skill = DYSkill::default();
        // The first line is the name, the following ones are the description
        (skill.name, skill.description) = block.get_text_with_joined_splits_at(1);
        for subblock in block.subblocks.iter() {
            let id = subblock.key.id;
            if id == SUBSKILL_SPEC.id {
                let (suberrors, subentity) = DYSkill::from_block_with_validation(subblock);
                skill.subskills.push(subentity);
                errors.extend(suberrors);
            }
        }
        (errors, skill)
    }
}

fn parse_skills(content: &str) -> ParseResult<DYSkill> {
    parse_with_spec(
        &ValidDYSpec::new(SKILLS_SPEC).expect("TESTING_SKILLS_SPEC is invalid !"),
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
