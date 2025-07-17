/// Core types to define a DY specification, that is the description of the structure of a file to parse
use std::{collections::HashSet, fmt::Debug};

/// The specification of a key
#[derive(Hash, Eq, PartialEq)]
pub struct KeySpec<'a> {
    /// The id of the key, its string representation, like "exo", "course", "code", ...
    pub id: &'a str,
    /// The description of this key, meant to be shown by the spec documentation and the language server
    pub desc: &'a str,
    /// The list of keys that can be defined under this keyspec that are children of the current key
    /// and that cannot be used without this parent key
    pub subkeys: &'a DYSpec<'a>,
    /// The type of this key, impacting the way
    pub vt: ValueType,

    /// Whether this key is only permitted once for its parent object
    ///
    /// For example, in PLX the `course` key is only acceptable once in a `course.dy` file as we only want
    /// to define a single course. For a skill, the `dir` key is only meaningful when given
    /// once. For a check, the `type` key is totally okay since we want to type different things
    /// as the sequence of the check.
    pub once: bool,
    /// Whether this key is required to be present and have non empty value
    /// It that's not the case, it will generate MissingRequiredValue
    pub required: bool,
}

impl<'a> Debug for KeySpec<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeySpec '{}'", self.id)
    }
}

impl<'a> KeySpec<'a> {
    pub fn is_entity(&self) -> bool {
        !self.subkeys.is_empty()
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum ValueType {
    SingleLine,
    Multiline,
}

/// The specification is just a list of keys that are valid at the current level
pub type DYSpec<'a> = [&'a KeySpec<'a>];

/// Wrapper type of DYSpec, to validate the spec semantically
#[derive(Debug, Eq, PartialEq)]
pub struct ValidDYSpec<'a>(&'a DYSpec<'a>);

/// Extract a flat vector of key specs to tokenize lines
pub fn all_valid_keys<'a>(spec: &'a DYSpec<'a>) -> Vec<&'a KeySpec<'a>> {
    let mut all_keys = spec.to_vec();
    all_keys.extend(spec.iter().flat_map(|k| all_valid_keys(k.subkeys)));
    all_keys
}

impl<'a> ValidDYSpec<'a> {
    pub fn new(spec: &'a DYSpec) -> Result<Self, String> {
        let mut keys: HashSet<&str> = HashSet::new();
        if spec.is_empty() {
            return Err("The spec cannot be empty".to_string());
        }
        Self::spec_does_not_contain_known_keys(&mut keys, spec)?;
        Ok(ValidDYSpec(spec))
    }

    pub fn get(&'a self) -> &'a DYSpec<'a> {
        self.0
    }

    fn spec_does_not_contain_known_keys(
        known_keys: &mut HashSet<&'a str>,
        spec: &'a DYSpec,
    ) -> Result<(), String> {
        for key_spec in spec {
            if known_keys.contains(key_spec.id) {
                return Err(format!("Duplicated key identifier '{}'", key_spec.id));
            } else {
                known_keys.insert(key_spec.id);
            }
            // Search recursively in subkeys
            if !key_spec.subkeys.is_empty() {
                Self::spec_does_not_contain_known_keys(known_keys, key_spec.subkeys)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::tests::{CODE_SPEC, GOAL_SPEC, TESTING_COURSE_SPEC};
    use crate::spec::{KeySpec, ValidDYSpec, ValueType};

    #[test]
    fn test_can_validate_valid_spec() {
        assert_eq!(
            ValidDYSpec::new(TESTING_COURSE_SPEC),
            Ok(ValidDYSpec(TESTING_COURSE_SPEC))
        );
    }

    #[test]
    fn test_empty_spec_is_invalid() {
        assert!(ValidDYSpec::new(&[]).unwrap_err().contains("empty"));
    }

    #[test]
    fn test_spec_with_duplicated_key_at_root() {
        assert!(
            ValidDYSpec::new(&[CODE_SPEC, GOAL_SPEC, CODE_SPEC])
                .unwrap_err()
                .contains("Duplicated key identifier 'code'")
        );
    }

    #[test]
    fn test_spec_with_duplicated_key_deeply() {
        assert!(
            ValidDYSpec::new(&[
                GOAL_SPEC,
                &KeySpec {
                    desc: "test",
                    id: "course",
                    subkeys: &[CODE_SPEC, GOAL_SPEC],
                    vt: ValueType::SingleLine,
                    once: true,
                    required: true,
                }
            ])
            .unwrap_err()
            .contains("Duplicated key identifier 'goal'")
        );
    }
}
