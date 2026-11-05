//! Peer authentication for the MA4 evidence channel. Neither side trusts a path, an environment
//! value or what the other side says about itself: each reads its peer's process identity from
//! the kernel (SO_PEERCRED). The assembler accepts only a server that is one of its own ancestors
//! and runs the same executable file as itself: the genuine xtask host, which serves only under a
//! captured isolated route. The host answers only processes descended from the executor it
//! spawned. Other platforms have no such anchor and refuse.
use std::os::unix::net::UnixStream;

const NOT_HOST: &str = "the MA4 evidence channel is not served by this invocation's controlling \
                        xtask host; a forged or replaced channel is not an installed route";

/// Client side: the server must be the controlling xtask host of this process.
pub(super) fn authenticate_server(stream: &UnixStream) -> Result<(), String> {
    linux::authenticate_server(stream)
}

/// Server side: the client must be the admitted executor or one of its descendants.
pub(super) fn authenticate_client(stream: &UnixStream, executor: u32) -> Result<(), String> {
    if executor == 0 {
        return Err("no executor is running for this MA4 invocation".into());
    }
    linux::authenticate_client(stream, executor)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{fs, os::unix::fs::MetadataExt};

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

    /// Whether `pid` runs the same executable file (device and inode) as this process.
    pub(in super::super) fn same_exe(pid: u32) -> bool {
        let id = |path: String| fs::metadata(path).ok().map(|m| (m.dev(), m.ino()));
        match (id(format!("/proc/{pid}/exe")), id("/proc/self/exe".into())) {
            (Some(peer), Some(own)) => peer == own,
            _ => false,
        }
    }

    pub(super) fn authenticate_server(stream: &UnixStream) -> Result<(), String> {
        let server = peer_pid(stream)?;
        let own = std::process::id();
        let lineage = if server == own {
            trusts_own_process()
        } else {
            parent(own).is_some_and(|p| descends_from(p, server))
        };
        if lineage && same_exe(server) {
            Ok(())
        } else {
            Err(NOT_HOST.into())
        }
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
    pub(super) fn authenticate_server(_stream: &UnixStream) -> Result<(), String> {
        Err(format!(
            "{NOT_HOST}: peer authentication is available only on Linux"
        ))
    }
    pub(super) fn authenticate_client(_stream: &UnixStream, _executor: u32) -> Result<(), String> {
        Err("the MA4 evidence channel authenticates its executor only on Linux".into())
    }
}
#[cfg(all(test, target_os = "linux"))]
pub(super) use linux::{descends_from, same_exe};

#[cfg(all(test, target_os = "linux"))]
thread_local!(static TRUST_OWN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });

/// In-process unit tests run the host and the assembler in one process. A test may trust its own
/// process as the host for its own thread; production never does.
#[cfg(all(test, target_os = "linux"))]
pub(super) fn trust_own_process() {
    TRUST_OWN.with(|t| t.set(true));
}
#[cfg(all(test, target_os = "linux"))]
fn trusts_own_process() -> bool {
    TRUST_OWN.with(|t| t.get())
}
#[cfg(all(not(test), target_os = "linux"))]
fn trusts_own_process() -> bool {
    false
}
