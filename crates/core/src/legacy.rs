//! The product name used before 0.16.4, kept only to read data written by older versions:
//! the settings folder (copied to the new location on first start), the web app's saved
//! preferences and the LaTeX source prefix in pictures of saved workbooks. Nothing is written
//! under this name any more.

/// The old product name. This is the only remaining use of it in the code; derive the other
/// spellings (`lower()`) from it instead of repeating it.
pub const NAME: &str = "GridCraft";

/// The lower-case form of [`NAME`], as used in file names, keys and prefixes.
pub fn lower() -> String {
    NAME.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    #[test]
    fn lower_is_ascii_lower_case() {
        let l = super::lower();
        assert_eq!(l.len(), super::NAME.len());
        assert!(l.chars().all(|c| c.is_ascii_lowercase()));
    }
}
