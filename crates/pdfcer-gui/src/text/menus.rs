//! # `text::menus` — the copy the context-menu surface owns
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! the four context menus in [`crate::shell::menus`] and the right-click
//! wiring in [`crate::canvas::menus`] and [`crate::panels`].
//!
//! It holds no copy while every menu item is a command reference; the test
//! asserting that is `shell::menus::tests::the_menu_surface_owns_no_copy_of_its_own`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/menus.md`.
