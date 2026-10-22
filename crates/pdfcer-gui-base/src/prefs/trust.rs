//! # `prefs::trust` — what a document may reach outside itself
//!
//! The answers the operator has given to Security ▸ Trust and to the prompts a
//! web link, a file launch or 3D content raises. Nothing here refuses without
//! the operator having said so: every default asks, or (3D) plays.
//!
//! Keys: `web_links` (`ask` | `allow` | `block`), `web_site` (repeated,
//! `allow <host>` or `block <host>`), `open_other_files` (`ask` | `allow`),
//! `play_3d` (`true` | `false`), `trusted_3d_document` (repeated, a path).
//!
//! Design and rationale: `DESIGNS.md` § O320.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::printing::KeyOutcome;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const WEB_KEY: &str = "web_links";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const SITE_KEY: &str = "web_site";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const FILES_KEY: &str = "open_other_files";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const PLAY_3D_KEY: &str = "play_3d";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const TRUSTED_3D_KEY: &str = "trusted_3d_document";
// ui-text-exempt: a file VALUE.
const ASK: &str = "ask";
// ui-text-exempt: a file VALUE.
const ALLOW: &str = "allow";
// ui-text-exempt: a file VALUE.
const BLOCK: &str = "block";

/// What a click on a web link does before any per-site answer is consulted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WebLinks {
    /// Ask, unless the site has a remembered answer.
    #[default]
    Ask,
    /// Open every web link without asking.
    Allow,
    /// Refuse every web link without asking; the status line says where to change it.
    Block,
}

impl WebLinks {
    const fn token(self) -> &'static str {
        match self {
            Self::Ask => ASK,
            Self::Allow => ALLOW,
            Self::Block => BLOCK,
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        [Self::Ask, Self::Allow, Self::Block]
            .into_iter()
            .find(|mode| mode.token() == token)
    }
}

/// What to do with one web link, after the mode and remembered sites are consulted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDecision {
    /// Hand the link to the system.
    Open,
    /// Do not open it; the operator chose this earlier.
    Refuse,
    /// Show the Security Warning prompt.
    Ask,
}

/// One remembered answer for a site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteAnswer {
    /// The host, lower-cased.
    pub host: String,
    /// `true` for Allow, `false` for Block.
    pub allow: bool,
}

/// Security ▸ Trust, and every remembered answer to its prompts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustPrefs {
    /// The web-link mode.
    pub web_links: WebLinks,
    sites: Vec<SiteAnswer>,
    /// Whether opening a file a link names asks first.
    pub ask_before_files: bool,
    /// Whether 3D content plays without asking.
    pub play_3d: bool,
    trusted_3d: Vec<PathBuf>,
}

impl Default for TrustPrefs {
    fn default() -> Self {
        Self {
            web_links: WebLinks::Ask,
            sites: Vec::new(),
            ask_before_files: true,
            play_3d: true,
            trusted_3d: Vec::new(),
        }
    }
}

impl TrustPrefs {
    /// The most sites, or documents, remembered; the oldest is dropped past it.
    pub const MAX_REMEMBERED: usize = 256;

    /// Every remembered site, oldest first.
    #[must_use]
    pub fn sites(&self) -> &[SiteAnswer] {
        &self.sites
    }

    /// Remember `allow` for `host`, replacing an earlier answer for it.
    pub fn remember_site(&mut self, host: &str, allow: bool) {
        let host = host.trim().to_ascii_lowercase();
        if host.is_empty() {
            return;
        }
        self.sites.retain(|s| s.host != host);
        if self.sites.len() >= Self::MAX_REMEMBERED {
            self.sites.remove(0);
        }
        self.sites.push(SiteAnswer { host, allow });
    }

    /// Replace every remembered site with `sites`, normalised as
    /// [`Self::remember_site`] normalises one.
    pub fn set_sites(&mut self, sites: &[SiteAnswer]) {
        self.sites.clear();
        for site in sites {
            self.remember_site(&site.host, site.allow);
        }
    }

    /// Forget the site at `index`, if there is one.
    pub fn forget_site(&mut self, index: usize) {
        if index < self.sites.len() {
            self.sites.remove(index);
        }
    }

    /// What a click on a link to `host` does. A remembered site answers only
    /// while the mode is `Ask`; `Allow` and `Block` are the operator's blanket word.
    #[must_use]
    pub fn web_decision(&self, host: &str) -> WebDecision {
        match self.web_links {
            WebLinks::Allow => WebDecision::Open,
            WebLinks::Block => WebDecision::Refuse,
            WebLinks::Ask => match self
                .sites
                .iter()
                .find(|s| s.host.eq_ignore_ascii_case(host))
            {
                Some(s) if s.allow => WebDecision::Open,
                Some(_) => WebDecision::Refuse,
                None => WebDecision::Ask,
            },
        }
    }

    /// Documents whose 3D content plays while Play 3D content is off.
    #[must_use]
    pub fn trusted_3d(&self) -> &[PathBuf] {
        &self.trusted_3d
    }

    /// Trust `doc`'s 3D content always.
    pub fn trust_3d(&mut self, doc: &Path) {
        if self.trusted_3d.iter().any(|d| d == doc) {
            return;
        }
        if self.trusted_3d.len() >= Self::MAX_REMEMBERED {
            self.trusted_3d.remove(0);
        }
        self.trusted_3d.push(doc.to_path_buf());
    }

    /// Stop trusting the document at `index`, if there is one.
    pub fn forget_3d(&mut self, index: usize) {
        if index < self.trusted_3d.len() {
            self.trusted_3d.remove(index);
        }
    }

    /// Whether `doc`'s 3D content plays without asking.
    #[must_use]
    pub fn plays_3d(&self, doc: Option<&Path>) -> bool {
        self.play_3d || doc.is_some_and(|d| self.trusted_3d.iter().any(|t| t == d))
    }
}

/// Read one `key = value` line into [`TrustPrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut TrustPrefs, key: &str, value: &str) -> KeyOutcome {
    let value = value.trim();
    let read = match key {
        WEB_KEY => WebLinks::from_token(value)
            .inspect(|&mode| prefs.web_links = mode)
            .is_some(),
        SITE_KEY => parse_site(value)
            .inspect(|&(host, allow)| prefs.remember_site(host, allow))
            .is_some(),
        FILES_KEY => {
            let ask = match value {
                ASK => Some(true),
                ALLOW => Some(false),
                _ => None,
            };
            ask.inspect(|&ask| prefs.ask_before_files = ask).is_some()
        }
        PLAY_3D_KEY => value
            .parse::<bool>()
            .ok()
            .inspect(|&on| prefs.play_3d = on)
            .is_some(),
        TRUSTED_3D_KEY => {
            if !value.is_empty() {
                prefs.trust_3d(Path::new(value));
            }
            !value.is_empty()
        }
        _ => return KeyOutcome::NotMine,
    };
    if read {
        KeyOutcome::Accepted
    } else {
        KeyOutcome::BadValue
    }
}

/// `allow <host>` or `block <host>`.
fn parse_site(value: &str) -> Option<(&str, bool)> {
    let (answer, host) = value.split_once(char::is_whitespace)?;
    let host = host.trim();
    if host.is_empty() || host.contains(char::is_whitespace) {
        return None;
    }
    match answer {
        ALLOW => Some((host, true)),
        BLOCK => Some((host, false)),
        _ => None,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &TrustPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # Security > Trust: what a document may reach outside itself.\n\
         #   web_links: ask, allow or block (default ask)\n\
         #   web_site: allow <host> or block <host>, one per line; consulted\n\
         #     only while web_links is ask\n\
         #   open_other_files: ask or allow (default ask)\n\
         #   play_3d: true or false (default true)\n\
         #   trusted_3d_document: a document whose 3D plays while play_3d is false\n\
         ",
    );
    // ui-text-exempt: file KEYS, separators and values, never displayed.
    let _ = writeln!(out, "{WEB_KEY} = {}", prefs.web_links.token());
    for site in &prefs.sites {
        let answer = if site.allow { ALLOW } else { BLOCK };
        let _ = writeln!(out, "{SITE_KEY} = {answer} {}", site.host); // ui-text-exempt: a preferences-file line, never displayed
    }
    let files = if prefs.ask_before_files { ASK } else { ALLOW };
    let _ = writeln!(out, "{FILES_KEY} = {files}"); // ui-text-exempt: a preferences-file line, never displayed
    let _ = writeln!(out, "{PLAY_3D_KEY} = {}", prefs.play_3d); // ui-text-exempt: a preferences-file line, never displayed
    for doc in &prefs.trusted_3d {
        let _ = writeln!(out, "{TRUSTED_3D_KEY} = {}", doc.display()); // ui-text-exempt: a preferences-file line, never displayed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_back(original: &TrustPrefs) -> TrustPrefs {
        let mut out = String::new();
        write_block(original, &mut out);
        let mut prefs = TrustPrefs::default();
        for line in out.lines().filter(|l| !l.starts_with('#')) {
            if let Some((key, value)) = line.split_once('=') {
                assert_eq!(
                    parse_key(&mut prefs, key.trim(), value),
                    KeyOutcome::Accepted,
                    "{line}"
                );
            }
        }
        prefs
    }

    #[test]
    fn every_answer_round_trips_through_the_block() {
        let mut original = TrustPrefs {
            web_links: WebLinks::Block,
            ask_before_files: false,
            play_3d: false,
            ..TrustPrefs::default()
        };
        original.remember_site("Example.COM", true);
        original.remember_site("bad.example", false);
        original.trust_3d(Path::new(r"D:\drawings\pump.pdf"));
        assert_eq!(read_back(&original), original);
    }

    #[test]
    fn a_remembered_site_answers_only_while_asking() {
        let mut prefs = TrustPrefs::default();
        assert_eq!(prefs.web_decision("example.com"), WebDecision::Ask);
        prefs.remember_site("example.com", false);
        assert_eq!(prefs.web_decision("EXAMPLE.com"), WebDecision::Refuse);
        prefs.remember_site("example.com", true);
        assert_eq!(prefs.sites().len(), 1);
        assert_eq!(prefs.web_decision("example.com"), WebDecision::Open);
        prefs.web_links = WebLinks::Block;
        assert_eq!(prefs.web_decision("example.com"), WebDecision::Refuse);
    }

    #[test]
    fn a_trusted_document_plays_3d_while_the_switch_is_off() {
        let mut prefs = TrustPrefs {
            play_3d: false,
            ..TrustPrefs::default()
        };
        let doc = Path::new(r"D:\a.pdf");
        assert!(!prefs.plays_3d(Some(doc)));
        prefs.trust_3d(doc);
        assert!(prefs.plays_3d(Some(doc)));
        assert!(!prefs.plays_3d(None));
    }

    #[test]
    fn a_malformed_value_is_ours_and_changes_nothing() {
        let mut prefs = TrustPrefs::default();
        for (key, value) in [
            (WEB_KEY, "sometimes"),
            (SITE_KEY, "maybe example.com"),
            (SITE_KEY, "allow"),
            (FILES_KEY, "never"),
            (PLAY_3D_KEY, "on"),
            (TRUSTED_3D_KEY, ""),
        ] {
            assert_eq!(
                parse_key(&mut prefs, key, value),
                KeyOutcome::BadValue,
                "{key}"
            );
        }
        assert_eq!(
            parse_key(&mut prefs, "snapshot_dpi", "300"),
            KeyOutcome::NotMine
        );
        assert_eq!(prefs, TrustPrefs::default());
    }
}
