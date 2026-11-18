//! The launcher, end to end: it runs the app's program from its own folder with `--cli` (on
//! Windows with its process id), the same arguments, input and output, and ends with its exit
//! code. This test's own program stands in for the app's, copied next to a copy of the launcher
//! (`harness = false`: it has its own `main`, which answers when the launcher starts it).

use std::io::{Read, Write};
use std::process::{Command, Stdio};

const LAUNCHER: &str = env!("CARGO_BIN_EXE_grabbit-cli");

/// The original MDM's program name, which starts grabbit.exe (plan 10c, src/old_name.rs).
const OLD_NAME: &str = env!("CARGO_BIN_EXE_misterdownloadmanager");

fn main() {
    // Standing in for grabbit.exe started by the old name: its arguments, to a file.
    if let Some(out) = std::env::var_os("LAUNCHER_TEST_ARGS_TO") {
        let args: Vec<String> = std::env::args().skip(1).collect();
        std::fs::write(out, args.join("|")).unwrap();
        std::process::exit(3);
    }
    if let Some(launch) = mdm_cli::launched(std::env::args_os().collect()) {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        println!("args {:?}", launch.args);
        println!("launcher {:?}", launch.launcher);
        println!("input {input}");
        eprintln!("an error");
        std::process::exit(7);
    }

    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::consts::EXE_SUFFIX;
    let launcher = dir.path().join(format!("grabbit-cli{exe}"));
    let program = dir.path().join(format!("grabbit{exe}"));
    std::fs::copy(LAUNCHER, &launcher).unwrap();
    std::fs::copy(std::env::current_exe().unwrap(), &program).unwrap();

    let mut child = Command::new(&launcher)
        .args(["get", "a b", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    child.stdin.take().unwrap().write_all(b"typed").unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(7), "{stdout}");
    assert!(
        stdout.contains(r#"args ["get", "a b", "--json"]"#),
        "{stdout}"
    );
    // Windows: the launcher waits for the program, which knows it; elsewhere it became it.
    let expected = Some(pid);
    assert!(
        stdout.contains(&format!("launcher {expected:?}")),
        "{stdout}"
    );
    assert!(stdout.contains("input typed"), "{stdout}");
    assert_eq!(String::from_utf8_lossy(&out.stderr).trim(), "an error");

    // Without the app's program: a failure, as the commands report one.
    std::fs::remove_file(&program).unwrap();
    let out = Command::new(&launcher)
        .args(["--json", "status"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let e: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        e["error"].as_str().unwrap().contains("isn't next to"),
        "{e}"
    );
    let out = Command::new(&launcher).arg("status").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("Error: "));

    // Plan 10c: the old name starts grabbit.exe beside it with the same arguments (an original
    // MDM's own update starts `misterdownloadmanager.exe /updated`), and leaves it running.
    let old = dir.path().join(format!("misterdownloadmanager{exe}"));
    std::fs::copy(OLD_NAME, &old).unwrap();
    std::fs::copy(std::env::current_exe().unwrap(), &program).unwrap();
    let args_to = dir.path().join("args.txt");
    let out = Command::new(&old)
        .args(["/updated", "a b"])
        .env("LAUNCHER_TEST_ARGS_TO", &args_to)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "started, not waited for");
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !args_to.is_file() && std::time::Instant::now() < until {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert_eq!(std::fs::read_to_string(&args_to).unwrap(), "/updated|a b");
    // An original MDM's command-line launcher: waited for, its exit code passed on.
    std::fs::remove_file(&args_to).unwrap();
    let out = Command::new(&old)
        .args(["--cli=1", "status"])
        .env("LAUNCHER_TEST_ARGS_TO", &args_to)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(std::fs::read_to_string(&args_to).unwrap(), "--cli=1|status");
    // No grabbit.exe beside it: it fails, starting nothing.
    std::fs::remove_file(&program).unwrap();
    let out = Command::new(&old).arg("/updated").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    println!("launcher: ok");
}
