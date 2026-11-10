//! Other programs MDM starts and waits for, and the files and pages it hands to the system,
//! never waited for on the window's thread (#222): a frame that waits for a program freezes the
//! window for as long as that program takes, and a program can take forever. So a
//! window starts them here, on a thread of their own, each with a time limit, and goes on.

use std::ffi::OsStr;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Hand `target` (a file, a folder or an address) to the system's program for it, on a thread
/// of its own: the `open` crate waits for the launcher it starts (up to 250 ms).
pub fn open(target: impl AsRef<OsStr>) {
    let target = target.as_ref().to_owned();
    let started = std::thread::Builder::new()
        .name("launch".into())
        .spawn(move || {
            let _ = open::that_detached(&target);
        });
    if let Err(e) = started {
        tracing::warn!("open: no thread to run it on: {e}");
    }
}

/// An answer the system may take its time to give (a D-Bus call, a look at a folder that may be
/// on a network share), asked on a thread of its own and kept: a window that needs it every frame
/// reads the last answer, and never waits for more than it says (the same rule as the programs
/// above, for calls).
pub struct Asked<K, T> {
    shared: Arc<(Mutex<AskedState<K, T>>, Condvar)>,
}

struct AskedState<K, T> {
    /// The last answer: for which key, what, and when it came.
    answer: Option<(K, T, Instant)>,
    /// A question is on its way (one at a time: a call that hangs isn't asked again meanwhile).
    asking: bool,
}

impl<K, T> Default for Asked<K, T> {
    fn default() -> Self {
        Asked {
            shared: Arc::new((
                Mutex::new(AskedState {
                    answer: None,
                    asking: false,
                }),
                Condvar::new(),
            )),
        }
    }
}

impl<K, T> Asked<K, T>
where
    K: Clone + PartialEq + Send + 'static,
    T: Clone + PartialEq + Send + 'static,
{
    /// The answer for `key`. The last one is asked again (`ask`, on a thread of its own) once it
    /// is older than `every` or was for another key. None while there is none for `key` yet,
    /// after waiting at most `wait` for it (zero: not at all). `repaint` is repainted when an
    /// answer comes that differs from the last one, so the window shows it.
    pub fn get(
        &self,
        key: &K,
        every: Duration,
        wait: Duration,
        repaint: Option<&egui::Context>,
        ask: impl FnOnce(&K) -> T + Send + 'static,
    ) -> Option<T> {
        let (lock, cv) = &*self.shared;
        let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
        let fresh = s
            .answer
            .as_ref()
            .is_some_and(|(k, _, at)| k == key && at.elapsed() < every);
        if !fresh && !s.asking {
            s.asking = true;
            let shared = self.shared.clone();
            let key = key.clone();
            let repaint = repaint.cloned();
            let started = std::thread::Builder::new()
                .name("asked".into())
                .spawn(move || {
                    let value = ask(&key);
                    let (lock, cv) = &*shared;
                    let mut s = lock.lock().unwrap_or_else(|e| e.into_inner());
                    let changed = s
                        .answer
                        .as_ref()
                        .is_none_or(|(k, v, _)| *k != key || *v != value);
                    s.answer = Some((key, value, Instant::now()));
                    s.asking = false;
                    cv.notify_all();
                    drop(s);
                    if changed && let Some(ctx) = repaint {
                        ctx.request_repaint();
                    }
                });
            if let Err(e) = started {
                tracing::warn!("asked: no thread to ask on: {e}");
                s.asking = false;
            }
        }
        let start = Instant::now();
        loop {
            if let Some((k, v, _)) = &s.answer
                && k == key
            {
                return Some(v.clone());
            }
            let left = wait.saturating_sub(start.elapsed());
            if left.is_zero() || !s.asking {
                return None;
            }
            s = cv
                .wait_timeout(s, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }
}

/// Run `cmd` quietly, waiting at most `limit` for it: true when it ran and succeeded. One that
/// runs out of time is ended. For a thread other than the window's.
pub fn run_within(cmd: &mut Command, limit: Duration) -> bool {
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => wait_within(&mut child, limit).is_some_and(|s| s.success()),
        Err(e) => {
            tracing::debug!("{}: {e}", program_name(cmd));
            false
        }
    }
}

/// Wait for `child` at most `limit`: its exit status, or None when it ran out of time (it is
/// ended and reaped) or couldn't be waited for.
pub(crate) fn wait_within(child: &mut Child, limit: Duration) -> Option<ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if start.elapsed() < limit => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                tracing::warn!("a program still running after {limit:?} was ended");
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Err(e) => {
                tracing::warn!("a program couldn't be waited for: {e}");
                return None;
            }
        }
    }
}

fn program_name(cmd: &Command) -> String {
    cmd.get_program().to_string_lossy().into_owned()
}
