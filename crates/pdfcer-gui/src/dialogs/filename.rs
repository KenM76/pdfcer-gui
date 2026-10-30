//! # `dialogs::filename` — a path's file name for a sentence that names a file
//!
//! Compiled in every build, so dialogs behind no feature never borrow it from
//! one behind a feature (`sign` is absent from Android builds).

use std::path::Path;

/// The path's last component, or the whole path when it has none.
pub(crate) fn file_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}
