//! Which files on the disk are a torrent's own (upstream a2f58025, its safety part). Before the
//! torrent engine sees a torrent, Grabbit opens every one of its files itself, ticked or not (an
//! unticked file can hold part of a piece a ticked one needs): a file that isn't there is made,
//! empty, and recorded; one that is there is used only when Grabbit made it or the person agreed
//! to use it. The engine writes only through those handles (`mdm_torrent::authorized_storage`).
//! Delete with files removes only what Grabbit made, each file recognised by who it is
//! (`win32::torrent_file_identity`), not by its name, so a file that took the name later stays.
//!
//! The record of each torrent (its ledger) is a small JSON file in the torrent data folder.

use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use mdm_torrent::FileEntry;
use mdm_torrent::authorized_storage::PreparedFile;
use serde::{Deserialize, Serialize};

/// Who a file is: see [`crate::win32::torrent_file_identity`].
pub type Identity = [u64; 4];

/// What Grabbit knows of one torrent's files, by file index.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    /// The files Grabbit made for it, as they were made: Delete with files removes these.
    #[serde(default)]
    pub made: BTreeMap<usize, Identity>,
    /// Files that were there already and the person agreed to use: written, never deleted.
    #[serde(default)]
    pub agreed: BTreeMap<usize, Identity>,
    /// The folders Grabbit made for it, inside its Save to folder: removed with it when empty.
    #[serde(default)]
    pub dirs: Vec<PathBuf>,
}

/// Why a torrent can't start yet.
#[derive(Debug)]
pub enum PrepareError {
    /// Files at its paths that Grabbit didn't make and the person didn't agree to use.
    NotOurs(Vec<PathBuf>),
    Io(io::Error),
}

impl From<io::Error> for PrepareError {
    fn from(e: io::Error) -> Self {
        PrepareError::Io(e)
    }
}

impl PrepareError {
    /// What the list says (the download's error).
    pub fn describe(&self) -> String {
        match self {
            PrepareError::Io(e) => e.to_string(),
            PrepareError::NotOurs(paths) => {
                let first = paths
                    .first()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let mut text = crate::i18n::fmt(
                    // i18n: a torrent didn't start: a file is where it would save one of its files, and Grabbit didn't make it; {file} is its path
                    crate::i18n::t(
                        "Grabbit didn't create {file}, which is where this torrent saves one of its files, so the torrent didn't start and the file wasn't changed. Move or rename it, then start the torrent again.",
                    ),
                    &[("file", &crate::i18n::ltr(&first))],
                );
                if paths.len() > 1 {
                    let n = (paths.len() - 1) as u64;
                    text.push(' ');
                    text.push_str(&crate::i18n::fmt(
                        // i18n: after the sentence above: how many more files are in the same case
                        &crate::i18n::tn(
                            n,
                            "{n} other file is in the same case.",
                            "{n} other files are in the same case.",
                        ),
                        &[("n", &crate::i18n::integer(n))],
                    ));
                }
                text
            }
        }
    }
}

fn ledger_path(dir: &Path, hash: &str) -> PathBuf {
    dir.join(format!("{}.json", hash.to_ascii_lowercase()))
}

/// The torrent's ledger in `dir` (None: there is none yet).
pub fn load(dir: &Path, hash: &str) -> io::Result<Option<Ledger>> {
    match std::fs::read(ledger_path(dir, hash)) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn save(dir: &Path, hash: &str, ledger: &Ledger) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = ledger_path(dir, hash);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(ledger).map_err(io::Error::other)?)?;
    std::fs::rename(&tmp, &path)
}

fn identity(file: &File) -> io::Result<Identity> {
    crate::win32::torrent_file_identity(file)
}

/// Where a torrent keeps its files: `folder` is its own folder when it has several files (Grabbit
/// made it inside Save to), else Save to itself.
fn save_to(folder: &Path, files: &[FileEntry]) -> PathBuf {
    if files.iter().filter(|f| !f.padding).count() > 1 {
        folder.parent().unwrap_or(folder).to_owned()
    } else {
        folder.to_owned()
    }
}

/// Make `dir` and the folders above it that are missing; those strictly inside `base` (the
/// torrent's Save to) are recorded as the torrent's.
fn make_dirs(dir: &Path, base: &Path, ledger: &mut Ledger) -> io::Result<()> {
    let missing: Vec<&Path> = dir.ancestors().take_while(|p| !p.is_dir()).collect();
    for p in missing.into_iter().rev() {
        match std::fs::create_dir(p) {
            Ok(()) => {
                if p.starts_with(base) && p != base && !ledger.dirs.iter().any(|d| d == p) {
                    ledger.dirs.push(p.to_owned());
                }
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn open_rw(path: &Path, create: bool) -> io::Result<File> {
    let mut o = std::fs::OpenOptions::new();
    o.read(true).write(true);
    if create {
        o.create_new(true);
    }
    o.open(path)
}

/// Open every file of the torrent (`files`, its list) at `folder` for the engine. A missing one is
/// made and recorded; one that is there is used only when the ledger has it (made or agreed to)
/// as the very file there. `adopt`: a torrent added before ledgers, with no ledger yet: its own
/// engine wrote what is there (in its own folder, or its one file), which is taken as made by
/// Grabbit, once. The ledger in `dir` is written either way.
pub fn prepare(
    dir: &Path,
    hash: &str,
    folder: &Path,
    files: &[FileEntry],
    adopt: bool,
) -> Result<Vec<PreparedFile>, PrepareError> {
    let found = load(dir, hash)?;
    let adopting = adopt && found.is_none();
    let mut ledger = found.unwrap_or_default();
    let base = save_to(folder, files);
    let mut out = Vec::new();
    let mut refused = Vec::new();
    let mut failed = None;
    for (index, entry) in files.iter().enumerate() {
        if entry.padding {
            continue;
        }
        let path = folder.join(&entry.path);
        match open_rw(&path, false) {
            Ok(file) => {
                let id = identity(&file).ok();
                let known = id.is_some_and(|id| {
                    ledger.made.get(&index) == Some(&id) || ledger.agreed.get(&index) == Some(&id)
                });
                if known {
                    out.push(PreparedFile { index, path, file });
                } else if let (true, Some(id)) = (adopting, id) {
                    ledger.made.insert(index, id);
                    out.push(PreparedFile { index, path, file });
                } else {
                    refused.push(path);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let made = path
                    .parent()
                    .map_or(Ok(()), |p| make_dirs(p, &base, &mut ledger))
                    .and_then(|()| open_rw(&path, true));
                match made {
                    Ok(file) => {
                        match identity(&file) {
                            Ok(id) => {
                                ledger.made.insert(index, id);
                            }
                            // Made just now, so written into; it can't be recognised later.
                            Err(e) => tracing::warn!(
                                "torrents: {}: no identity for a file it made ({e}): it won't be \
                                 deleted with the torrent",
                                path.display()
                            ),
                        }
                        out.push(PreparedFile { index, path, file });
                    }
                    // Another program made it between the look and the make.
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => refused.push(path),
                    Err(e) => {
                        failed = Some(e);
                        break;
                    }
                }
            }
            // A folder at its path, or a file another program holds shut.
            Err(e) => {
                failed = Some(e);
                break;
            }
        }
    }
    // What was made is the torrent's whether or not it starts now.
    save(dir, hash, &ledger)?;
    if let Some(e) = failed {
        return Err(e.into());
    }
    if !refused.is_empty() {
        return Err(PrepareError::NotOurs(refused));
    }
    Ok(out)
}

/// The person agreed to use the files of the torrent already at `folder` (Add torrent asked):
/// each one there now is recorded as agreed to, never deleted.
pub fn agree_existing(
    dir: &Path,
    hash: &str,
    folder: &Path,
    files: &[FileEntry],
) -> io::Result<()> {
    let mut ledger = load(dir, hash)?.unwrap_or_default();
    let mut changed = false;
    for (index, entry) in files.iter().enumerate() {
        if entry.padding {
            continue;
        }
        let Ok(file) = File::open(folder.join(&entry.path)) else {
            continue;
        };
        if let Ok(id) = identity(&file)
            && ledger.made.get(&index) != Some(&id)
        {
            ledger.agreed.insert(index, id);
            changed = true;
        }
    }
    if changed {
        save(dir, hash, &ledger)?;
    }
    Ok(())
}

/// Delete the files Grabbit made for the torrent (still the very files it made), then the empty
/// folders it made, and forget the ledger. Returns the files of the torrent left on the disk:
/// agreed to, or another file now at a path of one it made. Without a ledger: `legacy`, a torrent
/// added before ledgers, goes as it always did (its files by name, in its own folder or its one
/// file); else nothing of it was ever made.
pub fn delete(
    dir: &Path,
    hash: &str,
    folder: &Path,
    files: &[FileEntry],
    legacy: bool,
) -> io::Result<Vec<PathBuf>> {
    let Some(ledger) = load(dir, hash)? else {
        if legacy {
            delete_legacy(folder, files);
        }
        return Ok(Vec::new());
    };
    let mut kept = Vec::new();
    let path_of = |index: &usize| {
        files
            .get(*index)
            .filter(|f| !f.padding)
            .map(|f| folder.join(&f.path))
    };
    for (index, id) in &ledger.made {
        let Some(path) = path_of(index) else { continue };
        match crate::win32::remove_owned_torrent_file(&path, *id) {
            Ok(true) => {}
            Ok(false) => kept.push(path),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!("torrents: {} could not be deleted: {e}", path.display());
                kept.push(path);
            }
        }
    }
    for index in ledger.agreed.keys() {
        if let Some(path) = path_of(index).filter(|p| p.exists()) {
            kept.push(path);
        }
    }
    let mut dirs = ledger.dirs.clone();
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        // Only an empty one goes.
        let _ = std::fs::remove_dir(d);
    }
    forget(dir, hash);
    Ok(kept)
}

/// Forget the torrent's ledger (it left the list).
pub fn forget(dir: &Path, hash: &str) {
    let _ = std::fs::remove_file(ledger_path(dir, hash));
}

/// A torrent added before ledgers: its files by name, then the empty folders between them and
/// its own folder, that folder included when it has one (several files).
fn delete_legacy(folder: &Path, files: &[FileEntry]) {
    let mut dirs = Vec::new();
    for f in files.iter().filter(|f| !f.padding) {
        let path = folder.join(&f.path);
        if path.is_file() {
            let _ = std::fs::remove_file(&path);
        }
        dirs.extend(
            path.ancestors()
                .skip(1)
                .take_while(|p| p.starts_with(folder) && *p != folder)
                .map(Path::to_path_buf),
        );
    }
    dirs.sort();
    dirs.dedup();
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let _ = std::fs::remove_dir(d);
    }
    if files.iter().filter(|f| !f.padding).count() > 1 {
        let _ = std::fs::remove_dir(folder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str) -> FileEntry {
        FileEntry {
            path: path.into(),
            size: 5,
            padding: false,
        }
    }

    const HASH: &str = "0123456789abcdef0123456789abcdef01234567";

    /// A two-file torrent saved in `save`'s own folder "Set", its ledger in `data`.
    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, Vec<FileEntry>) {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("owned");
        let folder = tmp.path().join("save").join("Set");
        std::fs::create_dir_all(tmp.path().join("save")).unwrap();
        (tmp, data, folder, vec![entry("a.bin"), entry("sub/b.bin")])
    }

    #[test]
    fn missing_files_are_made_empty_recorded_and_used_again() {
        let (_tmp, data, folder, files) = setup();
        let got = prepare(&data, HASH, &folder, &files, false).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(std::fs::read(folder.join("a.bin")).unwrap(), b"");
        let ledger = load(&data, HASH).unwrap().unwrap();
        assert_eq!(ledger.made.len(), 2);
        assert!(ledger.agreed.is_empty());
        // Its own folder and the one inside it, not Save to.
        assert!(ledger.dirs.contains(&folder));
        assert!(ledger.dirs.contains(&folder.join("sub")));
        assert!(!ledger.dirs.contains(&folder.parent().unwrap().to_owned()));
        drop(got);
        // Written into meanwhile, still its own.
        std::fs::write(folder.join("a.bin"), b"piece").unwrap();
        assert_eq!(
            prepare(&data, HASH, &folder, &files, false).unwrap().len(),
            2
        );
    }

    #[test]
    fn a_file_grabbit_did_not_make_is_refused_and_left_as_it_is() {
        let (_tmp, data, folder, files) = setup();
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("a.bin"), b"mine").unwrap();
        let Err(PrepareError::NotOurs(paths)) = prepare(&data, HASH, &folder, &files, false) else {
            panic!("an unrecorded file was used");
        };
        assert_eq!(paths, [folder.join("a.bin")]);
        assert_eq!(std::fs::read(folder.join("a.bin")).unwrap(), b"mine");
        // The other one was made, and is recorded.
        assert!(load(&data, HASH).unwrap().unwrap().made.contains_key(&1));
        assert!(
            PrepareError::NotOurs(paths)
                .describe()
                .contains("didn't create")
        );
    }

    #[test]
    fn a_file_swapped_for_another_under_the_same_name_is_refused() {
        let (tmp, data, folder, files) = setup();
        drop(prepare(&data, HASH, &folder, &files, false).unwrap());
        std::fs::rename(folder.join("a.bin"), tmp.path().join("moved")).unwrap();
        std::fs::write(folder.join("a.bin"), b"someone else's").unwrap();
        assert!(matches!(
            prepare(&data, HASH, &folder, &files, false),
            Err(PrepareError::NotOurs(p)) if p == [folder.join("a.bin")]
        ));
    }

    #[test]
    fn agreed_files_are_used_and_kept_by_delete() {
        let (_tmp, data, folder, files) = setup();
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("a.bin"), b"mine").unwrap();
        agree_existing(&data, HASH, &folder, &files).unwrap();
        let got = prepare(&data, HASH, &folder, &files, false).unwrap();
        assert_eq!(got.len(), 2);
        drop(got);
        let kept = delete(&data, HASH, &folder, &files, false).unwrap();
        assert_eq!(kept, [folder.join("a.bin")]);
        assert_eq!(std::fs::read(folder.join("a.bin")).unwrap(), b"mine");
        assert!(
            !folder.join("sub").exists(),
            "the empty folder it made goes"
        );
        assert!(folder.is_dir(), "its folder holds the agreed file");
        assert!(load(&data, HASH).unwrap().is_none());
    }

    #[test]
    fn delete_removes_only_the_files_it_made_still_the_same() {
        let (tmp, data, folder, files) = setup();
        drop(prepare(&data, HASH, &folder, &files, false).unwrap());
        // Another file takes the name of one it made.
        std::fs::rename(folder.join("a.bin"), tmp.path().join("moved")).unwrap();
        std::fs::write(folder.join("a.bin"), b"theirs").unwrap();
        std::fs::write(folder.join("unrelated.txt"), b"keep").unwrap();
        let kept = delete(&data, HASH, &folder, &files, false).unwrap();
        assert_eq!(kept, [folder.join("a.bin")]);
        assert_eq!(std::fs::read(folder.join("a.bin")).unwrap(), b"theirs");
        assert!(!folder.join("sub/b.bin").exists());
        assert!(!folder.join("sub").exists());
        assert!(folder.join("unrelated.txt").exists());
        assert!(folder.parent().unwrap().is_dir(), "Save to stays");
    }

    #[test]
    fn an_older_torrent_takes_what_is_there_once() {
        let (_tmp, data, folder, files) = setup();
        std::fs::create_dir_all(folder.join("sub")).unwrap();
        std::fs::write(folder.join("a.bin"), b"downloaded").unwrap();
        drop(prepare(&data, HASH, &folder, &files, true).unwrap());
        assert_eq!(load(&data, HASH).unwrap().unwrap().made.len(), 2);
        // A ledger now: adopting again changes nothing, and the files go with it.
        delete(&data, HASH, &folder, &files, true).unwrap();
        assert!(!folder.join("a.bin").exists());
    }

    #[test]
    fn an_older_torrent_without_a_ledger_is_deleted_by_name_a_new_one_keeps_everything() {
        let (_tmp, data, folder, files) = setup();
        std::fs::create_dir_all(folder.join("sub")).unwrap();
        std::fs::write(folder.join("a.bin"), b"x").unwrap();
        std::fs::write(folder.join("sub/b.bin"), b"y").unwrap();
        delete(&data, HASH, &folder, &files, false).unwrap();
        assert!(
            folder.join("a.bin").exists(),
            "nothing of a new one was made"
        );
        delete(&data, HASH, &folder, &files, true).unwrap();
        assert!(
            !folder.exists(),
            "an older one goes as before, its own folder too"
        );
    }
}
