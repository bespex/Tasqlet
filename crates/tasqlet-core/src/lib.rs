//! Domain logic of Tasqlet, free of any GUI or desktop dependency.
//!
//! The specification lives in `docs/KONZEPT.md`; `docs/PLAN.md` says which part is built when.
//! This crate will hold the item model and its transitions (plan step 1.1), storage (1.2), the
//! quick-entry parser and smart labels (1.3), and themes with their contrast check (2.6).

/// Version of the data file format (`schema` in `items.json`, see `docs/KONZEPT.md` 2.2).
pub const SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_starts_at_one() {
        assert_eq!(SCHEMA_VERSION, 1);
    }
}
