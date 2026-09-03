//! # `dialogs::settings::comments` — who signs the comments you write
//!
//! One control, and a module for it because the *placement* of that control is
//! the part with an argument.
//!
//! ## ★★★ Why it is not in Appearance, Drawing, or Saving
//!
//! The window's ordering rule is stated in [`super`]: the groups run from what
//! the **program** looks like, through what the **document** is made of, to
//! what pdfcer **does with it**. A name is none of those — it is a fact about
//! the **person**, and it is the first of its kind in this window.
//!
//! It sits beside Measuring, which is the other group about *authoring* rather
//! than about reading, and immediately before Pages. An operator who has come
//! looking for it has come from writing a comment, and Measuring is the
//! nearest heading to that act.
//!
//! ## ★★ It is a preference, and it is the only one filed with the document groups
//!
//! Every other `Prefs` field lives in **Drawing the page**, at the end, under
//! that group's own note about *"the only group here whose values live in a
//! different file"*. This one does not, and the exception is deliberate: the
//! file a value lives in is an implementation detail, and the heading an
//! operator scans for is *"the thing I was doing"*. Filing a name under
//! *Drawing the page* because of where it is stored would be organising the
//! window by our filing system rather than by their question.
//!
//! ## ★ Empty is a valid, supported answer
//!
//! Blank leaves comments anonymous, which is legal, is what pdfcer did before
//! `Pass 150.0`, and is what an operator sending a drawing outside their firm
//! may actually want. There is no warning, no asterisk and no placeholder
//! guessed from the Windows login — see `app::prefs::Prefs::author_name`.

use egui::Ui;

use super::widgets;
use crate::text::settings as t;

/// The name written into `/T` on every comment this shell authors.
///
/// ★ `text_value` with an identity parse, which is the honest shape for a free
/// string: the helper exists to hold a half-typed *number* apart from a parsed
/// value, and a name has no invalid intermediate state. `Some(..)` on every
/// input means every keystroke reaches the draft, so Save writes exactly what
/// is on screen.
pub fn author_name(ui: &mut Ui, prefs: &mut crate::app::prefs::Prefs) {
    widgets::header(
        ui,
        t::author_name_title(),
        t::author_name_silence(),
        t::author_name_radius(),
    );
    widgets::text_value(
        ui,
        // ui-text-exempt: an egui control id, never displayed.
        "settings-author-name",
        &mut prefs.author_name,
        t::author_name_label(),
        Some(t::author_name_note()),
        Clone::clone,
        |typed| Some(typed.to_owned()),
    );
}
