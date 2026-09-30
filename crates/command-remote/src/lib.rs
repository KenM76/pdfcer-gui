//! Let another process on this machine, run by this user, drive an egui
//! application through its registered commands, with the operator's consent.
//!
//! **Shape.** [`Remote::start`] opens a named pipe that only the current user
//! can reach and writes a discovery file saying where it is. Each frame the
//! application calls [`Remote::poll`]; it returns the requests that have passed
//! consent, and the application answers each with [`Remote::respond`], in the
//! same frame or a later one. What the verbs mean is the application's
//! business; this crate knows commands by name and nothing about any document.
//!
//! **Consent.** A client must first send `hello <client> <purpose>`. Under
//! [`Policy::Ask`] that raises a question the application draws with
//! [`ui::banner`]; the client waits for the answer. A refusal or an unanswered
//! question (after [`consent::ASK_TIMEOUT`]) refuses the client, which then
//! cannot ask again for [`consent::COOL_OFF`]. Until allowed, every other verb
//! is answered `err not-enabled`. [`Remote::disconnect`] ends a connection at
//! the operator's request.
//!
//! **Protocol.** [`protocol`]: a request line in, a framed reply out. The
//! crate itself answers only `hello` and `bye`.
//!
//! **Wake-up.** The pipe thread calls the `wake` closure given to
//! [`Remote::start`] whenever something arrives; pass one that requests a
//! repaint, so an idle application still answers.

pub mod consent;
mod discovery;
pub mod protocol;
mod server;
#[cfg(feature = "ui")]
pub mod ui;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::time::Instant;

pub use consent::{Policy, Scope};
pub use discovery::{Instance, list as list_instances};
pub use protocol::{Line, Reply};

use consent::{Consent, Verdict};
use server::Event;

/// How many log entries are kept.
const LOG_CAPACITY: usize = 500;

/// What [`Remote::start`] needs.
#[derive(Debug, Clone)]
pub struct Config {
    /// The application's name: the pipe is `<app>-remote-<pid>`.
    pub app: String,
    /// Where discovery files live; `None` writes none.
    pub discovery_dir: Option<PathBuf>,
    /// The persisted policy.
    pub policy: Policy,
}

/// A client asking to be allowed, awaiting the operator.
#[derive(Debug)]
pub struct Knock {
    /// The name the client gave. Self-declared: any process of this user could send it.
    pub client: String,
    /// What the client says it wants to do.
    pub purpose: String,
    /// When the question was raised.
    pub raised: Instant,
    answer: Sender<Result<Scope, &'static str>>,
    id: u64,
}

/// A request that passed consent, for the application to answer.
#[derive(Debug)]
pub struct Request {
    /// The verb, lowercased.
    pub verb: String,
    /// The arguments.
    pub args: Vec<String>,
    reply: Sender<Reply>,
    log_seq: u64,
}

/// One line of the log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Monotonic sequence number.
    pub seq: u64,
    /// The client the line concerns.
    pub client: String,
    /// What was asked, as sent.
    pub asked: String,
    /// The reply's status line; `None` while unanswered.
    pub outcome: Option<String>,
}

#[derive(Debug)]
struct Connection {
    id: u64,
    client: Option<String>,
}

/// The remote's UI-thread half.
pub struct Remote {
    rx: Receiver<Event>,
    consent: Consent,
    pipe: String,
    discovery: Option<discovery::Discovery>,
    connection: Option<Connection>,
    knock: Option<Knock>,
    revoked: Arc<AtomicU64>,
    log: VecDeque<LogEntry>,
    next_seq: u64,
    failure: Option<String>,
}

impl std::fmt::Debug for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Remote")
            .field("pipe", &self.pipe)
            .field("connection", &self.connection)
            .finish_non_exhaustive()
    }
}

impl Remote {
    /// Open the pipe and write the discovery file.
    ///
    /// # Errors
    /// The discovery file cannot be written, or the thread cannot start.
    /// A pipe that cannot be created is reported later by [`failure`](Self::failure).
    pub fn start(config: Config, wake: impl Fn() + Send + Sync + 'static) -> std::io::Result<Self> {
        let server =
            native_pipe::PipeServer::new(&format!("{}-remote-{}", config.app, std::process::id()));
        let pipe = server.name().to_owned();
        let discovery = config
            .discovery_dir
            .as_deref()
            .map(|dir| discovery::Discovery::create(dir, &pipe, &config.app))
            .transpose()?;
        let (tx, rx) = channel();
        let revoked = Arc::new(AtomicU64::new(0));
        server::spawn(server, tx, Arc::new(wake), Arc::clone(&revoked))?;
        Ok(Self {
            rx,
            consent: Consent::new(config.policy),
            pipe,
            discovery,
            connection: None,
            knock: None,
            revoked,
            log: VecDeque::new(),
            next_seq: 0,
            failure: None,
        })
    }

    /// Take everything the pipe thread has sent. Returns the requests that
    /// passed consent; hellos are settled here or become the [`knock`](Self::knock).
    pub fn poll(&mut self, now: Instant) -> Vec<Request> {
        self.expire_knock(now);
        let mut out = Vec::new();
        loop {
            let event = match self.rx.try_recv() {
                Ok(event) => event,
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            };
            match event {
                Event::Connected { id } => self.connection = Some(Connection { id, client: None }),
                Event::Closed { id } => {
                    if self.connection.as_ref().is_some_and(|c| c.id == id) {
                        self.connection = None;
                    }
                    if self.knock.as_ref().is_some_and(|k| k.id == id) {
                        self.knock = None;
                    }
                }
                Event::Hello {
                    id,
                    client,
                    purpose,
                    answer,
                } => self.hello(id, client, purpose, answer, now),
                Event::Request { id, line, reply } => {
                    let client = self.client_of(id);
                    let asked = std::iter::once(line.verb.clone())
                        .chain(line.args.iter().map(|a| protocol::quote(a)))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let log_seq = self.push_log(client, asked, None);
                    out.push(Request {
                        verb: line.verb,
                        args: line.args,
                        reply,
                        log_seq,
                    });
                }
                Event::Failed(why) => self.failure = Some(why),
            }
        }
        out
    }

    /// Answer a request and record the outcome in the log.
    pub fn respond(&mut self, request: Request, reply: Reply) {
        let status = reply.encode().lines().next().unwrap_or_default().to_owned();
        if let Some(entry) = self.log.iter_mut().find(|e| e.seq == request.log_seq) {
            entry.outcome = Some(status);
        }
        let _ = request.reply.send(reply);
    }

    /// The question awaiting the operator, if any.
    pub fn knock(&self) -> Option<&Knock> {
        self.knock.as_ref()
    }

    /// The operator's answer to the [`knock`](Self::knock): `None` refuses.
    pub fn decide(&mut self, allowed: Option<Scope>, now: Instant) {
        let Some(knock) = self.knock.take() else {
            return;
        };
        self.consent.answer(&knock.client, allowed, now);
        self.settle(
            knock.id,
            &knock.client,
            allowed.ok_or("refused"),
            knock.answer,
        );
    }

    /// The allowed client of the live connection.
    pub fn connected_client(&self) -> Option<&str> {
        self.connection.as_ref()?.client.as_deref()
    }

    /// End the live connection now and withdraw a session allowance. The
    /// client's next request is answered `err disconnected`. A standing
    /// [`Policy::Always`] is untouched: that is the settings' to change.
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            self.revoked.fetch_max(conn.id, Ordering::AcqRel);
            let client = conn.client.unwrap_or_default();
            self.push_log(
                client,
                "(disconnected by the operator)".to_owned(),
                Some(String::new()),
            );
        }
        self.consent.withdraw_session();
    }

    /// The standing policy. The application persists it: an "Always allow"
    /// answer changes it, so compare after each [`decide`](Self::decide).
    pub fn policy(&self) -> Policy {
        self.consent.policy()
    }

    /// Replace the standing policy from the application's settings.
    pub fn set_policy(&mut self, policy: Policy) {
        self.consent.set_policy(policy);
    }

    /// Rewrite the discovery file's document field.
    pub fn set_document(&mut self, document: Option<&Path>) {
        if let Some(d) = &self.discovery {
            let _ = d.write(document);
        }
    }

    /// The pipe path clients open.
    pub fn pipe(&self) -> &str {
        &self.pipe
    }

    /// Why the pipe could not be opened, if it could not.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    /// The log, oldest first.
    pub fn log(&self) -> &VecDeque<LogEntry> {
        &self.log
    }

    fn hello(
        &mut self,
        id: u64,
        client: String,
        purpose: String,
        answer: Sender<Result<Scope, &'static str>>,
        now: Instant,
    ) {
        match self.consent.judge(&client, now) {
            Verdict::Allowed(scope) => self.settle(id, &client, Ok(scope), answer),
            Verdict::Refused(why) => self.settle(id, &client, Err(why), answer),
            Verdict::Ask if self.knock.is_some() => self.settle(id, &client, Err("busy"), answer),
            Verdict::Ask => {
                self.knock = Some(Knock {
                    client,
                    purpose,
                    raised: now,
                    answer,
                    id,
                });
            }
        }
    }

    fn settle(
        &mut self,
        id: u64,
        client: &str,
        result: Result<Scope, &'static str>,
        answer: Sender<Result<Scope, &'static str>>,
    ) {
        let outcome = match result {
            Ok(scope) => {
                if let Some(conn) = self.connection.as_mut().filter(|c| c.id == id) {
                    conn.client = Some(client.to_owned());
                }
                format!("allowed {scope:?}").to_lowercase()
            }
            Err(why) => format!("refused {why}"),
        };
        self.push_log(client.to_owned(), "hello".to_owned(), Some(outcome));
        let _ = answer.send(result);
    }

    fn expire_knock(&mut self, now: Instant) {
        if self
            .knock
            .as_ref()
            .is_some_and(|k| now.saturating_duration_since(k.raised) >= consent::ASK_TIMEOUT)
        {
            self.decide(None, now);
        }
    }

    fn client_of(&self, id: u64) -> String {
        self.connection
            .as_ref()
            .filter(|c| c.id == id)
            .and_then(|c| c.client.clone())
            .unwrap_or_default()
    }

    fn push_log(&mut self, client: String, asked: String, outcome: Option<String>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        if self.log.len() == LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(LogEntry {
            seq,
            client,
            asked,
            outcome,
        });
        seq
    }
}

#[cfg(all(test, windows))]
mod tests;

/// Where an application's discovery files live by convention:
/// `%LOCALAPPDATA%\<app>\remote`. `None` when the variable is unset.
#[must_use]
pub fn default_discovery_dir(app: &str) -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join(app).join("remote"))
}
