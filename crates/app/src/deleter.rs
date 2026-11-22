//! Downloads' files deleted beside the window (upstream issue 1170). Delete and Shift+Delete,
//! Delete All Completed and the automation API's `delete` take their rows off the list at once;
//! the files go here, on a thread of their own, one batch after the other in the order they
//! were deleted. Windows can take seconds over one delete (a disk waking up, an antivirus
//! scanning the file, a network or USB drive, a file another program holds), and the window
//! waited for each: 8 s in the report, for one file.
//!
//! A deleted download's place stays held ([`mdm_engine::naming::reserve`]) from the moment its
//! row leaves the list until its files are gone, so a download added meanwhile under that name
//! (or a video under that stem) is numbered as it would be for a file still there: what is
//! deleted here is only ever what the deleted download had, never a file of one that came after.
//!
//! A finished file goes to the Recycle Bin, where it can be restored from (plan 12a); the
//! download's working data (its resume state, part file, a video's pieces and leftovers) is
//! deleted outright.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::{Duration, Instant};

use mdm_engine::naming::{self, Reservation};

/// Files found when their download's turn comes, not when it is deleted (a video's leftovers:
/// looking for them reads its folder).
pub type Later = Box<dyn FnOnce() -> Vec<PathBuf> + Send>;

/// How a file is deleted (`std::fs::remove_file`, [`recycle`] for a finished file; the tests'
/// own).
type Remove = dyn Fn(&Path) -> io::Result<()> + Send + Sync;

/// How a finished file goes ([`recycle`]; the tests' own): with `true`, its place may be a
/// folder of its own (a debrid torrent's files).
type Trash = dyn Fn(&Path, bool) -> io::Result<()> + Send + Sync;

/// A finished file to the Recycle Bin (`trash`: [`crate::extractq::to_trash`]; a test's folder).
/// A file already gone is NotFound, as `remove_file` says it; one the shell kept is an error.
/// Only a file goes, or with `own_folder` a folder Grabbit made for the download (a debrid
/// torrent's): anything else at `path` (a gallery's folder, a link) stays, untouched, and is
/// said. Only NotFound counts as gone: a file that can't be looked at (access denied, a network
/// share dropped) is a failure, never taken for deleted.
fn recycle(path: &Path, own_folder: bool, trash: impl Fn(&[PathBuf]) -> bool) -> io::Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if !(meta.is_file() || own_folder && meta.is_dir()) {
        tracing::warn!(
            "delete: {} is not a file; it stays (never to the Recycle Bin)",
            path.display()
        );
        return Err(io::Error::other(
            // i18n: why a deleted download's place stayed (after "{file} could not be moved to the Recycle Bin: "); it is a folder or a link
            crate::i18n::t("It is not a file, so it was left as it is"),
        ));
    }
    if trash(&[path.to_owned()]) {
        return Ok(());
    }
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
        Ok(_) => {
            // The shell gives no reason: a file open in another program, or a path longer than
            // the 260 characters it takes. The file stays; it is never deleted outright instead.
            tracing::warn!(
                "delete: {} ({} characters) not moved to the Recycle Bin; it stays",
                path.display(),
                path.as_os_str().len()
            );
            Err(io::Error::other(
                // i18n: why a deleted download's file stayed (after "{file} could not be moved to the Recycle Bin: "); Windows gives no reason
                crate::i18n::t("Windows kept it (it may be open in another program)"),
            ))
        }
    }
}

/// What goes from the disk with one download taken off the list.
pub struct Doomed {
    /// Its name as the list shows it: said when its file can't go.
    name: String,
    /// Where it is saved.
    place: PathBuf,
    /// It was complete: its finished file at `place` goes to the Recycle Bin, and one that
    /// can't is said.
    finished: bool,
    /// What else it left there, looked for in its turn.
    later: Option<Later>,
    /// `place`, held from the moment its row left the list. None: another download in this
    /// process held it then (one of this row's that has just stopped and is letting go), or it
    /// is being cleared for a download to start there (`clearing`).
    hold: Option<Reservation>,
    /// Its files go so that a download starts at `place` afresh ([`Doomed::clearing`]): the
    /// downloads starting in its folder wait for them meanwhile.
    clearing: Option<naming::Clearing>,
    /// `place` may be a folder Grabbit made for it (a debrid torrent's files), which goes whole.
    own_folder: bool,
}

impl Doomed {
    /// A download's files, as its row leaves the list (on the window's thread: no disk here).
    pub fn new(name: String, place: PathBuf, finished: bool, later: Option<Later>) -> Doomed {
        let hold = naming::reserve(&place);
        Doomed {
            name,
            place,
            finished,
            later,
            hold,
            clearing: None,
            own_folder: false,
        }
    }

    /// Its place is a folder Grabbit made for it (a debrid torrent's): it goes whole when it is
    /// a folder.
    pub fn own_folder(mut self, yes: bool) -> Doomed {
        self.own_folder = yes;
        self
    }

    /// A download's files deleted for a download that starts at `place` afresh, under the same
    /// name (a Redownload, a duplicate overwritten, a restart under the same name). Its name
    /// isn't held, so that the next download takes it; instead the downloads that start in its
    /// folder, and the looks that number a name there, wait until these files are gone
    /// ([`naming::clear`]): none starts writing at `place` before then, nor finds the old state.
    pub fn clearing(name: String, place: PathBuf, finished: bool, later: Option<Later>) -> Doomed {
        let clearing = Some(naming::clear(&place));
        Doomed {
            name,
            place,
            finished,
            later,
            hold: None,
            clearing,
            own_folder: false,
        }
    }

    /// What goes, in order: its resume state and part file, what it left beside them, then its
    /// finished file (last: its name stays taken on the disk as long as anything of it is
    /// there). Its leftovers are looked for here, so this is in its turn.
    fn files(&mut self) -> (Vec<PathBuf>, Option<PathBuf>) {
        let mut working = vec![
            mdm_engine::state::state_path(&self.place),
            naming::part_path(&self.place),
        ];
        for f in self.later.take().map(|l| l()).unwrap_or_default() {
            if !working.contains(&f) && !(self.finished && f == self.place) {
                working.push(f);
            }
        }
        let finished = self.finished.then(|| self.place.clone());
        (working, finished)
    }
}

/// A finished file that couldn't be moved to the Recycle Bin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub name: String,
    pub error: String,
}

/// How long a deleted download's place may stay held by its own run that has just stopped,
/// before the place is taken for another download's and its files are left alone.
const LET_GO: Duration = Duration::from_secs(2);

/// One download's files: its resume state and part file, what it left beside them, then its
/// finished file, last, so its name stays taken on the disk as long as anything of it is there.
/// The working data goes with `remove`, the finished file with `trash` (the Recycle Bin). A file
/// already gone is no failure, and only the finished file's are told: the rest is the
/// download's own working data.
fn delete_one(mut d: Doomed, remove: &Remove, trash: &Trash) -> Option<Failure> {
    // Paused a moment before and still writing to a slow disk (upstream issue 1328): its last
    // write and its resume state come first, so neither comes back after the delete.
    naming::wait_settled(&d.place);
    let hold = d.hold.take().or_else(|| {
        let until = Instant::now() + LET_GO;
        loop {
            if let Some(h) = naming::reserve(&d.place) {
                break Some(h);
            }
            if Instant::now() >= until {
                break None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    });
    let Some(hold) = hold else {
        tracing::warn!(
            "delete: another download is saving as {} now; the deleted one's files stay",
            d.place.display()
        );
        return None;
    };
    let (working, finished) = d.files();
    for f in working {
        // A video's pieces from the engine wait in a folder of their own (upstream 6196f9ad).
        let _ = if f.is_dir() {
            std::fs::remove_dir_all(&f)
        } else {
            remove(&f)
        };
    }
    let failed = finished.and_then(|f| match trash(&f, d.own_folder) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Some(Failure {
            name: std::mem::take(&mut d.name),
            error: e.to_string(),
        }),
        _ => None,
    });
    // Its place let go first, then the downloads waiting in its folder go on: the one that
    // starts there finds its name free.
    drop(hold);
    drop(d.clearing.take());
    failed
}

/// A batch, in order; the finished files that couldn't go.
fn delete_batch(batch: Vec<Doomed>, remove: &Remove, trash: &Trash) -> Vec<Failure> {
    batch
        .into_iter()
        .filter_map(|d| delete_one(d, remove, trash))
        .collect()
}

/// What is said of a batch's failures: one message however many there were, the first named.
pub fn failures_text(failures: &[Failure]) -> Option<String> {
    let first = failures.first()?;
    let mut text = crate::i18n::fmt(
        // i18n: a message: a deleted download's finished file stayed on the disk; {file} = its name
        crate::i18n::t("{file} could not be moved to the Recycle Bin: {error}"),
        &[
            ("file", &crate::i18n::isolate(&first.name)),
            ("error", &first.error),
        ],
    );
    let more = failures.len() as u64 - 1;
    if more > 0 {
        text.push('\n');
        text += &crate::i18n::tn(
            // i18n: the message's next line: how many more deleted downloads' files stayed on the disk
            more,
            "{n} more file could not be moved either.",
            "{n} more files could not be moved either.",
        );
    }
    Some(text)
}

/// The thread that deletes, and the batches it has yet to finish.
pub struct Deleter {
    /// None when no thread could be made: then each batch goes on the caller's.
    tx: Option<mpsc::Sender<Vec<Doomed>>>,
    remove: Arc<Remove>,
    trash: Arc<Trash>,
    report: Arc<dyn Fn(Vec<Failure>) + Send + Sync>,
    pending: Arc<(Mutex<usize>, Condvar)>,
}

impl Deleter {
    /// Working data deleted from the disk, finished files moved to the Recycle Bin; a batch's
    /// failures told in the main window's notice (`dialogs::tell`), which `wake` has drawn.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Deleter {
        // ponytail: an App's own deleter in the tests deletes outright, so they never fill the
        // real Recycle Bin; the Recycle Bin path is tested with a folder of its own below.
        let trash: Arc<Trash> = if cfg!(test) {
            Arc::new(|p: &Path, _| std::fs::remove_file(p))
        } else {
            Arc::new(|p: &Path, own| recycle(p, own, crate::extractq::to_trash))
        };
        Deleter::with_trash(
            Arc::new(|p: &Path| std::fs::remove_file(p)),
            trash,
            move |failures| {
                if let Some(text) = failures_text(&failures) {
                    crate::dialogs::tell(crate::i18n::mark("Delete"), text);
                    wake();
                }
            },
        )
    }

    /// [`Deleter::new`] with its own way to delete a file, finished or not, and to say what
    /// failed (tests).
    #[cfg(test)]
    pub(crate) fn with(
        remove: impl Fn(&Path) -> io::Result<()> + Send + Sync + 'static,
        report: impl Fn(Vec<Failure>) + Send + Sync + 'static,
    ) -> Deleter {
        let remove: Arc<Remove> = Arc::new(remove);
        let both = remove.clone();
        Deleter::with_trash(remove, Arc::new(move |p: &Path, _| both(p)), report)
    }

    /// [`Deleter::new`] with its own ways to delete the working data (`remove`) and the
    /// finished file (`trash`), and to say what failed.
    pub(crate) fn with_trash(
        remove: Arc<Remove>,
        trash: Arc<Trash>,
        report: impl Fn(Vec<Failure>) + Send + Sync + 'static,
    ) -> Deleter {
        let report: Arc<dyn Fn(Vec<Failure>) + Send + Sync> = Arc::new(report);
        let pending: Arc<(Mutex<usize>, Condvar)> = Arc::default();
        let (tx, rx) = mpsc::channel::<Vec<Doomed>>();
        let spawned = std::thread::Builder::new()
            .name("delete files".into())
            .spawn({
                let (remove, trash) = (remove.clone(), trash.clone());
                let (report, pending) = (report.clone(), pending.clone());
                move || {
                    for batch in rx {
                        let (many, began) = (batch.len(), Instant::now());
                        let failures = delete_batch(batch, &*remove, &*trash);
                        // What a disk held, for a report's log.
                        if began.elapsed() >= Duration::from_secs(1) {
                            tracing::info!(
                                "delete: {many} deleted download(s)' files took {} ms, beside \
                                 the window",
                                began.elapsed().as_millis()
                            );
                        }
                        if !failures.is_empty() {
                            report(failures);
                        }
                        let (n, done) = &*pending;
                        *n.lock().unwrap() -= 1;
                        done.notify_all();
                    }
                }
            });
        let tx = match spawned {
            Ok(_) => Some(tx),
            Err(e) => {
                tracing::warn!("delete: no thread to delete files on ({e}); they go at once");
                None
            }
        };
        Deleter {
            tx,
            remove,
            trash,
            report,
            pending,
        }
    }

    /// Delete these downloads' files, after the batches before. Returns at once.
    pub fn delete(&self, batch: Vec<Doomed>) {
        if batch.is_empty() {
            return;
        }
        *self.pending.0.lock().unwrap() += 1;
        let batch = match &self.tx {
            Some(tx) => match tx.send(batch) {
                Ok(()) => return,
                Err(mpsc::SendError(batch)) => batch,
            },
            None => batch,
        };
        let failures = delete_batch(batch, &*self.remove, &*self.trash);
        if !failures.is_empty() {
            (self.report)(failures);
        }
        *self.pending.0.lock().unwrap() -= 1;
    }

    /// Wait until every batch handed over is deleted, for at most `wait` (the end: a disk that
    /// holds a delete doesn't hold the exit). True when none is left.
    pub fn finish(&self, wait: Duration) -> bool {
        let (n, done) = &*self.pending;
        let left = n.lock().unwrap();
        let (left, _) = done.wait_timeout_while(left, wait, |n| *n > 0).unwrap();
        *left == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doomed(place: &Path, finished: bool) -> Doomed {
        Doomed::new(
            place.file_name().unwrap().to_string_lossy().into_owned(),
            place.to_owned(),
            finished,
            None,
        )
    }

    /// A deleter whose deletes are recorded (and then made by `remove`), its failures sent back.
    #[allow(clippy::type_complexity)]
    fn recorded(
        remove: impl Fn(&Path) -> io::Result<()> + Send + Sync + 'static,
    ) -> (
        Deleter,
        Arc<Mutex<Vec<PathBuf>>>,
        mpsc::Receiver<Vec<Failure>>,
    ) {
        let seen: Arc<Mutex<Vec<PathBuf>>> = Arc::default();
        let (tx, rx) = mpsc::channel();
        let tx = Mutex::new(tx);
        let deleter = Deleter::with(
            {
                let seen = seen.clone();
                move |p| {
                    seen.lock().unwrap().push(p.to_owned());
                    remove(p)
                }
            },
            move |f| {
                let _ = tx.lock().unwrap().send(f);
            },
        );
        (deleter, seen, rx)
    }

    /// Each download's files in turn, in the order deleted, batch after batch: its state and
    /// part file, then what it left (the engine's pieces folder too), then its finished file
    /// (an unfinished one has none).
    #[test]
    fn files_go_in_order_the_finished_one_last() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let (a, b, c) = (d.join("a.zip"), d.join("b.iso"), d.join("c.mp4"));
        for f in [
            &a,
            &naming::part_path(&b),
            &mdm_engine::state::state_path(&b),
            &d.join("c.f137.mp4"),
        ] {
            std::fs::write(f, b"x").unwrap();
        }
        let pieces = d.join("c.mp4.mdm.pieces");
        std::fs::create_dir(&pieces).unwrap();
        std::fs::write(pieces.join("0"), b"x").unwrap();
        let (deleter, seen, failures) = recorded(|p| std::fs::remove_file(p));
        let left = vec![d.join("c.f137.mp4"), pieces.clone()];
        let later: Later = Box::new(move || left);
        deleter.delete(vec![doomed(&a, true), doomed(&b, false)]);
        deleter.delete(vec![Doomed::new(
            "c.mp4".into(),
            c.clone(),
            true,
            Some(later),
        )]);
        assert!(deleter.finish(Duration::from_secs(10)));
        let state = mdm_engine::state::state_path;
        assert_eq!(
            *seen.lock().unwrap(),
            vec![
                state(&a),
                naming::part_path(&a),
                a.clone(),
                state(&b),
                naming::part_path(&b),
                state(&c),
                naming::part_path(&c),
                d.join("c.f137.mp4"),
                c.clone(),
            ]
        );
        assert_eq!(std::fs::read_dir(d).unwrap().count(), 0, "all gone");
        // A finished file already gone (c.mp4 never was there) isn't a failure.
        assert!(failures.try_recv().is_err());
    }

    /// A finished file that can't go is said, once a batch for all of them; a state or part
    /// file that can't, or a file gone already, isn't; the rest of the batch still goes.
    #[test]
    fn failures_are_collected_per_batch() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let names = ["held 1.bin", "gone.bin", "held 2.bin", "fine.bin"];
        for n in ["held 1.bin", "held 2.bin", "fine.bin"] {
            std::fs::write(d.join(n), b"x").unwrap();
        }
        let (deleter, seen, failures) = recorded(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            if name.starts_with("held") {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "in use"))
            } else {
                std::fs::remove_file(p)
            }
        });
        deleter.delete(names.iter().map(|n| doomed(&d.join(n), true)).collect());
        let told = failures.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(
            told,
            vec![
                Failure {
                    name: "held 1.bin".into(),
                    error: "in use".into()
                },
                Failure {
                    name: "held 2.bin".into(),
                    error: "in use".into()
                },
            ]
        );
        assert!(deleter.finish(Duration::from_secs(10)));
        assert!(failures.try_recv().is_err(), "one report for the batch");
        assert!(!d.join("fine.bin").exists());
        assert!(seen.lock().unwrap().contains(&d.join("gone.bin")));
        let text = failures_text(&told).unwrap();
        assert!(
            text.contains("held 1.bin") && text.contains("in use"),
            "{text}"
        );
        assert!(
            text.contains("could not be moved to the Recycle Bin"),
            "{text}"
        );
        assert!(text.contains("1 more file could not be moved"), "{text}");
        assert!(!failures_text(&told[..1]).unwrap().contains("more"));
        assert_eq!(failures_text(&[]), None);
    }

    /// The window hands a batch over and goes on: a delete the disk holds for seconds holds the
    /// deleter's thread only. Its place stays taken meanwhile: a download added under its name
    /// is numbered, and the name is free once its files are gone.
    #[test]
    fn a_slow_disk_never_holds_the_caller() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let file = d.join("slow.bin");
        std::fs::write(&file, b"x").unwrap();
        let (go, wait) = mpsc::channel::<()>();
        let wait = Mutex::new(wait);
        let (deleter, _, _) = recorded(move |p| {
            if p.extension().is_some_and(|e| e == "bin") {
                // The disk answers when the test says so (at most 30 s).
                let _ = wait.lock().unwrap().recv_timeout(Duration::from_secs(30));
            }
            std::fs::remove_file(p)
        });
        let began = Instant::now();
        deleter.delete(vec![doomed(&file, true)]);
        deleter.delete(vec![doomed(&d.join("next.bin"), true)]);
        assert!(
            began.elapsed() < Duration::from_millis(500),
            "{:?}",
            began.elapsed()
        );
        assert!(
            !deleter.finish(Duration::from_millis(200)),
            "still deleting"
        );
        assert!(file.exists());
        assert_eq!(naming::unique_path(d, "slow.bin"), d.join("slow (1).bin"));
        go.send(()).unwrap();
        go.send(()).unwrap();
        assert!(deleter.finish(Duration::from_secs(10)));
        assert!(!file.exists());
        assert_eq!(naming::unique_path(d, "slow.bin"), d.join("slow.bin"));
    }

    /// Plan 12a: a deleted download's finished file goes to the Recycle Bin (here a test folder,
    /// as `MDM_TEST_TRASH_DIR` would make it) and its working data is deleted outright; a
    /// finished file already gone is no failure, and one the bin refused stays and is said.
    #[test]
    fn a_finished_file_goes_to_the_recycle_bin_its_working_data_is_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let (d, bin) = (dir.path().join("downloads"), dir.path().join("bin"));
        std::fs::create_dir(&d).unwrap();
        let file = d.join("done.zip");
        let (state, part) = (
            mdm_engine::state::state_path(&file),
            naming::part_path(&file),
        );
        for f in [&file, &state, &part] {
            std::fs::write(f, b"x").unwrap();
        }
        let (tx, failures) = mpsc::channel();
        let tx = Mutex::new(tx);
        let deleter = Deleter::with_trash(
            Arc::new(|p: &Path| std::fs::remove_file(p)),
            Arc::new({
                let bin = bin.clone();
                move |p: &Path, own| recycle(p, own, |f| crate::extractq::into_test_trash(&bin, f))
            }),
            move |f| {
                let _ = tx.lock().unwrap().send(f);
            },
        );
        deleter.delete(vec![doomed(&file, true), doomed(&d.join("gone.zip"), true)]);
        assert!(deleter.finish(Duration::from_secs(10)));
        assert!(bin.join("done.zip").exists(), "in the Recycle Bin");
        assert_eq!(
            std::fs::read_dir(&bin).unwrap().count(),
            1,
            "only the finished file"
        );
        assert_eq!(
            std::fs::read_dir(&d).unwrap().count(),
            0,
            "state and part deleted"
        );
        assert!(
            failures.try_recv().is_err(),
            "a file already gone is no failure"
        );
        // The bin refuses: the file stays, and is said.
        std::fs::write(&file, b"x").unwrap();
        let kept = recycle(&file, false, |_| false).unwrap_err();
        assert_ne!(kept.kind(), io::ErrorKind::NotFound);
        assert!(file.exists());
    }

    /// Only a file goes to the Recycle Bin: a folder at a finished download's place (a
    /// gallery's) stays with all it holds, and is said; a place that can't be looked at (here a
    /// name Windows refuses) is a failure, not a file already gone.
    #[test]
    fn a_folder_or_a_place_that_cannot_be_read_never_goes() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("gallery");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("1.jpg"), b"x").unwrap();
        let never = |_: &[PathBuf]| -> bool { panic!("not handed to the Recycle Bin") };
        let kept = recycle(&folder, false, never).unwrap_err();
        assert_ne!(kept.kind(), io::ErrorKind::NotFound);
        assert!(kept.to_string().contains("not a file"), "{kept}");
        assert!(folder.join("1.jpg").exists());
        let unreadable = recycle(&dir.path().join("bad|name.zip"), false, never).unwrap_err();
        assert_ne!(unreadable.kind(), io::ErrorKind::NotFound, "{unreadable}");
        // Through the deleter: said as a failure, the folder untouched.
        let (tx, rx) = mpsc::channel();
        let tx = Mutex::new(tx);
        let deleter = Deleter::with_trash(
            Arc::new(|p: &Path| std::fs::remove_file(p)),
            Arc::new(move |p: &Path, own| recycle(p, own, never)),
            move |f| {
                let _ = tx.lock().unwrap().send(f);
            },
        );
        deleter.delete(vec![Doomed::clearing(
            "gallery".into(),
            folder.clone(),
            true,
            None,
        )]);
        assert!(deleter.finish(Duration::from_secs(10)));
        assert_eq!(rx.try_recv().unwrap().len(), 1, "said");
        assert!(folder.join("1.jpg").exists());
        // A folder Grabbit made for the download (a debrid torrent's files) goes whole.
        let bin = dir.path().join("bin");
        recycle(&folder, true, |f| crate::extractq::into_test_trash(&bin, f)).unwrap();
        assert!(
            bin.join("gallery").join("1.jpg").exists(),
            "in the Recycle Bin, whole"
        );
        assert!(!folder.exists());
    }

    /// A place another download holds (for longer than its own run takes to let go) is that
    /// download's now: nothing of it is deleted.
    #[test]
    fn a_place_another_download_holds_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let place = dir.path().join("taken.bin");
        let part = naming::part_path(&place);
        std::fs::write(&part, b"another download's").unwrap();
        let other = naming::reserve(&place).unwrap();
        let (deleter, seen, _) = recorded(|p| std::fs::remove_file(p));
        deleter.delete(vec![doomed(&place, false)]);
        assert!(deleter.finish(LET_GO + Duration::from_secs(10)));
        assert!(part.exists());
        assert!(seen.lock().unwrap().is_empty());
        drop(other);
        // One whose own run lets go meanwhile: its files go.
        let held = naming::reserve(&place).unwrap();
        deleter.delete(vec![doomed(&place, false)]);
        std::thread::sleep(Duration::from_millis(200));
        drop(held);
        assert!(deleter.finish(Duration::from_secs(10)));
        assert!(!part.exists());
    }
}
