/// Common testing specs
#[cfg(test)]
pub mod tests {
    use crate::spec::{DYSpec, KeySpec, KeyType};

    pub const GOAL_SPEC: &KeySpec = &KeySpec {
        id: "goal",
        subkeys: &[],
        kt: KeyType::Multiline,
    };
    pub const CODE_SPEC: &KeySpec = &KeySpec {
        id: "code",
        subkeys: &[],
        kt: KeyType::SingleLine,
    };
    pub const COURSE_SPEC: &KeySpec = &KeySpec {
        id: "course",
        subkeys: &[CODE_SPEC, GOAL_SPEC],
        kt: KeyType::SingleLine,
    };
    pub const TESTING_COURSE_SPEC: &DYSpec = &[COURSE_SPEC];

    pub const SUBSKILL_SPEC: &KeySpec = &KeySpec {
        id: "subskill",
        subkeys: &[],
        kt: KeyType::Multiline,
    };
    pub const SKILL_SPEC: &KeySpec = &KeySpec {
        id: "skill",
        subkeys: &[SUBSKILL_SPEC],
        kt: KeyType::Multiline,
    };
    pub const TESTING_SKILLS_SPEC: &DYSpec = &[SKILL_SPEC];
}
