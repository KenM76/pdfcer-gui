//! End to end through a real pipe: consent gates every verb, the operator's
//! answer reaches the client, and a disconnect ends the connection.

use super::*;
use std::io::{BufReader, Write};
use std::time::Duration;

struct Client {
    reader: BufReader<std::fs::File>,
}

impl Client {
    fn open(pipe: &str) -> Self {
        for _ in 0..300 {
            if let Ok(f) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(pipe)
            {
                return Self {
                    reader: BufReader::new(f),
                };
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no connection to {pipe}");
    }

    fn send(&mut self, line: &str) {
        let f = self.reader.get_mut();
        f.write_all(format!("{line}\n").as_bytes()).expect("write");
    }

    fn reply(&mut self) -> Reply {
        protocol::read_reply(&mut self.reader).expect("reply")
    }
}

/// Poll until `f` returns something, or fail after two seconds.
fn until<T>(remote: &mut Remote, mut f: impl FnMut(&mut Remote, Vec<Request>) -> Option<T>) -> T {
    for _ in 0..200 {
        let got = remote.poll(Instant::now());
        if let Some(t) = f(remote, got) {
            return t;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("condition never held");
}

fn start(name: &str, policy: Policy) -> Remote {
    Remote::start(
        Config {
            app: format!("command-remote-test-{name}"),
            discovery_dir: None,
            policy,
        },
        || {},
    )
    .expect("start")
}

#[test]
fn a_verb_before_consent_is_refused_and_an_allowed_client_reaches_the_app() {
    let mut remote = start("allowed", Policy::Ask);
    let mut client = Client::open(remote.pipe());
    client.send("run file.save");
    assert_eq!(client.reply().error.as_deref(), Some("not-enabled"));

    client.send("hello tester \"move a door\"");
    let asked = until(&mut remote, |r, _| {
        r.knock().map(|k| (k.client.clone(), k.purpose.clone()))
    });
    assert_eq!(asked, ("tester".to_owned(), "move a door".to_owned()));
    remote.decide(Some(Scope::Once), Instant::now());
    assert!(client.reply().is_ok());
    assert_eq!(remote.connected_client(), Some("tester"));

    client.send("run file.save");
    let request = until(&mut remote, |_, mut got| got.pop());
    assert_eq!(
        (request.verb.as_str(), request.args.as_slice()),
        ("run", &["file.save".to_owned()][..])
    );
    remote.respond(request, Reply::ok("done").with_body(["line".to_owned()]));
    let reply = client.reply();
    assert_eq!((reply.message.as_str(), reply.body.len()), ("done", 1));
    assert_eq!(
        remote
            .log()
            .back()
            .and_then(|e| e.outcome.clone())
            .as_deref(),
        Some("ok done")
    );

    remote.disconnect();
    client.send("run file.save");
    assert_eq!(client.reply().error.as_deref(), Some("disconnected"));
    assert_eq!(remote.connected_client(), None);
}

#[test]
fn a_refused_client_is_told_and_cannot_ask_again_at_once() {
    let mut remote = start("refused", Policy::Ask);
    let mut client = Client::open(remote.pipe());
    client.send("hello pest");
    until(&mut remote, |r, _| r.knock().map(|_| ()));
    remote.decide(None, Instant::now());
    assert_eq!(client.reply().error.as_deref(), Some("refused"));
    client.send("hello pest");
    until(&mut remote, |r, _| {
        r.log()
            .iter()
            .filter(|e| e.asked == "hello")
            .nth(1)
            .map(|_| ())
    });
    let reply = client.reply();
    assert_eq!(
        (reply.error.as_deref(), reply.message.as_str()),
        (Some("refused"), "cooling-off")
    );
    assert!(
        remote.knock().is_none(),
        "a cooling-off client raises no question"
    );
}

#[test]
fn never_refuses_without_asking() {
    let mut remote = start("never", Policy::Never);
    let mut client = Client::open(remote.pipe());
    client.send("hello anyone");
    until(&mut remote, |r, _| r.log().back().map(|_| ()));
    assert_eq!(client.reply().message, "disabled");
    assert!(remote.knock().is_none());
}
