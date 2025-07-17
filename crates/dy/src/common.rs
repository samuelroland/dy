/// Common testing specs
/// They are very near the PLX spec because that's useful to test with real hierarchy and it's
/// easier to represent mentally than invented random structures where it's not clear which key is under which other one.
///
/// It doesn't mean it's up-to-date with the PLX spec though... There is no need to keep it up-to-date.
#[cfg(test)]
pub mod tests {
    use crate::spec::{DYSpec, KeySpec, KeyType};

    pub const GOAL_SPEC: &KeySpec = &KeySpec {
        id: "goal",
        desc: "test",
        subkeys: &[],
        kt: KeyType::Multiline,
        once: true,
        required: true,
    };
    pub const CODE_SPEC: &KeySpec = &KeySpec {
        id: "code",
        desc: "test",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: true,
        required: true,
    };
    pub const COURSE_SPEC: &KeySpec = &KeySpec {
        id: "course",
        desc: "test",
        subkeys: &[CODE_SPEC, GOAL_SPEC],
        kt: KeyType::SingleLine,
        once: true,
        required: true,
    };
    pub const TESTING_COURSE_SPEC: &DYSpec = &[COURSE_SPEC];

    pub const SUBSKILL_SPEC: &KeySpec = &KeySpec {
        id: "subskill",
        desc: "test",
        subkeys: &[],
        kt: KeyType::Multiline,
        once: false,
        required: false,
    };
    pub const SKILL_SPEC: &KeySpec = &KeySpec {
        id: "skill",
        desc: "test",
        subkeys: &[SUBSKILL_SPEC],
        kt: KeyType::Multiline,
        once: false,
        required: true,
    };
    pub const TESTING_SKILLS_SPEC: &DYSpec = &[SKILL_SPEC];

    pub const ARGS_SPEC: &KeySpec = &KeySpec {
        id: "args",
        desc: "test",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: true,
        required: false,
    };
    pub const SEE_SPEC: &KeySpec = &KeySpec {
        id: "see",
        desc: "test",
        subkeys: &[],
        kt: KeyType::Multiline,
        once: false,
        required: true,
    };
    pub const TYPE_SPEC: &KeySpec = &KeySpec {
        id: "type",
        desc: "test",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: false,
        required: false,
    };
    pub const EXIT_SPEC: &KeySpec = &KeySpec {
        id: "exit",
        desc: "test",
        subkeys: &[],
        kt: KeyType::SingleLine,
        once: true,
        required: false,
    };
    pub const CHECK_SPEC: &KeySpec = &KeySpec {
        id: "check",
        desc: "test",
        subkeys: &[ARGS_SPEC, SEE_SPEC, TYPE_SPEC, EXIT_SPEC],
        kt: KeyType::SingleLine,
        once: false,
        required: true,
    };
    pub const EXO_SPEC: &KeySpec = &KeySpec {
        id: "exo",
        desc: "test",
        subkeys: &[CHECK_SPEC],
        kt: KeyType::Multiline,
        once: true, // for now, only one exo per file
        required: true,
    };
    pub const TESTING_EXOS_SPEC: &DYSpec = &[EXO_SPEC];
}
