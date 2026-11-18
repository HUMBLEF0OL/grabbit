//! `misterdownloadmanager.exe`: the original MDM's program name, kept in Grabbit's zips (plan
//! 10c). It starts `grabbit.exe` from its own folder with the same arguments and ends.
//!
//! An original MDM installed from a zip updates itself by unpacking the new zip over its folder
//! and starting `misterdownloadmanager.exe /updated` (its own updater, which can't change): this
//! program is what it starts, and Grabbit takes over from there. Shortcuts and scripts made for
//! the old name keep opening Grabbit too. A windowed program, as the app is: no console opens.
#![windows_subsystem = "windows"]

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let Some(program) = program() else {
        std::process::exit(1)
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    // The command line (an original MDM's launcher, `--cli=<id>`): waited for, its exit code
    // passed on, the launcher's console still there for it. Anything else (a window, a command
    // for the running copy) is started and left: Grabbit runs on its own.
    let command_line = args
        .first()
        .and_then(|a| a.to_str())
        .is_some_and(|a| a == "--cli" || a.starts_with("--cli="));
    let mut cmd = Command::new(program);
    cmd.args(&args);
    let code = if command_line {
        cmd.status().map_or(1, |s| s.code().unwrap_or(1))
    } else {
        i32::from(cmd.spawn().is_err())
    };
    std::process::exit(code)
}

/// `grabbit.exe` in this program's folder; started through a link, in the folder it points into.
fn program() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let name = format!("grabbit{}", std::env::consts::EXE_SUFFIX);
    [Some(exe.clone()), std::fs::canonicalize(&exe).ok()]
        .into_iter()
        .flatten()
        .map(|e| e.with_file_name(&name))
        .find(|p| p.is_file())
}
