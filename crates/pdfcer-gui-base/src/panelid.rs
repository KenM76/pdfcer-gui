//! # `panelid` — which dockable panels exist, and the command that opens each
//!
//! The catalog half of `pdfcer-gui`'s `panels` module: the [`Panel`] enum, its
//! exhaustive [`Panel::ALL`], and the one binding between a panel and its
//! ribbon command. Drawing a panel stays in the application, as
//! `panels::show`; nothing here touches egui or a document.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/panelid.md`.

/// One dockable panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Panel {
    /// The document's outline, as navigation.
    Bookmarks,
    /// The document's optional-content groups.
    Layers,
    /// What each digital signature covers.
    Signatures,
    /// What fonts the document declares, and what they cost.
    Fonts,
    /// Everything drawn on the current page.
    Objects,
    /// The read-only facts about one object — **and about nothing else**.
    ///
    /// The document's own title, author, subject and keywords are true of no
    /// selection, so a panel holding both would draw them permanently above
    /// the selection. They are [`Self::DocumentProperties`].
    Properties,
    /// **The document's own title, author, subject and keywords**, and the
    /// facts pdfcer read about the file — `file.document_properties`.
    ///
    /// **A new id rather than a second meaning for `file.properties`**:
    /// [`Self::command_id`] is the single binding between a command and a
    /// panel, and the dispatcher resolves toggles through
    /// [`Self::from_command_id`], so one id cannot open two panels.
    ///
    /// **A toggle, unlike [`Self::Properties`]**, whose command is show-only
    /// because the Objects row context menu offers it to describe the row
    /// just clicked, and a second invocation that closed the description
    /// would be hostile. Nothing offers this command to describe anything.
    ///
    /// **Mounted by all three modes.** Reading a document's title is reading,
    /// and Read is shown the `file` tab.
    DocumentProperties,
    /// The document's form fields, for **filling** — not for authoring.
    ///
    /// Creating, deleting, renaming and grouping fields are `Edit ▸ Forms`
    /// authoring work behind a different certification gate, and are absent.
    Forms,
    /// The document's pages, as pictures — navigate, pick, and act on sheets.
    ///
    /// Offered by **all three** modes (page operations do not alter content,
    /// so a reviewer may rotate and extract), and the only panel whose body
    /// renders, which is why it has a rendering policy rather than a loop.
    Pages,
    /// Every annotation on the document — the comment list a reviewer works
    /// through. A **report with one verb**: it raises a go-to-page action and
    /// nothing else.
    Comments,
    /// Marking content for permanent removal, and reviewing what is marked.
    ///
    /// A **toggle**: pressing Redact with the panel open closes it, through
    /// [`Self::from_command_id`] like every `view.panel_*` control. The
    /// irreversible half is the `edit.redact_apply` dialog, because applying is
    /// a single transaction and a control that commits an irreversible
    /// operation must not sit two rows below one that merely marks.
    Redact,
    /// Where ce-dimension groups are made, chosen and configured.
    ///
    /// A dock column rather than a window, because a window taller than the
    /// screen pushes its own title bar and close button off the desktop.
    DimensionGroups,
    /// The whole files this document carries inside itself (§7.11.4.1).
    ///
    /// The cost of `edit.attachments` is stated rather than hidden: Acrobat
    /// *Reader* lists attachments and saves them out, and Read mode here
    /// cannot. The fix the day that matters is a second, verb-less listing
    /// panel, not a second id for this one.
    Attachments,
    /// Inkscape's Align and Distribute, over the selected page objects.
    /// `ALIGN_AND_DISTRIBUTE.md` maps each Inkscape control.
    AlignDistribute,
}

impl Panel {
    /// Every panel.
    pub const ALL: [Self; 14] = [
        Self::Attachments,
        Self::Bookmarks,
        Self::Layers,
        Self::Signatures,
        Self::Fonts,
        Self::Objects,
        Self::Properties,
        Self::DocumentProperties,
        Self::Forms,
        Self::Pages,
        Self::Comments,
        Self::Redact,
        Self::DimensionGroups,
        Self::AlignDistribute,
    ];

    /// The ribbon command that shows this panel.
    ///
    /// Seven of these are not `view.panel_*` ids; the placement of each is
    /// argued in this module's doc. The ids that open authoring surfaces sit on
    /// tabs Read is not shown, so the mode taxonomy gates them with no
    /// capability flag of its own.
    #[must_use]
    pub fn command_id(self) -> &'static str {
        match self {
            Self::Attachments => "edit.attachments",
            Self::Bookmarks => "view.panel_bookmarks",
            Self::Layers => "view.panel_layers",
            Self::Signatures => "view.panel_signatures",
            Self::Fonts => "file.fonts",
            Self::Objects => "view.panel_objects",
            Self::Properties => "file.properties",
            Self::DocumentProperties => "file.document_properties",
            // On View, not Edit: Read fills forms, and Read is shown only
            // `file` and `view`.
            Self::Forms => "view.panel_forms",
            // Not registered in this build, so every arrangement filters the
            // panel out; see the `pages` panel's header.
            Self::Pages => "view.panel_pages",
            // `RIBBON_IA.md` §7's per-control ruling sends it to Markup.
            Self::Comments => "markup.comments",
            Self::Redact => "edit.redact",
            Self::DimensionGroups => "measure.manage_groups",
            Self::AlignDistribute => "edit.align",
        }
    }

    /// The panel whose [`Self::command_id`] is `id`, if any.
    #[must_use]
    pub fn from_command_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.command_id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::Panel;

    /// **No two panels claim the same command.**
    #[test]
    fn no_two_panels_share_a_command() {
        let mut seen: Vec<&str> = Vec::new();
        for panel in Panel::ALL {
            let id = panel.command_id();
            assert!(
                !seen.contains(&id),
                "{panel:?} claims `{id}`, which another panel already claims. \
                 One command opens one panel."
            );
            seen.push(id);
        }
    }

    /// **The hand-written catalog is exhaustive.**
    #[test]
    fn the_panel_catalog_is_complete() {
        // Exhaustive by construction: no `_` arm.
        const fn ordinal(p: Panel) -> usize {
            match p {
                Panel::Bookmarks => 0,
                Panel::Layers => 1,
                Panel::Signatures => 2,
                Panel::Fonts => 3,
                Panel::Objects => 4,
                Panel::Properties => 5,
                Panel::Forms => 6,
                Panel::Pages => 7,
                Panel::Comments => 8,
                Panel::Redact => 9,
                Panel::DimensionGroups => 10,
                Panel::Attachments => 11,
                Panel::DocumentProperties => 12,
                Panel::AlignDistribute => 13,
            }
        }
        let mut ordinals: Vec<usize> = Panel::ALL.iter().copied().map(ordinal).collect();
        ordinals.sort_unstable();
        ordinals.dedup();
        assert_eq!(
            ordinals,
            (0..Panel::ALL.len()).collect::<Vec<_>>(),
            "Panel::ALL is missing a variant, or lists one twice"
        );
    }

    /// Every id resolves back to its own panel, and an unknown id to none.
    #[test]
    fn every_command_id_resolves_to_its_own_panel() {
        for panel in Panel::ALL {
            assert_eq!(Panel::from_command_id(panel.command_id()), Some(panel));
        }
        assert_eq!(Panel::from_command_id("view.panel_absent"), None);
    }
}
