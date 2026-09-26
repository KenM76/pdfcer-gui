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

    /// **★ The menu surface owns no copy of its own — asserted, not
    /// assumed.**
    ///
    /// This module's emptiness is a *consequence* of every menu item being a
    /// command reference, and that consequence has a precise failure mode:
    /// an `Item::Custom` row is drawn by the application, so its words come
    /// from the application, and there is no other honest place for them
    /// than this file. A separator has no words either, so it is allowed —
    /// it is punctuation.
    ///
    /// If this fails, the fix is **not** to delete the test. It is to write
    /// the string into this module and hand it to whatever renders the
    /// custom row, which is the sequence the whole catalog rule exists to
    /// force.
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
