//! Application colour roles the shell stores but never interprets.
//!
//! # Why this module exists
//!
//! The salvage source's [`Palette`](super::Palette) carries nineteen
//! roles. Twelve are chrome — surface, panel, text, accent, outline,
//! danger, notice and the like — and are perfectly generic. Seven are
//! not:
//!
//! | Role | What it meant |
//! |---|---|
//! | `node_mark` | a vector path anchor: "a point is here" |
//! | `node_mark_fill` | that mark's interior, so it reads against dark artwork |
//! | `subpath_outline` | one subpath's outline: "this run is one subpath" |
//! | `dimension_selected` | a committed measurement, selected |
//! | `preview` | an uncommitted proposal |
//! | `guide` | a snap or alignment guide |
//! | `field_chrome` | "an interactive form control lives here" |
//!
//! Those are a vector-PDF-editor's vocabulary. They cannot be renamed
//! into domain-neutrality without being renamed into meaninglessness —
//! `overlay_role_3` is not a role, it is a slot — and a shell that
//! shipped them would be telling the next application that it has
//! subpaths.
//!
//! So they stay out, and the application owns them. What must **not**
//! stay out is the *enforcement* built around them, because that
//! enforcement is generic and is the part that is expensive to learn.
//!
//! # The enforcement, and why it is worth carrying
//!
//! From the salvage source's module header, which is worth quoting
//! because the reasoning is the transferable part:
//!
//! > The canvas overlay palette is not free choice. Several of its
//! > entries carry meaning the operator is expected to learn: the node
//! > mark and the subpath outline are different colours because they
//! > answer different questions; the measurement preview and the
//! > committed dimension differ because one is a proposal and one is
//! > document state; the form-field chrome has a hue of its own because
//! > it means "a control lives here" rather than "this is selected".
//! >
//! > A theme may re-tune those hues. It may **not** collapse two of them
//! > into one […] any theme in which two semantically distinct roles
//! > resolve to the same colour fails the build. […] Colour is never the
//! > only cue for any of these — each also carries a shape, a dash
//! > pattern or a label — but a theme that merges two roles removes a cue
//! > that was doing work, and it would do so silently.
//!
//! That last clause is the whole argument. A merged pair is not a visual
//! defect that someone notices; it is a *cue that stops being a cue*, and
//! the application keeps working while telling the operator slightly
//! less. Nothing but an explicit check finds it.
//!
//! The collision it catches is not hypothetical: a chrome accent picked
//! for chrome reasons alone can land exactly on an overlay role meaning
//! something else, at which point selection and that overlay are
//! indistinguishable on the same surface. `Theme::quiet`'s accent is
//! `#175CC4` rather than the more obvious blue for that reason, and its
//! source says so.
//!
//! # How an application uses this
//!
//! ```no_run
//! use egui_shell::theme::{Overlays, Preset, Theme};
//!
//! // Built per preset, beside the theme, by the application.
//! fn overlays_for(preset: Preset) -> Overlays {
//!     let mut o = Overlays::new();
//!     o.set("node_mark", egui::Color32::from_rgb(30, 110, 220));
//!     o.set("subpath_outline", egui::Color32::from_rgb(210, 140, 40));
//!     let _ = preset; // …re-tuned per preset, as the dark theme does
//!     o
//! }
//!
//! // Once per frame, beside `Theme::apply`.
//! fn each_frame(ctx: &egui::Context, theme: &Theme) {
//!     theme.apply(ctx);
//!     Overlays::install(ctx, overlays_for(theme.preset));
//! }
//!
//! // Anywhere that paints.
//! fn paint(ctx: &egui::Context) {
//!     let overlays = Overlays::of(ctx);
//!     let _mark = overlays.get("node_mark");
//! }
//! ```
//!
//! And the test the application owes, once, for every preset:
//!
//! ```
//! # use egui_shell::theme::Overlays;
//! # use egui::Color32;
//! # let overlays = Overlays::new()
//! #     .with("node_mark", Color32::from_rgb(30, 110, 220))
//! #     .with("subpath_outline", Color32::from_rgb(210, 140, 40))
//! #     .with("preview", Color32::from_rgb(210, 90, 40))
//! #     .with("guide", Color32::from_rgb(160, 90, 40));
//! overlays
//!     .assert_distinct(&["node_mark", "subpath_outline", "preview", "guide"])
//!     .expect("two roles that answer different questions must look different");
//! ```
//!
//! # Why a string-keyed map rather than a generic parameter
//!
//! A `Theme<X>` generic over an application's own palette extension would
//! be type-safe and would infect every signature in the shell that
//! mentions a theme — including the ones in `ribbon`, `dock` and
//! `layout` that have no interest in colour at all. A string-keyed map
//! costs one lookup and a `None` for a typo, and keeps
//! [`super::Theme`] `Copy` and free of parameters.
//!
//! The typo is handled honestly: [`Overlays::get`] returns `Option`, and
//! [`Overlays::assert_distinct`] treats an unknown role as a failure
//! rather than as "trivially distinct from everything". A test that
//! passes because both of its role names were misspelled is worse than no
//! test.

use egui::Color32;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A named set of application colour roles.
///
/// Ordered (`BTreeMap`) rather than hashed, so iteration is deterministic
/// and a failure message lists roles in the same order on every machine.
/// A diagnostic that reorders itself between runs is one nobody can diff.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overlays {
    roles: BTreeMap<String, Color32>,
}

impl Overlays {
    /// An empty set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a role, replacing any previous definition.
    ///
    /// Silently replacing is deliberate: a preset is normally built by
    /// spreading a base set and overriding a few entries, exactly as
    /// `Theme::dark` does with `..quiet.palette`, and making the
    /// override an error would forbid the idiom that makes presets
    /// readable. The risk that idiom carries — two roles quietly becoming
    /// one — is what [`Self::assert_distinct`] is for.
    pub fn set(&mut self, role: impl Into<String>, colour: Color32) {
        self.roles.insert(role.into(), colour);
    }

    /// Builder form of [`Self::set`], for one-expression construction.
    #[must_use]
    pub fn with(mut self, role: impl Into<String>, colour: Color32) -> Self {
        self.set(role, colour);
        self
    }

    /// The colour for a role, or `None` if no such role is defined.
    ///
    /// `Option` rather than a fallback colour. A missing role is a
    /// programming error — a typo, or a role the preset forgot — and
    /// returning magenta or transparent would make it a *rendering*
    /// question the reader has to notice, on the frame where it happens,
    /// on the preset where it happens.
    #[must_use]
    pub fn get(&self, role: &str) -> Option<Color32> {
        self.roles.get(role).copied()
    }

    /// Every defined role, in name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Color32)> {
        self.roles.iter().map(|(k, v)| (k.as_str(), *v))
    }

    /// How many roles are defined.
    #[must_use]
    pub fn len(&self) -> usize {
        self.roles.len()
    }

    /// Whether no roles are defined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }

    /// Assert that the named roles all resolve to different colours.
    ///
    /// This is the generic form of the salvage source's
    /// `distinct_overlay_roles_stay_distinct_in_every_preset`. The
    /// application supplies the list, because only the application knows
    /// which of its roles answer different questions — two roles that
    /// happen to share a colour because they are the *same* semantic in
    /// two places are fine and must not be forced apart.
    ///
    /// # Errors
    ///
    /// [`RoleCollision::Merged`] when two of the named roles resolve to
    /// the same colour, naming both. [`RoleCollision::Undefined`] when a
    /// named role is not defined at all — because a test that passes
    /// because both of its role names were misspelled is worse than no
    /// test, and "unknown roles are trivially distinct" is exactly how
    /// that happens.
    ///
    /// Reports the first collision rather than all of them: unlike the
    /// contrast gate, one merged pair is almost always one edit, and the
    /// pairwise product of a large role set makes an exhaustive report
    /// noisier than it is useful.
    pub fn assert_distinct(&self, roles: &[&str]) -> Result<(), RoleCollision> {
        let mut resolved: Vec<(&str, Color32)> = Vec::with_capacity(roles.len());
        for &role in roles {
            let colour = self
                .get(role)
                .ok_or_else(|| RoleCollision::Undefined { role: role.into() })?;
            if let Some((other, _)) = resolved.iter().find(|(_, c)| *c == colour) {
                return Err(RoleCollision::Merged {
                    first: (*other).into(),
                    second: role.into(),
                    colour,
                });
            }
            resolved.push((role, colour));
        }
        Ok(())
    }

    /// The `egui::Id` under which [`Self::install`] stashes the set.
    const CTX_ID: &'static str = "egui-shell-overlays";

    /// Publish this set for the frame, where any painting code can reach
    /// it.
    ///
    /// Wrapped in an `Arc` on the way in, so [`Self::of`] is a refcount
    /// bump rather than a map clone. This is called once per frame beside
    /// [`super::Theme::apply`] and read by every painter, so the
    /// asymmetry is the right way round.
    pub fn install(ctx: &egui::Context, overlays: Self) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(Self::CTX_ID), Arc::new(overlays)));
    }

    /// The set published for this frame, or an empty set if none was.
    ///
    /// An empty set rather than a panic: a shell that aborts because an
    /// application has not published overlays would be making an optional
    /// extension point mandatory. Every [`Self::get`] then returns `None`,
    /// which is the same signal a missing role gives.
    #[must_use]
    pub fn of(ctx: &egui::Context) -> Arc<Self> {
        ctx.data(|d| d.get_temp::<Arc<Self>>(egui::Id::new(Self::CTX_ID)))
            .unwrap_or_default()
    }
}

/// Why [`Overlays::assert_distinct`] refused a set.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RoleCollision {
    /// Two roles that must answer different questions resolve to the same
    /// colour, so one of the two cues is gone.
    #[error(
        "overlay roles `{first}` and `{second}` both resolve to {colour:?}, \
         so a cue that distinguished them is gone"
    )]
    Merged {
        /// The role that claimed the colour first, in the order given.
        first: String,
        /// The role that collided with it.
        second: String,
        /// The colour they share.
        colour: Color32,
    },
    /// A role named in the check is not defined in this set.
    #[error(
        "overlay role `{role}` is not defined, so the distinctness check \
         would have passed vacuously"
    )]
    Undefined {
        /// The undefined role's name.
        role: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Overlays {
        Overlays::new()
            .with("node_mark", Color32::from_rgb(30, 110, 220))
            .with("subpath_outline", Color32::from_rgb(210, 140, 40))
            .with("preview", Color32::from_rgb(210, 90, 40))
    }

    /// Roles round-trip, and an undefined role is `None` rather than a
    /// substitute colour.
    #[test]
    fn roles_round_trip_and_an_unknown_role_is_none() {
        let o = sample();
        assert_eq!(o.get("node_mark"), Some(Color32::from_rgb(30, 110, 220)));
        assert_eq!(o.get("nodemark"), None, "a typo must not resolve");
        assert_eq!(o.len(), 3);
    }

    /// **Distinct roles pass; merged roles fail, and the failure names
    /// both.**
    ///
    /// The message is the deliverable: "two roles collided" sends the
    /// reader back to work out which two, on a preset with a dozen of
    /// them.
    #[test]
    fn merged_roles_are_refused_and_both_are_named() {
        let ok = sample();
        ok.assert_distinct(&["node_mark", "subpath_outline", "preview"])
            .expect("three different colours are distinct");

        // The real failure shape: a preset spread from another one, with
        // an override that lands on a sibling's value.
        let collided = sample().with("preview", Color32::from_rgb(210, 140, 40));
        let err = collided
            .assert_distinct(&["node_mark", "subpath_outline", "preview"])
            .expect_err("two roles sharing a colour must be refused");
        let text = err.to_string();
        assert!(text.contains("subpath_outline"), "{text}");
        assert!(text.contains("preview"), "{text}");
    }

    /// **An undefined role fails the check rather than passing it
    /// vacuously.**
    ///
    /// This is the difference between a gate and a decoration. If unknown
    /// roles were treated as trivially distinct, a check whose role names
    /// were both misspelled — or whose roles were renamed in the palette
    /// and not in the test — would go green forever while measuring
    /// nothing. The salvage source's whole family of "green is not
    /// evidence" lessons is this same failure in other clothes.
    #[test]
    fn an_undefined_role_fails_rather_than_passing_vacuously() {
        let err = sample()
            .assert_distinct(&["node_mark", "nodemark"])
            .expect_err("a misspelled role must not pass as distinct");
        assert!(matches!(err, RoleCollision::Undefined { .. }), "{err:?}");
        assert!(err.to_string().contains("vacuously"));
    }

    /// An empty set is usable and reports itself as empty.
    #[test]
    fn an_empty_set_is_usable() {
        let o = Overlays::new();
        assert!(o.is_empty());
        assert_eq!(o.get("anything"), None);
        assert_eq!(o.iter().count(), 0);
    }
}
