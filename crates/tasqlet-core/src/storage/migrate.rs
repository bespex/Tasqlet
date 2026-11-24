//! Migrations of the data files between `schema` versions (`docs/KONZEPT.md` 2.5).
//!
//! A migration step works on the raw JSON, before it is read into the current types. To change
//! the format: raise [`SCHEMA_VERSION`](crate::SCHEMA_VERSION) and append one step to [`STEPS`]
//! that turns a file of the previous version into the new one.

use serde_json::Value;

use crate::SCHEMA_VERSION;

/// One migration: from version `n` to `n + 1`.
pub(crate) type Step = fn(Value) -> Result<Value, String>;

/// `STEPS[i]` migrates from version `i + 1` to `i + 2`. Empty while the format is at version 1.
const STEPS: &[Step] = &[];

const _: () = assert!(STEPS.len() + 1 == SCHEMA_VERSION as usize);

/// Why a file could not be brought to the current version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MigrationError {
    /// Written by a newer Tasqlet; reading it could lose data.
    TooNew(u64),
    /// `schema` is there but not a valid version, or a step failed.
    Invalid(String),
}

/// Bring a data file to the current version. A file without `schema` counts as version 1, so
/// hand-written files work. Returns the file and whether it was migrated.
pub(crate) fn migrate(value: Value) -> Result<(Value, bool), MigrationError> {
    migrate_with(value, STEPS, u64::from(SCHEMA_VERSION))
}

fn migrate_with(
    mut value: Value,
    steps: &[Step],
    current: u64,
) -> Result<(Value, bool), MigrationError> {
    let mut version = match value.get("schema") {
        None => 1,
        Some(v) => v
            .as_u64()
            .filter(|&v| v >= 1)
            .ok_or_else(|| MigrationError::Invalid(format!("invalid schema {v}")))?,
    };
    if version > current {
        return Err(MigrationError::TooNew(version));
    }
    let migrated = version < current;
    while version < current {
        let step = steps[(version - 1) as usize];
        value = step(value).map_err(MigrationError::Invalid)?;
        version += 1;
        if let Some(obj) = value.as_object_mut() {
            obj.insert("schema".into(), version.into());
        }
    }
    Ok((value, migrated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rename_items(mut v: Value) -> Result<Value, String> {
        let entries = v["entries"].take();
        v["items"] = entries;
        Ok(v)
    }

    fn add_flag(mut v: Value) -> Result<Value, String> {
        v["on_break"] = false.into();
        Ok(v)
    }

    #[test]
    fn steps_run_in_order_and_set_the_schema() {
        let old = json!({ "schema": 1, "entries": [1, 2] });
        let (new, migrated) = migrate_with(old, &[rename_items, add_flag], 3).unwrap();
        assert!(migrated);
        assert_eq!(
            new,
            json!({ "schema": 3, "items": [1, 2], "entries": null, "on_break": false })
        );
    }

    #[test]
    fn current_and_missing_schema_pass_unchanged() {
        let file = json!({ "schema": 1, "items": [] });
        assert_eq!(migrate(file.clone()).unwrap(), (file, false));
        let bare = json!({ "items": [] });
        assert_eq!(migrate(bare.clone()).unwrap(), (bare, false));
    }

    #[test]
    fn newer_or_invalid_schema_is_refused() {
        assert_eq!(
            migrate(json!({ "schema": 99 })),
            Err(MigrationError::TooNew(99))
        );
        assert!(matches!(
            migrate(json!({ "schema": 0 })),
            Err(MigrationError::Invalid(_))
        ));
        assert!(matches!(
            migrate(json!({ "schema": "1" })),
            Err(MigrationError::Invalid(_))
        ));
    }
}
