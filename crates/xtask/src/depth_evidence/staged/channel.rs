//! The invocation-local channel: a Unix socket in the host's private staging directory, served by
//! the live MA4 host for exactly one candidate while its executor runs. Both sides authenticate
//! the other's process (see `peer`). The host revokes the channel the moment its executor exits,
//! so nothing copied out of it (bundle, path or locator) can be replayed by a later process.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Offer {
    pub bundle: PathBuf,
    pub sha256: String,
    pub invocation: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    candidate: PathBuf,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
enum Response {
    Offer(Offer),
    Refused(String),
}

struct Shared {
    revoked: AtomicBool,
    /// The spawned executor's pid; 0 while none runs.
    executor: AtomicU32,
    /// The socket and the bundle; both are removed on revocation.
    paths: Vec<PathBuf>,
}

/// The host's control over its channel: admit the executor it spawned, and revoke the channel
/// when that executor exits. A revoked channel never admits or answers again.
#[derive(Clone)]
pub(crate) struct Admission(Arc<Shared>);
impl Admission {
    pub(crate) fn admit(&self, pid: u32) {
        if !self.0.revoked.load(Ordering::SeqCst) {
            self.0.executor.store(pid, Ordering::SeqCst);
        }
    }
    pub(crate) fn revoke(&self) {
        self.0.revoked.store(true, Ordering::SeqCst);
        self.0.executor.store(0, Ordering::SeqCst);
        for path in &self.0.paths {
            let _ = fs::remove_file(path);
        }
    }
}
impl std::fmt::Debug for Admission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Admission")
            .field("revoked", &self.0.revoked.load(Ordering::SeqCst))
            .field("executor", &self.0.executor.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

pub(super) struct Server {
    path: PathBuf,
    admission: Admission,
    #[cfg(unix)]
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    pub(super) fn locator(&self) -> &Path {
        &self.path
    }
    pub(super) fn admission(&self) -> &Admission {
        &self.admission
    }
}
fn admission(path: &Path, answer: &Result<Offer, String>) -> Admission {
    let mut paths = vec![path.to_owned()];
    if let Ok(offer) = answer {
        paths.push(offer.bundle.clone());
    }
    Admission(Arc::new(Shared {
        revoked: AtomicBool::new(false),
        executor: AtomicU32::new(0),
        paths,
    }))
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        io::{ErrorKind, Read, Write},
        os::unix::net::{UnixListener, UnixStream},
        time::{Duration, Instant},
    };
    /// Absolute bound on one request; a trickling peer never holds the host longer.
    const SERVE_DEADLINE: Duration = Duration::from_secs(10);
    const CLIENT_DEADLINE: Duration = Duration::from_secs(30);
    const POLL: Duration = Duration::from_millis(25);

    impl Server {
        pub(in super::super) fn start(
            path: &Path,
            candidate: PathBuf,
            answer: Result<Offer, String>,
        ) -> Result<Self, String> {
            let listener = UnixListener::bind(path)
                .map_err(|e| format!("cannot open the MA4 evidence channel: {e}"))?;
            listener.set_nonblocking(true).map_err(|e| e.to_string())?;
            let admission = admission(path, &answer);
            let thread = {
                let shared = admission.0.clone();
                std::thread::spawn(move || {
                    while !shared.revoked.load(Ordering::SeqCst) {
                        match listener.accept() {
                            Ok((stream, _)) => {
                                let _ = answer_one(stream, &candidate, &answer, &shared);
                            }
                            Err(_) => std::thread::sleep(POLL),
                        }
                    }
                })
            };
            Ok(Self {
                path: path.to_owned(),
                admission,
                thread: Some(thread),
            })
        }
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.admission.revoke();
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    /// Read one newline-terminated line of at most `limit` bytes before `deadline`, giving up as
    /// soon as `stop` is set.
    fn read_line(
        stream: &UnixStream,
        deadline: Instant,
        stop: &AtomicBool,
        limit: usize,
    ) -> std::io::Result<Vec<u8>> {
        let mut line = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let now = Instant::now();
            if stop.load(Ordering::SeqCst) || now >= deadline {
                return Err(ErrorKind::TimedOut.into());
            }
            stream.set_read_timeout(Some(POLL.min(deadline - now)))?;
            match (&*stream).read(&mut chunk) {
                Ok(0) => return Ok(line),
                Ok(n) => {
                    line.extend_from_slice(&chunk[..n]);
                    if let Some(end) = line.iter().position(|b| *b == b'\n') {
                        line.truncate(end);
                        return Ok(line);
                    }
                    if line.len() > limit {
                        return Err(ErrorKind::InvalidData.into());
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn answer_one(
        stream: UnixStream,
        candidate: &Path,
        answer: &Result<Offer, String>,
        shared: &Shared,
    ) -> std::io::Result<()> {
        let deadline = Instant::now() + SERVE_DEADLINE;
        stream.set_nonblocking(false)?;
        let client = super::super::peer::authenticate_client(
            &stream,
            shared.executor.load(Ordering::SeqCst),
        );
        let response = match client {
            Err(reason) => Response::Refused(reason),
            Ok(()) => {
                let line = read_line(&stream, deadline, &shared.revoked, 65_536)?;
                match serde_json::from_slice::<Request>(&line) {
                    // Equality with this host's own canonical root only; the request path is
                    // never resolved.
                    Ok(request) if request.candidate == candidate => match answer {
                        Ok(offer) => Response::Offer(offer.clone()),
                        Err(reason) => Response::Refused(format!(
                            "no authenticated evidence was staged for this invocation: {reason}"
                        )),
                    },
                    Ok(_) => Response::Refused(
                        "this MA4 invocation staged no evidence for that root".into(),
                    ),
                    Err(_) => Response::Refused("malformed evidence channel request".into()),
                }
            }
        };
        let now = Instant::now();
        if shared.revoked.load(Ordering::SeqCst) || now >= deadline {
            return Ok(());
        }
        stream.set_write_timeout(Some(deadline - now))?;
        let mut stream = stream;
        writeln!(stream, "{}", serde_json::to_string(&response)?)
    }

    pub(in super::super) fn request(locator: &Path, candidate: &Path) -> Result<Offer, String> {
        use std::os::unix::fs::FileTypeExt;
        let not_live = || {
            "the MA4 invocation that staged evidence is not live; a copied bundle, path or \
             locator is not an installed route"
                .to_string()
        };
        match fs::symlink_metadata(locator) {
            Ok(m) if m.file_type().is_socket() => {}
            _ => return Err(not_live()),
        }
        let deadline = Instant::now() + CLIENT_DEADLINE;
        let mut stream = UnixStream::connect(locator).map_err(|_| not_live())?;
        // Nothing is sent to, or accepted from, a server that is not the controlling host.
        super::super::peer::authenticate_server(&stream)?;
        stream
            .set_write_timeout(Some(CLIENT_DEADLINE))
            .map_err(|e| e.to_string())?;
        let request = Request {
            candidate: candidate.to_owned(),
        };
        writeln!(
            stream,
            "{}",
            serde_json::to_string(&request).map_err(|e| e.to_string())?
        )
        .map_err(|_| not_live())?;
        let line = read_line(&stream, deadline, &AtomicBool::new(false), 1_048_576)
            .map_err(|_| not_live())?;
        match serde_json::from_slice(&line).map_err(|_| not_live())? {
            Response::Offer(offer) => Ok(offer),
            Response::Refused(reason) => Err(reason),
        }
    }
}
#[cfg(unix)]
pub(super) use unix::request;

#[cfg(not(unix))]
impl Server {
    pub(super) fn start(
        _path: &Path,
        _candidate: PathBuf,
        _answer: Result<Offer, String>,
    ) -> Result<Self, String> {
        let _ = admission;
        Err("the MA4 evidence channel requires Unix sockets".into())
    }
}
#[cfg(not(unix))]
pub(super) fn request(_locator: &Path, _candidate: &Path) -> Result<Offer, String> {
    Err("the MA4 evidence channel requires Unix sockets".into())
}
