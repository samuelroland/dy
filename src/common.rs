/// Common testing specs
#[cfg(test)]
pub mod tests {
    use crate::spec::{DYSpec, KeySpec, KeyType};

    pub const GOAL_SPEC: &KeySpec = &KeySpec {
        id: "goal",
        subkeys: &[],
    };
    pub const CODE_SPEC: &KeySpec = &KeySpec {
        id: "code",
        subkeys: &[],
    };
    pub const COURSE_SPEC: &KeySpec = &KeySpec {
        id: "course",
        subkeys: &[CODE_SPEC, GOAL_SPEC],
    };
    pub const PLX_COURSE_SPEC: &DYSpec = &[COURSE_SPEC];
}
