# Contributing

Thanks for helping with Mister Download Manager. Bug reports, ideas and pull requests are all
welcome.

## Getting set up

You need Windows 10 or 11 and Rust stable 1.95 or later (the MSVC toolchain).

```
cargo build
cargo run -p mdm-app
```

To try changes without touching your own downloads, give the app a separate data folder:

```
set MDM_DATA_DIR=C:\some\empty\folder
cargo run -p mdm-app
```

`cargo run -p mdm-labserver -- --faults "rate=2M"` starts a local test server that serves
generated files with whatever trouble you ask for (slow connections, cuts, busy answers, no
ranges; `GET /` lists them), so downloads can be tested without the internet.

## Branches

- **`dev`** is where work lands. Branch from it and open your pull request against it.
- **`main`** is what's released: the website publishes from it and release tags (`vX.Y.Z`) are
  cut on it. It only moves by a pull request from `dev` (a release) or a hotfix.
- **Hotfix:** branch from `main`, open the pull request against `main`, then merge `main` back
  into `dev`.

## Before you open a pull request

These are the checks CI runs. CI runs only on pull requests into `main` (a release, or a
hotfix), to spend GitHub Actions minutes once per release: **run them yourself before a pull
request into `dev` merges**, or before you push to `dev`. Each works on a clean clone:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node tools/extension/test.mjs
python tools/upstream-sync/test_sync.py
```

Keep changes focused and match the code around them. A few rules hold everywhere:

- **Clean room.** MDM contains no code, binaries, artwork, icons, sounds or text from Internet
  Download Manager or any other download manager. Never disassemble, decompile or patch another
  program, and never bypass anyone's licensing. See
  [Not a reverse-engineering project](README.md#not-a-reverse-engineering-project).
- **Every text the user sees goes through `crate::i18n`** (`t`, `tc`, `tn`, `fmt`). MDM is
  English only, so they return the English text. Data inside a sentence (a file name, an
  address, a size) goes in through `i18n::ltr` or `i18n::isolate`, so a file name in Arabic or
  Hebrew can't reorder the sentence around it.
- **MDM is Windows only. Windows-only calls live in `crates/app/src/win32.rs`**; the rest of the
  workspace is safe, portable Rust (`unsafe_code = "deny"`).
- **Fix causes, not cases**, and keep producers and consumers of shared data (the download list,
  the browser extension's messages, the catalogs) in step.

## Languages

MDM, its installer, the browser extension and the website are in English only. The translations
(86 languages), the language picker and the translation tools are kept on the
`archive/multiplatform-i18n` branch.

## Reporting problems

Use **Help → Report a problem** in the app: it opens the bug report form with your version,
system and how MDM was installed filled in. The form asks for the same when you
open it yourself. Security problems: see [SECURITY.md](SECURITY.md).

Pull requests are accepted under the project's licence, the
[GNU GPL, version 3 or later](LICENSE).
