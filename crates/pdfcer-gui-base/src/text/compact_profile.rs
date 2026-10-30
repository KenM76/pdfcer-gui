//! # `text::compact_profile` — the compact window's settings files
//!
//! Every word of the Load settings… and Save settings… row in Save a
//! compacted copy.

/// Load settings… button.
#[must_use]
pub const fn load_button() -> &'static str {
    "Load settings…"
}

/// Hover on Load settings….
#[must_use]
pub const fn load_tooltip() -> &'static str {
    "Use the settings saved in a file, in place of the starting point, switches and size \
     target below. The file is the one pdfcer optimize --profile reads."
}

/// Save settings… button.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save settings…"
}

/// Hover on Save settings….
#[must_use]
pub const fn save_tooltip() -> &'static str {
    "Save what this window would do to a file, to load here again or to pass to \
     pdfcer optimize --profile. Cutting fonts to the glyphs drawn and the size target are \
     not part of a settings file."
}

/// Stop using button, beside a loaded file's name.
#[must_use]
pub const fn stop_button() -> &'static str {
    "Stop using"
}

/// The picker's title when loading.
#[must_use]
pub const fn load_title() -> &'static str {
    "Load compact settings"
}

/// The picker's title when saving.
#[must_use]
pub const fn save_title() -> &'static str {
    "Save compact settings"
}

/// The picker's filter name.
#[must_use]
pub const fn filter() -> &'static str {
    "Settings files"
}

/// The name a new settings file is offered.
#[must_use]
pub const fn suggested_name() -> &'static str {
    "compact settings.toml"
}

/// The line while a file is in use.
#[must_use]
pub fn in_use(name: &str) -> String {
    format!(
        "Using the settings in {name}. The starting point, switches and size target are set \
         aside while it is in use."
    )
}

/// The line after a save.
#[must_use]
pub fn saved(name: &str) -> String {
    format!("Saved these settings to {name}.")
}

/// A file that could not be read or written: its path, then the system's reason.
#[must_use]
pub fn file_problem(path: &str, why: &str) -> String {
    format!("{path}: {why}")
}

/// The line when a file could not be read or written.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("The settings file could not be used: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_names_its_file() {
        assert!(in_use("mine.toml").contains("mine.toml"));
        assert!(saved("mine.toml").contains("mine.toml"));
        assert!(failed("line 3").contains("line 3"));
    }
}
