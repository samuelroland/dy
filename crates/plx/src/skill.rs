use dy::{
    FromDYBlock, ParseResult,
    error::{ParseError, ParseErrorType},
    parse_with_spec, range_on_line_part,
    semantic::Block,
    spec::{DYSpec, KeySpec, ValidDYSpec, ValueType},
};

#[derive(Default, Debug, PartialEq)]
pub struct DYSkill {
    pub name: String,
    pub description: String,
    /// the directory where associated exos are stored, the existance of this directory cannot be checked by the parser as it doesn't touch the file system !
    pub directory: String,
    /// Note: subskills are not supported by PLX at the moment, they are just ignored
    pub subskills: Vec<DYSkill>,
}
pub const DIR_SPEC: &KeySpec = &KeySpec {
    id: "dir",
    desc: "The directory where exos of this skill are stored. This directory must be unique among listed skills.",
    subkeys: &[],
    vt: ValueType::SingleLine,
    once: true,
    required: true,
};
// TODO: how to support dir also for subskill ? this is detected as a duplicated keyspec !
// For now, PLX doesn't support subskills so we will just ignore them when converting DYSkill to Skill
pub const SUBSKILL_SPEC: &KeySpec = &KeySpec {
    id: "subskill",
    desc: "The subskill is the same as a skill but must be more specific and focused.",
    subkeys: &[],
    vt: ValueType::Multiline,
    once: false,
    required: false,
};
pub const SKILL_SPEC: &KeySpec = &KeySpec {
    id: "skill",
    desc: "The skill is describing what students are expected to be able to do. Subskills can be used to define more specific inner skills.\nThe first line is the skill name and following lines define the details of the skill.",
    subkeys: &[SUBSKILL_SPEC, DIR_SPEC],
    vt: ValueType::Multiline,
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
            if id == DIR_SPEC.id {
                skill.directory = subblock.get_joined_text()
            }
            if id == SUBSKILL_SPEC.id {
                // Make sure subskill value is not empty
                if subblock.get_joined_text().is_empty() {
                    errors.push(ParseError {
                        range: range_on_line_part(
                            subblock.range.start.line,
                            SUBSKILL_SPEC.id.len() as u32,
                            SUBSKILL_SPEC.id.len() as u32,
                        ),
                        some_file: None,
                        error: ParseErrorType::MissingRequiredValue(SUBSKILL_SPEC.id.to_string()),
                    });
                }

                let (suberrors, subentity) = DYSkill::from_block_with_validation(subblock);
                skill.subskills.push(subentity);
                errors.extend(suberrors);
            }
        }
        (errors, skill)
    }
}

pub fn parse_skills(some_file: &Option<String>, content: &str) -> ParseResult<DYSkill> {
    parse_with_spec(
        &ValidDYSpec::new(SKILLS_SPEC).expect("TESTING_SKILLS_SPEC is invalid !"),
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

    use crate::skill::{DYSkill, parse_skills};

    use pretty_assertions::assert_eq;

    #[test]
    fn test_can_parse_simple_skills() {
        let text = "
// Great skills list
skill Classes
dir classes

skill Opérateurs
Maitriser l'usage de **tous les opérateurs utiles**, pour manipuler
des nombres, des bits, ou encore des flux. Redéfinir les opérateurs
existants pour nos classes.
dir ops
subskill Calculs simples
Juste des calculs de math !

subskill Calculs avancés et précision
subskill Opérateurs de flux

subskill Manipulation de bits
subskill Redéfinition d'opérateurs

";
        let some_file = &Some("skills.dy".to_string());
        assert_eq!(
            parse_skills(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: None,
        items: vec![
            DYSkill {
                name: "Classes".to_string(),
                directory: "classes".to_string(),
                description: "".to_string(),
                subskills: vec![],
            },
            DYSkill {
                name: "Opérateurs".to_string(),
                directory: "ops".to_string(),
                description: "Maitriser l'usage de **tous les opérateurs utiles**, pour manipuler\ndes nombres, des bits, ou encore des flux. Redéfinir les opérateurs\nexistants pour nos classes.".to_string(),
                subskills: vec![
                    DYSkill {
                        name: "Calculs simples".to_string(),
                        directory: "".to_string(),
                        description: "Juste des calculs de math !".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Calculs avancés et précision".to_string(),
                        directory: "".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Opérateurs de flux".to_string(),
                        directory: "".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Manipulation de bits".to_string(),
                        directory: "".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    },
                    DYSkill {
                        name: "Redéfinition d'opérateurs".to_string(),
                        directory: "".to_string(),
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

    #[test]
    fn test_can_detect_subskill_missing_value() {
        let text = "skill A
great desc
dir a
subskill


";
        let some_file = &Some("skills.dy".to_string());
        assert_eq!(
            parse_skills(some_file, text),
            ParseResult {
                some_file_path: some_file.clone(),
                some_file_content: Some(text.to_string()),
                items: vec![DYSkill {
                    name: "A".to_string(),
                    directory: "a".to_string(),
                    description: "great desc".to_string(),
                    subskills: vec![DYSkill {
                        name: "".to_string(),
                        directory: "".to_string(),
                        description: "".to_string(),
                        subskills: vec![],
                    }],
                }],
                errors: vec![ParseError {
                    range: range_on_line_part(3, 8, 8),
                    some_file: None,
                    error: ParseErrorType::MissingRequiredValue("subskill".to_string()),
                }]
            }
        )
    }
}
