//! The RFC 3161 time-stamping server a PAdES B-T signature asks.
//!
//! [`available`] is the only answer to whether this build can reach one: the
//! Sign window offers the server field exactly when it is true (R8 — presence
//! is what this build linked, never a `cfg` in a surface). [`HttpAuthority`]
//! is the engine's `TimestampAuthority` over `pdfcer_fetch`, whose transport
//! bounds the whole exchange by [`TIMEOUT`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/sign/timestamp.md`.

use std::time::Duration;

/// How long one request to the server may take before signing gives up.
pub const TIMEOUT: Duration = Duration::from_secs(30);

/// Whether this build can ask a time-stamping server.
#[must_use]
pub const fn available() -> bool {
    cfg!(feature = "timestamp")
}

/// The server field as typed, or `None` when it asks for no timestamp.
///
/// Blank is the ordinary B-B signature; anything else is a request for B-T,
/// and a malformed address is refused by the transport, by name, rather than
/// quietly signing without the timestamp the operator asked for.
#[must_use]
pub fn requested(field: &str) -> Option<&str> {
    let url = field.trim();
    (!url.is_empty()).then_some(url)
}

/// The authority for `url`, or `None` in a build that cannot ask one.
#[must_use]
pub fn authority_for(
    url: &str,
) -> Option<Box<dyn pdfcer_core::sign::timestamp::TimestampAuthority>> {
    #[cfg(feature = "timestamp")]
    {
        Some(Box::new(HttpAuthority::new(url)))
    }
    #[cfg(not(feature = "timestamp"))]
    {
        let _ = url;
        None
    }
}

/// A time-stamping server named by URL, asked over HTTP(S).
#[cfg(feature = "timestamp")]
#[derive(Clone, Debug)]
pub struct HttpAuthority {
    url: String,
    timeout: Duration,
}

#[cfg(feature = "timestamp")]
impl HttpAuthority {
    /// The server at `url`, bounded by [`TIMEOUT`].
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            timeout: TIMEOUT,
        }
    }

    /// The same server with another bound; tests use a short one.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[cfg(feature = "timestamp")]
impl pdfcer_core::sign::timestamp::TimestampAuthority for HttpAuthority {
    fn time_stamp(&self, request_der: &[u8]) -> Result<Vec<u8>, String> {
        let options = pdfcer_fetch::TimeStampOptions::default().with_timeout(self.timeout);
        pdfcer_fetch::post_time_stamp_query_with(&self.url, request_der, &options)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_field_asks_for_no_timestamp() {
        assert_eq!(requested("   "), None);
        assert_eq!(
            requested(" http://tsa.example/ "),
            Some("http://tsa.example/")
        );
    }

    #[test]
    fn availability_is_the_feature() {
        assert_eq!(available(), cfg!(feature = "timestamp"));
    }

    /// A server that accepts the connection and never answers must cost the
    /// bound, not forever. Local and offline: the listener is ours.
    #[cfg(feature = "timestamp")]
    #[test]
    fn a_silent_server_is_given_up_on_at_the_bound() {
        use pdfcer_core::sign::timestamp::TimestampAuthority;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let hold = std::thread::spawn(move || {
            let conn = listener.accept();
            std::thread::sleep(Duration::from_secs(3));
            drop(conn);
        });
        let started = std::time::Instant::now();
        let authority = HttpAuthority::new(&format!("http://127.0.0.1:{port}/"))
            .with_timeout(Duration::from_millis(300));
        let answer = authority.time_stamp(b"\x30\x00");
        assert!(answer.is_err(), "{answer:?}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "{:?}",
            started.elapsed()
        );
        let _ = hold.join();
    }

    #[cfg(feature = "timestamp")]
    #[test]
    fn a_scheme_other_than_http_is_refused() {
        use pdfcer_core::sign::timestamp::TimestampAuthority;
        let answer = HttpAuthority::new("ftp://tsa.example/").time_stamp(b"\x30\x00");
        assert!(answer.is_err());
    }
}
