//! Whether another program on this computer may drive the window (the live
//! link): ask each time, always allow, or never allow. Persisted in the
//! preferences file; the link itself is `command-remote`.

/// The operator's standing answer to a program asking to drive the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RemoteControl {
    /// Ask each time. The default.
    #[default]
    Ask,
    /// Allow any program run by this user, without asking.
    Always,
    /// Refuse every program, without asking.
    Never,
}

impl RemoteControl {
    /// Every value, in the order the settings window lists them.
    pub const ALL: &'static [Self] = &[Self::Ask, Self::Always, Self::Never];

    /// The token written to the preferences file.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: a file token, never displayed.
            Self::Ask => "ask",
            // ui-text-exempt: a file token, never displayed.
            Self::Always => "always",
            // ui-text-exempt: a file token, never displayed.
            Self::Never => "never",
        }
    }

    /// Inverse of [`Self::key`].
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|v| v.key() == key.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips_and_a_stranger_is_refused() {
        for v in RemoteControl::ALL {
            assert_eq!(RemoteControl::from_key(v.key()), Some(*v));
        }
        assert_eq!(RemoteControl::from_key("maybe"), None);
    }
}
