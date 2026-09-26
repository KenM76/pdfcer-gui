//! # `text::menus` — the copy the context-menu surface owns
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! the four context menus in [`crate::shell::menus`] and the right-click
//! wiring in [`crate::canvas::menus`] and [`crate::panels`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/menus.md`.

#[cfg(test)]
mod tests {
    use crate::shell::menus;
    use egui_shell::manifest::Item;

    /// **The menu surface owns no copy of its own — asserted, not
    /// assumed.**
    #[test]
    fn the_menu_surface_owns_no_copy_of_its_own() {
        for menu in menus::built_in().iter() {
            for item in menu.items() {
                match item {
                    // A command carries an id; its words are the registry's.
                    Item::Command { .. } => {}
                    // Punctuation. No words.
                    Item::Separator => {}
                    Item::Custom { kind, .. } => panic!(
                        // ui-text-exempt: a test panic, read by whoever is looking at
                        // the failure. Never rendered to an operator.
                        "menu `{}` holds a custom row `{kind}`, which the application draws \
                         itself — so it has words, and they belong in `text::menus` rather \
                         than at the call site. This module is empty only while every menu \
                         item is a command reference.",
                        menu.context
                    ),
                }
            }
        }
    }
}
