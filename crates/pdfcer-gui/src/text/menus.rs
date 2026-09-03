//! # `text::menus` — the copy the context-menu surface owns
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! the four context menus in [`crate::shell::menus`] and the right-click
//! wiring in [`crate::canvas::menus`] and [`crate::panels`].
//!
//! ## ★ It is empty, and the emptiness is the design working
//!
//! Not an oversight and not a file created ahead of its contents. A context
//! menu has **no words of its own**. `RIBBON_IA.md` §5.8:
//!
//! > A third surface, the **context menu**, carries the same commands again
//! > for the user who right-clicks. That is not duplication in the P1 sense
//! > — context menus are not tabs.
//!
//! *The same commands* is the whole point, and it is enforced by types
//! rather than by care: a menu item is `egui_shell::manifest::Item`, which
//! carries a command **id** and nothing else, and the row's label and
//! tooltip are looked up in the same `CommandRegistry` the ribbon draws
//! from — so they come from [`crate::text::commands`]. A `Delete` written
//! down here would be a second `Delete`, and the day one of the two was
//! reworded the ribbon and the right-click would disagree about what the
//! same command is called.
//!
//! The other three sources of text in a menu row are the same way:
//!
//! | Part of a row | Comes from |
//! |---|---|
//! | the label | the command's, via [`crate::text::commands`] |
//! | the hover tooltip — including the one a **greyed** row shows, which is what makes P3's *"always explained on hover"* true here | the command's, same source |
//! | the right-aligned chord hint | the manifest's **keymap**, inverted by `egui_shell::menu::Shortcuts` — never a string, so an operator who rebinds a key sees the menu follow |
//! | the accessible name announced to a screen reader | assembled by `egui_shell::menu::a11y` from the two above |
//!
//! And a menu has no caption, because *"a context menu's caption is the
//! thing you right-clicked"*. There is nothing left for this module to
//! hold.
//!
//! ## Why the module exists at all, then
//!
//! Three reasons, and the first is mechanical.
//!
//! 1. **`tools/gates/check-ui-strings.sh` needs a home to point at.** The
//!    gate fails the build on a whitespace-bearing literal anywhere outside
//!    the catalog. The moment the menu surface grows one line of copy, the
//!    author needs somewhere obvious to put it that is not
//!    `text::commands` (which is *commands*, not menus) and not
//!    `text::panels` (which is *panel bodies*). A module that already
//!    exists is a decision nobody has to make under time pressure, and the
//!    alternative — inventing the file at that moment — is how a literal
//!    ends up inlined "just for now".
//! 2. **The emptiness is a claim worth testing.** It is only true while
//!    every menu item is a `Command`. A `Custom` item — a colour swatch on
//!    a markup's menu, say, which is exactly the shape
//!    `manifest::markup`'s Style band already uses on the ribbon — is drawn
//!    by the application and *would* carry its own words.
//!    [`tests::the_menu_surface_owns_no_copy_of_its_own`] is what turns the
//!    paragraph above from an assertion into a check.
//! 3. **Absence is documented as data in this project.** Same discipline as
//!    `shell::manifest::PLANNED`: the next person to read this should be
//!    able to tell *considered and unnecessary* from *never noticed*.
//!
//! ## What would land here
//!
//! Written down so the first person to need it does not have to re-derive
//! whether it belongs. Each is a string a **menu** owns rather than a
//! command:
//!
//! - **A submenu's caption.** `egui-shell` deliberately implements no
//!   submenus today (`menu::render`'s header: a nested menu is a second
//!   popup, a hover-intent timer and a keyboard model of its own, and none
//!   of `RIBBON_IA.md` §6's menus needs one). A submenu is a row that names
//!   a *group* rather than a command, so its caption would be the first
//!   genuine entry here.
//! - **The label of a `Custom` row** — an in-menu colour swatch, a recent
//!   list, a set of preset scales.
//! - **A heading or a hint inside a menu**, if one ever earns its place.
//!   The bar is high: a menu is a list of verbs, and a sentence in one is
//!   read by nobody in a hurry.
//!
//! What would **not** land here, so the boundary stays sharp: anything a
//! command already says. If a command's label reads wrongly in a menu, the
//! fix is the command's label — the ribbon has the same problem and has not
//! noticed.

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
