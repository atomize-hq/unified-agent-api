//! Host-side admission for the MA4 evidence channel. The host reads its peer's process from the
//! kernel (SO_PEERCRED) and answers only the executor it spawned or one of that executor's
//! descendants; the request must also carry the invocation's nonce. Other platforms have no such
//! anchor and refuse every client.
//!
//! The assembler does not authenticate the host: any check of a socket's listener is forgeable by
//! a same-user process. Its channel use is a fail-closed convenience. The authority is the host's
//! captured route plus byte binding at delivery, so a forged run can write only an undeliverable
//! local E.
use std::os::unix::net::UnixStream;

/// The client must be the admitted executor or one of its descendants.
pub(super) fn authenticate_client(stream: &UnixStream, executor: u32) -> Result<(), String> {
    if executor == 0 {
        return Err("no executor is running for this MA4 invocation".into());
    }
    linux::authenticate_client(stream, executor)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;

    fn peer_pid(stream: &UnixStream) -> Result<u32, String> {
        let cred = rustix::net::sockopt::socket_peercred(stream)
            .map_err(|e| format!("cannot read the evidence channel peer: {e}"))?;
        u32::try_from(cred.pid.as_raw_nonzero().get()).map_err(|e| e.to_string())
    }

    /// The parent of `pid`, from `/proc/<pid>/stat` (the field after the command name).
    fn parent(pid: u32) -> Option<u32> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let (_, rest) = stat.rsplit_once(')')?;
        rest.split_whitespace().nth(1)?.parse().ok()
    }

    /// Whether `pid` is `ancestor` or descends from it.
    pub(in super::super) fn descends_from(mut pid: u32, ancestor: u32) -> bool {
        for _ in 0..4096 {
            if pid == ancestor {
                return true;
            }
            if pid <= 1 {
                return false;
            }
            match parent(pid) {
                Some(next) => pid = next,
                None => return false,
            }
        }
        false
    }

    pub(super) fn authenticate_client(stream: &UnixStream, executor: u32) -> Result<(), String> {
        if descends_from(peer_pid(stream)?, executor) {
            Ok(())
        } else {
            Err("the MA4 evidence channel serves only this invocation's executor".into())
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod linux {
    use super::*;
    pub(super) fn authenticate_client(_stream: &UnixStream, _executor: u32) -> Result<(), String> {
        Err("the MA4 evidence channel admits its executor only on Linux".into())
    }
}
#[cfg(all(test, target_os = "linux"))]
pub(super) use linux::descends_from;
