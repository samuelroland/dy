/// Common testing specs
#[cfg(test)]
pub mod tests {
    use crate::spec::{DYSpec, KeySpec, KeyType};

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

    pub const ARGS_SPEC: &KeySpec = &KeySpec {
        id: "args",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: true,
    };
    pub const SEE_SPEC: &KeySpec = &KeySpec {
        id: "see",
        subkeys: &[],
        kt: KeyType::Multiline,
        once: false,
    };
    pub const TYPE_SPEC: &KeySpec = &KeySpec {
        id: "type",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: false,
    };
    pub const EXIT_SPEC: &KeySpec = &KeySpec {
        id: "exit",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: true,
    };
    pub const CHECK_SPEC: &KeySpec = &KeySpec {
        id: "check",
        subkeys: &[ARGS_SPEC, SEE_SPEC, TYPE_SPEC, EXIT_SPEC],
        kt: KeyType::SingleLine,
        once: false,
    };
    pub const EXO_SPEC: &KeySpec = &KeySpec {
        id: "exo",
        subkeys: &[CHECK_SPEC],
        kt: KeyType::Multiline,
        once: true, // for now, only one exo per file
    };
    pub const TESTING_EXOS_SPEC: &DYSpec = &[EXO_SPEC];
}
