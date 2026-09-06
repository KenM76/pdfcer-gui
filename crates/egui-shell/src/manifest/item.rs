//! # `manifest::item` — what one entry in a group is, and how much room it asks
//! for
//!
//! Split out of [`super`] under **R2** on 2026-09-06, when
//! `SHELL_FRAMEWORK.md` §5b's conditional-item field
//! ([`Item::Command::capability`]) took `manifest/mod.rs` past the 1,500-line
//! ceiling. **The seam was already drawn in the source**: [`ItemSize`] and
//! [`Item`] are the only two types in that file that describe *a control in a
//! band* rather than *a region of the shell* — [`super::Tab`],
//! [`super::Group`], [`super::Qat`], [`super::Trailing`] and
//! [`super::Keymap`] are all containers, and these two are the thing they
//! contain.
//!
//! ★ Both are re-exported from [`super`], so **no call site moved**. The
//! ceiling exists to stop a file becoming two subjects, not to make callers
//! learn where a type sleeps.
//!
//! # The three questions an item answers, and they are different questions
//!
//! | field | question | when it is answered |
//! |---|---|---|
//! | [`ItemSize`] | how much room does this ask for? | at layout, every band re-measure |
//! | `visible_when` | should it be drawn right now? | every frame — this document, this mode, this selection |
//! | `capability` | is it in this build at all? | once, at start-up, and it cannot change while the program runs |
//!
//! ★★★ The last two are the pair worth keeping apart, and the reason is R9:
//! *an unavailable capability renders nothing; greying is reserved for
//! **temporarily** unavailable and is always explained on hover.* A
//! per-frame condition standing in for a link-time fact would be asking a
//! question sixty times a second whose answer was fixed when the binary was
//! built.

use serde::{Deserialize, Serialize};

/// **How much room a control asks for, and how much of itself it shows.**
///
/// `RIBBON_SCALING.md` §5.1, learned by photographing Word at twelve widths.
/// Word has exactly three sizes and a group mixes them freely — one Large
/// button beside a column of three Small ones is its Clipboard group — and
/// that mixing is where its density comes from. Measured: at 884 client points
/// Word puts **ten** groups on the band and this shell put **three**, because
/// every control here was Medium and nothing could be narrower.
///
/// ★ [`Self::Medium`] is the default **and is exactly the presentation this
/// shell had before sizes existed**, so a manifest that says nothing renders
/// identically. That is what makes the vocabulary safe to introduce in one
/// change rather than behind a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ItemSize {
    /// Icon, gap, label, on one row. The default, and every control's
    /// presentation before `RIBBON_SCALING.md`.
    #[default]
    Medium,
    /// **Icon only.**
    ///
    /// ★★ Earned, not asserted. A control renders icon-only only when it names
    /// an icon, carries a tooltip **and** a painter is installed — the rule
    /// [`crate::ribbon::qat`] already applies to the quick-access toolbar,
    /// applied here unchanged. The tooltip is the icon's accessible name;
    /// without one an icon-only button is an unlabelled rectangle to a screen
    /// reader and a guess to everybody else. A `Small` that has not earned it
    /// **falls back to `Medium`** rather than rendering a mystery.
    Small,
    /// **Icon above the label**, spanning the band's rows.
    ///
    /// The group's headline verb — Word's Paste, Dictate, Editor. Its width is
    /// the wider of its icon and its label, so a long label makes a wide
    /// button.
    Large,
}

impl ItemSize {
    /// Whether this is the default, for `skip_serializing_if`.
    ///
    /// Keeps `Command(id: "file.open")` in the manifest file rather than
    /// `Command(id: "file.open", size: Medium)` on every one of a hundred
    /// lines. The on-disk manifest is meant to be read and edited by an
    /// operator, and a field that is the default everywhere is noise that
    /// hides the two places it is not.
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Medium)
    }
}

/// One entry in a group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// A command, by id. The id is resolved against the registry; the manifest
    /// carries nothing else **about the command** — no label, no icon, no
    /// tooltip. What it does carry is how the command is *presented here*,
    /// which is a property of this position on this tab rather than of the
    /// command, and is therefore the manifest's to state.
    Command {
        /// The registered command id.
        id: String,
        /// How much room it asks for. See [`ItemSize`].
        ///
        /// ★ Ignored by menus, which have one row shape and no use for a
        /// size. `Item` is the shared vocabulary for ribbon groups and menus
        /// both; the alternative — a second item type for menus — would
        /// duplicate `visible_when`, which they genuinely do share.
        #[serde(default, skip_serializing_if = "ItemSize::is_default")]
        size: ItemSize,
        /// A condition name. When set, the item is drawn **only** while the
        /// condition holds — and when it is not, its space is reclaimed
        /// **before measurement**, so the group re-flows and a group with
        /// nothing left is not drawn at all.
        ///
        /// ★★★ This is visibility, not enablement, and the difference is R9:
        /// *an unavailable capability renders nothing; greying is reserved for
        /// **temporarily** unavailable and is always explained on hover.*
        /// [`crate::commands::Command::enable`] is the greying; this is the
        /// disappearing.
        ///
        /// It is what lets one tab definition serve Read, Review and Edit with
        /// different contents rather than three near-identical tabs —
        /// `RIBBON_SCALING.md` §5.3.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visible_when: Option<String>,
        /// **The name of a capability this item is CONDITIONAL on**, when the
        /// application can be built without it.
        ///
        /// ★★★ THIS IS NOT `visible_when`, AND CONFUSING THE TWO IS THE WHOLE
        /// REASON IT IS A SECOND FIELD.
        ///
        /// `visible_when` is about **this document, this mode, this moment** —
        /// it is re-evaluated every frame and its answer changes while the
        /// application runs. This is about **this build**, it is answered once
        /// at start-up, and its answer can never change without recompiling or
        /// (eventually) dropping a different DLL beside the executable. A
        /// condition evaluated per frame for a fact that is fixed at link time
        /// would be a per-frame lie about what kind of question is being asked.
        ///
        /// # What it means, in one table
        ///
        /// `SHELL_FRAMEWORK.md` §5b, which specified this field and left it as
        /// *"the gap that must be closed"* until 2026-09-06:
        ///
        /// | item | command registered? | result |
        /// |---|---|---|
        /// | **mandatory** (this field is `None`) | no | **hard validation failure** — a programming error, unchanged |
        /// | **conditional** (this field is `Some`) | no | dropped, [`merge::SkipReason::CapabilityAbsent`] — informational |
        /// | either | yes | rendered |
        ///
        /// ★★ The distinction exists so the two never get confused **in a
        /// log**: one says *"this build does not include that"*, the other
        /// says *"someone made a mistake"*. Without it, modularity and a typo
        /// are the same event, and the only ways to handle them are to block
        /// start-up on a legitimate lite build or to swallow a real bug on
        /// every machine that runs it.
        ///
        /// # ★★★ It carries no meaning to the shell beyond its presence
        ///
        /// The string is **never matched against anything**. The shell does
        /// not hold a list of known capability names, does not ask an
        /// application whether `"signing"` is available, and has no way to
        /// find out. The only question it asks is the one it already asked:
        /// *is a command with this id in the registry?* The name is carried
        /// so the **skip report** can say which capability was absent, and for
        /// nothing else.
        ///
        /// That is what keeps `SHELL_FRAMEWORK.md` §5b's one rule true — *a
        /// capability's presence is expressed by registering its command, and
        /// by nothing else*. A field the shell interpreted would be a second
        /// place that knows, and the exe→DLL move would stop being a swap.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        capability: Option<String>,
    },
    /// A vertical rule between neighbours. Presentation only.
    Separator,
    /// Something the application draws itself.
    ///
    /// The extension point for controls that are not a button: a colour
    /// swatch, a zoom slider, a scale picker, a split button with a
    /// gallery. The shell reserves the space and hands `kind` and
    /// `payload` back; it draws nothing and interprets neither.
    ///
    /// This is what keeps the item vocabulary from growing a variant per
    /// widget an application happens to want — which is the road by which
    /// a reusable shell acquires a `ColourSwatch` variant and stops being
    /// reusable.
    Custom {
        /// An application-defined kind, e.g. `"colour_swatch"`.
        kind: String,
        /// Optional application-defined payload.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<String>,
        /// A condition name, with exactly the meaning it has on
        /// [`Self::Command`] — the item is drawn only while the condition
        /// holds, and its space is reclaimed **before measurement** when it
        /// does not.
        ///
        /// # ★★ Why this is a second copy of the field rather than a wrapper
        ///
        /// [`Self::visible_condition`] used to close with a standing
        /// instruction, and it is quoted rather than deleted because the
        /// reasoning is sound and the decision to depart from it has to be
        /// argued rather than assumed:
        ///
        /// > A separator and a custom item cannot carry one yet; when one
        /// > needs to, the field moves onto a **wrapper** rather than being
        /// > copied into three variants, because three copies of a rule is
        /// > three chances for it to drift.
        ///
        /// The need arrived on 2026-08-27: pdfcer's Format tab carries a Font
        /// group whose face chooser, size field and colour swatch are all
        /// custom items, and the whole group must be **absent** in a mode that
        /// cannot edit page content — R9's rule that an unavailable
        /// *capability* renders nothing while a temporarily unavailable one
        /// greys. Without this field, three of that group's seven controls
        /// would draw in Read mode and the application would have to fake
        /// their absence by drawing nothing into a slot the band had already
        /// reserved, which leaves a hole rather than reflowing the group.
        ///
        /// **What makes the copy safe is that the rule was never in the
        /// field.** It is in [`Self::visible_condition`] — one accessor, one
        /// match, read by exactly one predicate
        /// (`crate::ribbon::sizing::visible`). Two variants declaring a
        /// `visible_when` produce two serde attributes and two arms of that
        /// one match; they do not produce two statements of *when an item is
        /// drawn*. The drift the old note feared is drift in the **rule**, and
        /// the rule stayed single.
        ///
        /// **What would still justify the wrapper**, and this is the trigger
        /// to watch for: a *second* per-position property — an `enabled_when`,
        /// a `label_override`, an `order` — or a `Separator` that needs to
        /// disappear with its neighbours. At that point the fields stop being
        /// one field on two variants and become a *record*, and a record
        /// belongs beside the item rather than inside it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        visible_when: Option<String>,
    },
}

impl Item {
    /// A command item, at the default size, always visible.
    ///
    /// The terse constructor, and the one nearly every call site wants: the
    /// manifest names a command and says nothing else about it.
    #[must_use]
    pub fn command(id: impl Into<String>) -> Self {
        Item::Command {
            id: id.into(),
            size: ItemSize::default(),
            visible_when: None,
            capability: None,
        }
    }

    /// The same item, at a different size.
    ///
    /// A builder rather than a second constructor, so the common form above
    /// stays the short one and a sized item reads as *"this command, but
    /// large"* — which is what it is.
    ///
    /// ★ A separator and a custom item have no size to set, and this returns
    /// them untouched rather than panicking. A manifest is **data**, and the
    /// honest response to nonsense in data is that it does nothing, not that
    /// the application stops.
    #[must_use]
    pub fn sized(self, size: ItemSize) -> Self {
        match self {
            Item::Command {
                id,
                visible_when,
                capability,
                ..
            } => Item::Command {
                id,
                size,
                visible_when,
                capability,
            },
            other => other,
        }
    }

    /// **The same item, present only in a build that registered its command.**
    ///
    /// The builder for [`Item::Command::capability`] — read that field's
    /// documentation before using this, because the distinction from
    /// [`Self::shown_when`] is the whole of it: this is about the *build*, that
    /// is about the *frame*.
    ///
    /// Named `provided_by` rather than `when_available` or `requires` because
    /// the sentence it makes at a call site is the true one: `file.sign`
    /// **is provided by** the `signing` capability. `requires` would read as a
    /// precondition the shell checks, and the shell checks nothing — see the
    /// field's *"carries no meaning beyond its presence"*.
    ///
    /// ★ A separator and a custom item are returned untouched, for
    /// [`Self::sized`]'s reason: a manifest is data, and the honest response to
    /// nonsense in data is that it does nothing. A custom item drawn by the
    /// application is the application's to omit; it has no command id, so there
    /// is nothing here that could be absent from a registry.
    #[must_use]
    pub fn provided_by(self, capability: impl Into<String>) -> Self {
        match self {
            Item::Command {
                id,
                size,
                visible_when,
                ..
            } => Item::Command {
                id,
                size,
                visible_when,
                capability: Some(capability.into()),
            },
            other => other,
        }
    }

    /// The same item, drawn only while `condition` holds.
    #[must_use]
    pub fn shown_when(self, condition: impl Into<String>) -> Self {
        match self {
            Item::Command {
                id,
                size,
                capability,
                ..
            } => Item::Command {
                id,
                size,
                visible_when: Some(condition.into()),
                capability,
            },
            // ★ A custom item takes one too, since 2026-08-27. A separator
            // still does not and returns untouched, for the reason
            // [`Self::visible_condition`] gives: a divider's visibility is a
            // fact about its neighbours, not about itself.
            Item::Custom { kind, payload, .. } => Item::Custom {
                kind,
                payload,
                visible_when: Some(condition.into()),
            },
            other => other,
        }
    }

    /// A custom item with no payload.
    #[must_use]
    pub fn custom(kind: impl Into<String>) -> Self {
        Item::Custom {
            kind: kind.into(),
            payload: None,
            visible_when: None,
        }
    }

    /// The command id, if this item is a command.
    #[must_use]
    pub fn command_id(&self) -> Option<&str> {
        match self {
            Item::Command { id, .. } => Some(id),
            Item::Separator | Item::Custom { .. } => None,
        }
    }

    /// How much room this item asks for. A separator and a custom item have
    /// one presentation each and report the default.
    #[must_use]
    pub fn size(&self) -> ItemSize {
        match self {
            Item::Command { size, .. } => *size,
            Item::Separator | Item::Custom { .. } => ItemSize::default(),
        }
    }

    /// The condition this item is shown under, if any.
    ///
    /// ★ `None` means *always*, which is what the overwhelming majority of
    /// items are.
    ///
    /// ★★ **This function is where the rule lives, and that is what let the
    /// field be copied onto a second variant** on 2026-08-27. The note that
    /// used to sit here forbade the copy and named a wrapper as the remedy;
    /// [`Item::Custom`]'s `visible_when` carries the argument for departing
    /// from it, and the trigger that would still bring the wrapper back.
    ///
    /// A **separator** still cannot carry one, and deliberately: a rule for
    /// when a divider disappears is a rule about its *neighbours*, which is
    /// the record-shaped problem the wrapper exists for. A separator between
    /// two hidden items is a cosmetic defect; a separator with its own
    /// condition, set independently of the items it divides, is a
    /// contradiction that renders.
    #[must_use]
    pub fn visible_condition(&self) -> Option<&str> {
        match self {
            Item::Command { visible_when, .. } | Item::Custom { visible_when, .. } => {
                visible_when.as_deref()
            }
            Item::Separator => None,
        }
    }

    /// **The capability this item is conditional on**, if it is conditional.
    ///
    /// `Some` means *a build without this need not register the command, and
    /// dropping the item is the intended configuration*; `None` means *this
    /// item's command is mandatory and its absence is a bug*. See
    /// [`Item::Command::capability`] for the whole rule and
    /// `SHELL_FRAMEWORK.md` §5b for why the two cases must stay
    /// distinguishable.
    ///
    /// ⚠ **Read by the merge and by nothing else.** It is deliberately not a
    /// question the ribbon renderer, a panel or an application ever asks: the
    /// one rule §5b keeps is that a capability's presence is expressed by
    /// registering its command, and a second reader of this field would be a
    /// second place that knows.
    #[must_use]
    pub fn capability(&self) -> Option<&str> {
        match self {
            Item::Command { capability, .. } => capability.as_deref(),
            Item::Separator | Item::Custom { .. } => None,
        }
    }
}
