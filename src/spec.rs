/// Core types to define a DY specification, that is the description of the structure of a file to parse
use std::collections::HashSet;

/// The specification of a key
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct KeySpec<'a> {
    /// The id of the key, its string representation, like "exo", "course", "code", ...
    pub id: &'a str,
    /// The list of keys that can be defined under this keyspec that are children of the current key
    /// and that cannot be used without this parent key
    pub subkeys: &'a DYSpec<'a>,
}

/// The specification is just a list of keys that are valid at the current level
pub type DYSpec<'a> = [&'a KeySpec<'a>];

/// Wrapper type of DYSpec, to validate the spec semantically
#[derive(Debug, Eq, PartialEq)]
pub struct ValidDYSpec<'a>(&'a DYSpec<'a>);

impl<'a> ValidDYSpec<'a> {
    pub fn new(spec: &'a DYSpec) -> Result<Self, String> {
        let mut keys: HashSet<&str> = HashSet::new();
        if spec.is_empty() {
            return Err("The spec cannot be empty".to_string());
        }
        Self::spec_does_not_contain_known_keys(&mut keys, spec)?;
        Ok(ValidDYSpec(spec))
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
    use crate::common::tests::{CODE_SPEC, GOAL_SPEC, PLX_COURSE_SPEC};
    use crate::spec::{KeySpec, KeyType, ValidDYSpec};

    #[test]
    fn test_can_validate_valid_spec() {
        assert_eq!(
            ValidDYSpec::new(PLX_COURSE_SPEC),
            Ok(ValidDYSpec(PLX_COURSE_SPEC))
        );
    }

    #[test]
    fn test_empty_spec_is_invalid() {
        assert!(ValidDYSpec::new(&[]).unwrap_err().contains("empty"));
    }

    #[test]
    fn test_spec_with_duplicated_key_at_root() {
        assert!(ValidDYSpec::new(&[CODE_SPEC, GOAL_SPEC, CODE_SPEC])
            .unwrap_err()
            .contains("Duplicated key identifier 'code'"));
    }

    #[test]
    fn test_spec_with_duplicated_key_deeply() {
        assert!(ValidDYSpec::new(&[
            GOAL_SPEC,
            &KeySpec {
                id: "course",
                subkeys: &[CODE_SPEC, GOAL_SPEC],
            }
        ])
        .unwrap_err()
        .contains("Duplicated key identifier 'goal'"));
    }
}
