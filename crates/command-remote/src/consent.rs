//! Who may drive the application: the operator's standing policy, the grant
//! the current connection holds, and the cool-off that stops a refused client
//! from re-raising the question in a loop.
//!
//! Pure state; time is passed in, so every rule is testable without a clock.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How long a refused client waits before it may ask again.
pub const COOL_OFF: Duration = Duration::from_secs(60);
/// How long an unanswered question stays up; expiry is a refusal.
pub const ASK_TIMEOUT: Duration = Duration::from_secs(120);

/// The operator's standing answer, persisted by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Policy {
    /// Ask each time a client says hello. The default.
    #[default]
    Ask,
    /// Allow every client without asking.
    Always,
    /// Refuse every client without asking.
    Never,
}

impl Policy {
    /// The persisted spelling.
    pub fn key(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    /// Inverse of [`key`](Self::key).
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "ask" => Some(Self::Ask),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

/// How far an allowance reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// This connection only.
    Once,
    /// Every connection until the application exits.
    Session,
    /// Every connection from now on; the policy becomes [`Policy::Always`].
    Always,
}

/// What happens to a hello.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Allowed without asking.
    Allowed(Scope),
    /// Refused without asking; the reason is one word for the wire.
    Refused(&'static str),
    /// The operator must be asked.
    Ask,
}

/// The consent state for one application session.
#[derive(Debug, Default)]
pub struct Consent {
    policy: Policy,
    session_allowed: bool,
    refused_at: HashMap<String, Instant>,
}

impl Consent {
    /// Start from the persisted policy.
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            ..Self::default()
        }
    }

    /// The standing policy.
    pub fn policy(&self) -> Policy {
        self.policy
    }

    /// Replace the standing policy (the application's settings changed it).
    /// Moving away from `Always` also withdraws a session allowance, so
    /// revoking in settings takes effect for the next hello.
    pub fn set_policy(&mut self, policy: Policy) {
        if policy != Policy::Always {
            self.session_allowed = false;
        }
        self.policy = policy;
    }

    /// Withdraw a session allowance; the standing policy is untouched.
    pub fn withdraw_session(&mut self) {
        self.session_allowed = false;
    }

    /// Decide a hello from `client`, before any question is raised.
    pub fn judge(&mut self, client: &str, now: Instant) -> Verdict {
        match self.policy {
            Policy::Never => return Verdict::Refused("disabled"),
            Policy::Always => return Verdict::Allowed(Scope::Always),
            Policy::Ask => {}
        }
        if self.session_allowed {
            return Verdict::Allowed(Scope::Session);
        }
        self.refused_at
            .retain(|_, at| now.saturating_duration_since(*at) < COOL_OFF);
        if self.refused_at.contains_key(client) {
            return Verdict::Refused("cooling-off");
        }
        Verdict::Ask
    }

    /// Record the operator's answer to a question about `client`.
    pub fn answer(&mut self, client: &str, allowed: Option<Scope>, now: Instant) {
        match allowed {
            None => {
                self.refused_at.insert(client.to_owned(), now);
            }
            Some(Scope::Once) => {}
            Some(Scope::Session) => self.session_allowed = true,
            Some(Scope::Always) => {
                self.session_allowed = true;
                self.policy = Policy::Always;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policies_decide_without_asking() {
        let now = Instant::now();
        assert_eq!(
            Consent::new(Policy::Never).judge("a", now),
            Verdict::Refused("disabled")
        );
        assert_eq!(
            Consent::new(Policy::Always).judge("a", now),
            Verdict::Allowed(Scope::Always)
        );
        assert_eq!(Consent::new(Policy::Ask).judge("a", now), Verdict::Ask);
    }

    #[test]
    fn a_refusal_cools_off_that_client_only_and_expires() {
        let t0 = Instant::now();
        let mut c = Consent::new(Policy::Ask);
        c.answer("a", None, t0);
        assert_eq!(
            c.judge("a", t0 + Duration::from_secs(1)),
            Verdict::Refused("cooling-off")
        );
        assert_eq!(c.judge("b", t0 + Duration::from_secs(1)), Verdict::Ask);
        assert_eq!(c.judge("a", t0 + COOL_OFF), Verdict::Ask);
    }

    #[test]
    fn scopes_reach_as_far_as_they_say_and_settings_revoke_them() {
        let now = Instant::now();
        let mut c = Consent::new(Policy::Ask);
        c.answer("a", Some(Scope::Once), now);
        assert_eq!(c.judge("a", now), Verdict::Ask);
        c.answer("a", Some(Scope::Session), now);
        assert_eq!(c.judge("b", now), Verdict::Allowed(Scope::Session));
        c.set_policy(Policy::Ask);
        assert_eq!(c.judge("b", now), Verdict::Ask);
        c.answer("a", Some(Scope::Always), now);
        assert_eq!(c.policy(), Policy::Always);
        c.set_policy(Policy::Never);
        assert_eq!(c.judge("a", now), Verdict::Refused("disabled"));
    }
}
