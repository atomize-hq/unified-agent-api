//! The invocation-local channel: a Unix socket in the host's private staging directory, served by
//! the live MA4 host for exactly one candidate while its executor runs. When the invocation ends
//! the socket is gone, so nothing copied out of it (bundle, path or locator) can be replayed.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

pub(super) struct Server {
    path: PathBuf,
    #[cfg(unix)]
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(unix)]
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    pub(super) fn locator(&self) -> &Path {
        &self.path
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        os::unix::net::{UnixListener, UnixStream},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };
    const TIMEOUT: Duration = Duration::from_secs(30);

    impl Server {
        pub(in super::super) fn start(
            path: &Path,
            candidate: PathBuf,
            answer: Result<Offer, String>,
        ) -> Result<Self, String> {
            let listener = UnixListener::bind(path)
                .map_err(|e| format!("cannot open the MA4 evidence channel: {e}"))?;
            listener.set_nonblocking(true).map_err(|e| e.to_string())?;
            let stop = Arc::new(AtomicBool::new(false));
            let thread = {
                let stop = stop.clone();
                std::thread::spawn(move || {
                    while !stop.load(Ordering::SeqCst) {
                        match listener.accept() {
                            Ok((stream, _)) => {
                                let _ = answer_one(stream, &candidate, &answer);
                            }
                            Err(_) => std::thread::sleep(Duration::from_millis(10)),
                        }
                    }
                })
            };
            Ok(Self {
                path: path.to_owned(),
                stop,
                thread: Some(thread),
            })
        }
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
            let _ = fs::remove_file(&self.path);
        }
    }

    fn answer_one(
        stream: UnixStream,
        candidate: &Path,
        answer: &Result<Offer, String>,
    ) -> std::io::Result<()> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        let mut line = String::new();
        BufReader::new((&stream).take(65_536)).read_line(&mut line)?;
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request)
                if fs::canonicalize(&request.candidate).ok().as_deref() == Some(candidate) =>
            {
                match answer {
                    Ok(offer) => Response::Offer(offer.clone()),
                    Err(reason) => Response::Refused(format!(
                        "no authenticated evidence was staged for this invocation: {reason}"
                    )),
                }
            }
            Ok(_) => {
                Response::Refused("this MA4 invocation staged no evidence for that root".into())
            }
            Err(_) => Response::Refused("malformed evidence channel request".into()),
        };
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
        let mut stream = UnixStream::connect(locator).map_err(|_| not_live())?;
        stream
            .set_read_timeout(Some(TIMEOUT))
            .and_then(|()| stream.set_write_timeout(Some(TIMEOUT)))
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
        let mut line = String::new();
        BufReader::new((&stream).take(1_048_576))
            .read_line(&mut line)
            .map_err(|_| not_live())?;
        match serde_json::from_str(&line).map_err(|_| not_live())? {
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
        Err("the MA4 evidence channel requires Unix sockets".into())
    }
}
#[cfg(not(unix))]
pub(super) fn request(_locator: &Path, _candidate: &Path) -> Result<Offer, String> {
    Err("the MA4 evidence channel requires Unix sockets".into())
}
