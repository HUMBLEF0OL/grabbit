//! Subtitles a site refuses never cost the video (#68), against a fake yt-dlp that answers as the
//! real one did for YouTube's automatic English captions: `Unable to download video subtitles for
//! 'en': HTTP Error 429: Too Many Requests`. Without `--ignore-errors` the fake fails the whole
//! run, as yt-dlp does; with it, it warns and saves the video.
//!
//! One program, three roles (no test harness, `harness = false`):
//! - run as `fake-ytdlp[.exe]` (a copy of itself): it is yt-dlp;
//! - run with `MDM_SUBS_RETRY_SECS` set (the retry's pause, shortened): it runs the checks;
//! - run plainly (`cargo test`): it runs itself again with that variable, the way to set it
//!   without touching its own environment.
//!
//! #769: with `page.json` beside it, the fake's page is that one (its formats' addresses on a lab
//! server, `{EXPIRES}` replaced by a time a little ahead each look-up); asked for its choice
//! (`-O "MDMJ %()j"`) it answers as yt-dlp does after choosing; and asked to download a choice
//! whose streams are already there (MDM's engine downloaded them), it merges them as yt-dlp
//! does ("has already been downloaded"), downloading only what isn't there.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use mdm_tools::video::{State, SubReason, Subs, VideoJob};

fn main() -> ExitCode {
    let me = std::env::current_exe().expect("own path");
    let name = me.file_stem().map(|s| s.to_string_lossy().into_owned());
    if name.as_deref() == Some("fake-ytdlp") {
        return fake_ytdlp();
    }
    if name.as_deref() == Some("fake-ffmpeg") {
        return fake_ffmpeg();
    }
    if name.as_deref() == Some("fake-ffprobe") {
        return fake_ffprobe();
    }
    if std::env::var_os("MDM_SUBS_RETRY_SECS").is_some() {
        held_output_checks(&me);
        checks(&me);
        job_checks(&me);
        merge_checks(&me);
        list_checks(&me);
        engine_checks(&me);
        recording_checks(&me);
        println!("fake_ytdlp: ok");
        return ExitCode::SUCCESS;
    }
    let status = Command::new(&me)
        .env("MDM_SUBS_RETRY_SECS", "1")
        .status()
        .expect("run the checks");
    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// What the fake does, from a file beside the output: `refuse-always` (English is refused every
/// time) or `refuse-once` (the first time only).
fn mode(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("mode.txt")).unwrap_or_default()
}

/// A process the fake leaves behind (as yt-dlp's launcher leaves its child, and ffmpeg): it
/// writes to `file` every 100 ms for `secs` (30 s unless said), unless ended.
fn linger(file: &Path, secs: Option<&str>) -> ExitCode {
    use std::io::Write;
    let secs = secs.and_then(|s| s.parse().ok()).unwrap_or(30);
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
        {
            let _ = f.write_all(b"x");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    ExitCode::SUCCESS
}

/// #1521: a process the fake leaves behind holding its stdout and stderr open for 90 s
/// ([`linger`], writing to `file`), as an ffmpeg yt-dlp started and shares its output with can.
fn holder(file: &Path) {
    let me = std::env::current_exe().unwrap();
    let _ = Command::new(me)
        .args(["--linger", &file.to_string_lossy(), "90"])
        .stdin(std::process::Stdio::null())
        .spawn();
}

/// Raw bytes on stdout (a line in another code page than UTF-8, #79).
fn out_bytes(b: &[u8]) {
    use std::io::Write;
    let mut o = std::io::stdout().lock();
    let _ = o.write_all(b);
    let _ = o.flush();
}

fn fake_ytdlp() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--linger") {
        return linger(Path::new(&args[1]), args.get(2).map(String::as_str));
    }
    let after = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let has = |flag: &str| args.iter().any(|a| a == flag);
    // #69: `https://list.example/playlist/<n>` (its size said) or `.../channel/<n>` (not).
    if let Some(list) = args
        .last()
        .and_then(|u| u.strip_prefix("https://list.example/"))
    {
        return fake_list(list, has("-J"), after("--playlist-items"));
    }
    // #352: yt-dlp's answer when the cookies leave it nothing to offer.
    if args
        .last()
        .is_some_and(|u| u == "https://cookies.example/none")
        && has("--cookies")
    {
        eprintln!(
            "ERROR: [youtube] abc: Requested format is not available. Use --list-formats for a list of available formats"
        );
        return ExitCode::FAILURE;
    }
    let home = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_owned))
        .expect("the fake's folder");
    if has("-J") {
        const PAGE: &str = r#"{"title": "Talk", "language": "ru", "formats": [
            {"format_id": "18", "ext": "mp4", "height": 360, "vcodec": "avc1", "acodec": "mp4a"}],
            "automatic_captions": {"en": [{"name": "English"}], "ru": [{"name": "Russian"}]}}"#;
        // Upstream issue 852: YouTube looked up without a JavaScript runtime, as yt-dlp says it.
        if args
            .last()
            .is_some_and(|u| u == "https://nojs.example/watch")
        {
            eprint!(
                "{}",
                include_str!("fixtures/ytdlp-js-runtime/no-runtime.stderr")
            );
        }
        // Each look-up counted beside the fake.
        let file = home.join("looked.txt");
        let n = std::fs::read_to_string(&file)
            .ok()
            .and_then(|t| t.trim().parse::<u32>().ok())
            .unwrap_or(0);
        let _ = std::fs::write(&file, (n + 1).to_string());
        // Every look-up after the first slow (3 s), when asked: a moment to act in between runs.
        if n >= 1 && home.join("slow-second-look-up").exists() {
            std::thread::sleep(Duration::from_secs(3));
        }
        // #1521: a process left behind holding the output open, when asked.
        if home.join("hold-output").exists() {
            holder(&home.join("linger.log"));
        }
        // Upstream issue 769: a page of the test's, its links signed for a little while from now.
        match std::fs::read_to_string(home.join("page.json")) {
            Ok(page) => {
                let ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                println!("{}", page.replace("{EXPIRES}", &(ms + 1500).to_string()));
            }
            Err(_) => println!("{PAGE}"),
        }
        return ExitCode::SUCCESS;
    }
    // #769: the choice (`-O "MDMJ %()j"`), as yt-dlp prints it after choosing the formats.
    if after("-O").is_some_and(|o| o.starts_with("MDMJ")) {
        let info =
            std::fs::read_to_string(after("--load-info-json").expect("a page")).expect("the page");
        let info: serde_json::Value = serde_json::from_str(&info).expect("json");
        let template = after("-o").expect("-o").replace("%%", "%");
        let chosen = choice(&info, &after("-f").unwrap_or_default(), &template);
        println!("MDMJ {chosen}");
        return ExitCode::SUCCESS;
    }
    let template = after("-o").expect("-o");
    let dir = Path::new(&template).parent().expect("a folder").to_owned();
    let mode = mode(&dir);
    let tries = dir.join("tries.txt");
    let tried = std::fs::read_to_string(&tries)
        .ok()
        .and_then(|t| t.trim().parse::<u32>().ok())
        .unwrap_or(0);
    std::fs::write(&tries, (tried + 1).to_string()).unwrap();
    let format = after("-f").unwrap_or_default();
    // A live stream, recorded as yt-dlp's ffmpeg records one: into `<stem>.mp4.part` (written
    // over, `-y`), MPEG-TS (`live`; `live-raw`: not), growing, saying nothing of its progress,
    // never ending by itself (a minute at most). `live-dns-once`: its first run fails a while
    // in, a media server no longer found. `live-ends`: the stream ends 3 s in, and yt-dlp
    // finishes the file itself. `live-split`: the picture and the sound recorded apart.
    // `live-late`: as ffmpeg opens its file late and writes its first block later still: after
    // `MDMQ` it waits (30 s at most) for `open.txt` in the folder, creates the `.part` empty, then
    // waits (30 s at most) for `go.txt` before it writes as `live` does.
    if mode.starts_with("live") {
        use std::io::Write;
        println!("MDMQ 96 1280x720");
        let part = format!(
            "{}.part",
            template.replace("%(ext)s", "mp4").replace("%%", "%")
        );
        let mut packet = [0u8; 188];
        if mode != "live-raw" {
            packet[0] = 0x47;
        }
        let chunk = packet.repeat(348);
        // `live-split`: the picture and the sound apart, each a stream's own part (the sound's
        // a third as big).
        if mode == "live-split" {
            let stream = |id: &str, ext: &str| {
                format!(
                    "{}.part",
                    template
                        .replace("%(ext)s", &format!("f{id}.{ext}"))
                        .replace("%%", "%")
                )
            };
            let mut video = std::fs::File::create(stream("1", "mp4")).unwrap();
            let mut audio = std::fs::File::create(stream("2", "m4a")).unwrap();
            let end = Instant::now() + Duration::from_secs(60);
            while Instant::now() < end {
                video.write_all(&chunk).unwrap();
                audio.write_all(&chunk[..116 * 188]).unwrap();
                std::thread::sleep(Duration::from_millis(100));
            }
            eprintln!("ERROR: the fake live stream went on too long");
            return ExitCode::FAILURE;
        }
        // `live-late`: a file that isn't there until the test says (`open.txt`) and holds nothing
        // until it says again (`go.txt`).
        let wait_for = |name: &str| {
            let until = Instant::now() + Duration::from_secs(30);
            while !dir.join(name).exists() {
                if Instant::now() >= until {
                    eprintln!("ERROR: the fake live stream waited too long for {name}");
                    return false;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            true
        };
        if mode == "live-late" && !wait_for("open.txt") {
            return ExitCode::FAILURE;
        }
        let mut file = std::fs::File::create(&part).unwrap();
        if mode == "live-late" && !wait_for("go.txt") {
            return ExitCode::FAILURE;
        }
        let end = Instant::now() + Duration::from_secs(60);
        let mut written = 0;
        while Instant::now() < end {
            file.write_all(&chunk).unwrap();
            file.flush().unwrap();
            written += chunk.len();
            if mode == "live-dns-once" && tried == 0 && written >= 256 << 10 {
                eprintln!(
                    "ERROR: [download] Got error: HTTPSConnection(host='rr1---sn-dead.googlevideo.com', port=443): Failed to resolve 'rr1---sn-dead.googlevideo.com' ([Errno 11004] getaddrinfo failed). Giving up after 10 retries"
                );
                return ExitCode::FAILURE;
            }
            // The stream ends (3 s in): as yt-dlp says it once its ffmpeg is done, all of it
            // in, then the file named and fixed up (`FixupM3u8`: an MP4 now), and said.
            if mode == "live-ends" && written >= 30 * chunk.len() {
                drop(file);
                println!("MDMP {written} {written} NA 650000 96");
                let video = template.replace("%(ext)s", "mp4").replace("%%", "%");
                std::fs::rename(&part, &video).unwrap();
                if args.iter().any(|a| a.starts_with("post_process:MDMM")) {
                    println!("MDMM {video}");
                }
                let mut fixed = b"FAKEMP4 ".to_vec();
                fixed.extend(std::fs::read(&video).unwrap());
                std::fs::write(&video, fixed).unwrap();
                println!("MDMF {video}");
                return ExitCode::SUCCESS;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        eprintln!("ERROR: the fake live stream went on too long");
        return ExitCode::FAILURE;
    }
    // #94: the chosen height isn't offered (the reroute left it out); a fallback is.
    if mode == "unavailable" && format.contains("[height=") {
        eprintln!(
            "ERROR: [youtube] abc: Requested format is not available. Use --list-formats for a list of available formats"
        );
        return ExitCode::FAILURE;
    }
    // #94: a media server that resolved when looked up doesn't when downloading, once.
    if mode == "dns-once" && tried == 0 {
        eprintln!(
            "ERROR: [download] Got error: HTTPSConnection(host='rr1---sn-dead.googlevideo.com', port=443): Failed to resolve 'rr1---sn-dead.googlevideo.com' ([Errno 11004] getaddrinfo failed). Giving up after 10 retries"
        );
        return ExitCode::FAILURE;
    }
    println!("MDMP 1024 4096 NA 1000");
    println!("MDMQ 18 640x360");
    // #79: it falls over mid-way, leaving a helper running (Windows: the launcher's child).
    if mode == "crash" {
        let me = std::env::current_exe().unwrap();
        let _ = Command::new(me)
            .args(["--linger", &dir.join("linger.log").to_string_lossy()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        std::thread::sleep(Duration::from_millis(500));
        eprintln!("ERROR: unable to download video data: HTTP Error 500");
        return ExitCode::FAILURE;
    }
    // #1521: it ends (well, or falling over) leaving behind a process that still holds its
    // output open, as an ffmpeg yt-dlp started and shares its output with can.
    if mode == "held" || mode == "held-crash" {
        holder(&dir.join("linger.log"));
        if mode == "held-crash" {
            eprintln!("ERROR: unable to download video data: HTTP Error 500");
            return ExitCode::FAILURE;
        }
    }
    // #79: lines that aren't UTF-8, before and as the final path (a legacy code page).
    if mode == "odd-lines" {
        out_bytes(b"\xff\xfe [download] Destination: \xde\xfd.mp4\n");
        println!("not one of our lines");
        println!("MDMP NA NA NA NA");
    }
    // #79: silent for a while, as when ffmpeg merges a long video.
    if let Some(secs) = mode.strip_prefix("quiet:") {
        std::thread::sleep(Duration::from_secs(secs.trim().parse().unwrap_or(1)));
    }
    // #736: a video and its sound, merged as yt-dlp's ffmpeg does: into `<stem>.temp.mp4`,
    // growing for a while, then renamed. Said first when asked (`post_process:` print).
    if mode == "merge" {
        println!("MDMQ 137+140 1920x1080");
        println!("MDMP 3000 3000 NA 1000 137");
        println!("MDMP 1000 1000 NA 1000 140");
        let video = template.replace("%(ext)s", "mp4").replace("%%", "%");
        if args.iter().any(|a| a.starts_with("post_process:MDMM")) {
            println!("MDMM {video}");
        }
        let temp = video.replace(".mp4", ".temp.mp4");
        for k in 1..=10u64 {
            std::fs::write(&temp, vec![0u8; (k * 400) as usize]).unwrap();
            std::thread::sleep(Duration::from_millis(300));
        }
        std::fs::rename(&temp, &video).unwrap();
        println!("MDMF {video}");
        return ExitCode::SUCCESS;
    }
    let langs = after("--sub-langs").unwrap_or_default();
    for lang in langs.split(',').filter(|l| !l.is_empty()) {
        let refused = lang == "en" && (mode == "refuse-always" || tried == 0);
        if refused {
            let why =
                "Unable to download video subtitles for 'en': HTTP Error 429: Too Many Requests";
            if has("--ignore-errors") {
                eprintln!("WARNING: {why}");
                continue;
            }
            eprintln!("ERROR: {why}");
            return ExitCode::FAILURE;
        }
        let file = template.replace("%(ext)s", &format!("{lang}.srt"));
        std::fs::write(file, "1\n00:00:00,000 --> 00:00:01,000\nHello\n").unwrap();
    }
    // Upstream issue 769: a choice whose streams the engine downloaded: merged as yt-dlp merges
    // them.
    if let Some(info) = after("--load-info-json")
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .filter(|i| i["formats"][0]["url"].is_string())
        && !has("--skip-download")
    {
        let template = template.replace("%%", "%");
        let chosen = choice(&info, &format, &template);
        let output = chosen["filename"].as_str().unwrap().to_owned();
        let streams: Vec<String> = match chosen["requested_formats"].as_array() {
            Some(fs) => fs
                .iter()
                .map(|f| {
                    output.replace(
                        ".mp4",
                        &format!(
                            ".f{}.{}",
                            f["format_id"].as_str().unwrap(),
                            f["ext"].as_str().unwrap()
                        ),
                    )
                })
                .collect(),
            None => vec![output.clone()],
        };
        let mut merged = Vec::new();
        for s in &streams {
            match std::fs::read(s) {
                // "has already been downloaded"
                Ok(b) => merged.extend(b),
                Err(_) => merged.extend(b"downloaded by yt-dlp"),
            }
        }
        for s in streams.iter().filter(|s| **s != output) {
            let _ = std::fs::remove_file(s);
        }
        // #736: several streams are merged as yt-dlp's ffmpeg does, said first when asked
        // (`post_process:` print), into `<stem>.temp.mp4`, growing for a while, then renamed.
        if streams.len() > 1 && args.iter().any(|a| a.starts_with("post_process:MDMM")) {
            println!("MDMQ {} 1920x1080", chosen["format_id"].as_str().unwrap());
            println!("MDMM {output}");
            let temp = output.replace(".mp4", ".temp.mp4");
            let step = merged.len().div_ceil(5);
            for k in 1..=5 {
                std::fs::write(&temp, &merged[..(k * step).min(merged.len())]).unwrap();
                std::thread::sleep(Duration::from_millis(300));
            }
            std::fs::rename(&temp, &output).unwrap();
        } else {
            std::fs::write(&output, merged).unwrap();
        }
        println!("MDMF {output}");
        return ExitCode::SUCCESS;
    }
    if !has("--skip-download") {
        let video = template.replace("%(ext)s", "mp4").replace("%%", "%");
        std::fs::write(&video, vec![0u8; 4096]).unwrap();
        if mode == "odd-lines" {
            // The path in a one-byte code page: every letter beyond ASCII one byte, not UTF-8.
            let mut line = b"MDMF ".to_vec();
            line.extend(
                video
                    .chars()
                    .map(|c| if c.is_ascii() { c as u8 } else { 0xde }),
            );
            line.push(b'\n');
            out_bytes(&line);
        } else {
            println!("MDMF {video}");
        }
    }
    ExitCode::SUCCESS
}

/// yt-dlp's choice of formats on page `info` for `selector`: a format id the page has, that one;
/// else every format of the page merged (two or more), or the only one. With `filename` from the
/// output `template`.
fn choice(info: &serde_json::Value, selector: &str, template: &str) -> serde_json::Value {
    let formats = info["formats"].as_array().cloned().unwrap_or_default();
    let mut chosen = info.clone();
    let one = formats
        .iter()
        .find(|f| f["format_id"].as_str() == Some(selector))
        .or((formats.len() == 1).then(|| &formats[0]));
    match one {
        Some(f) => {
            for (k, v) in f.as_object().unwrap() {
                chosen[k] = v.clone();
            }
        }
        None => {
            chosen["requested_formats"] = serde_json::Value::Array(formats.clone());
            chosen["ext"] = "mp4".into();
            chosen["format_id"] = formats
                .iter()
                .map(|f| f["format_id"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
                .join("+")
                .into();
        }
    }
    let ext = chosen["ext"].as_str().unwrap_or("mp4").to_owned();
    chosen["filename"] = template.replace("%(ext)s", &ext).into();
    chosen
}

/// A list of `n` videos as yt-dlp lists them: `-J` (flat, one entry) says what it is; flat
/// `-j` prints the entries in `--playlist-items`' range, a line each, a little at a time (as
/// YouTube's pages of 100 come). Every 7th is private.
fn fake_list(list: &str, whole: bool, items: Option<String>) -> ExitCode {
    let (kind, n) = list.split_once('/').unwrap_or((list, "0"));
    // `?fail=<i>`: the site stops answering at entry i (on a listing from the start only: listed
    // again from there, as after a pause, it goes on).
    let (n, fail) = match n.split_once("?fail=") {
        Some((n, f)) => (n, f.parse::<u64>().ok()),
        None => (n, None),
    };
    let n: u64 = n.parse().unwrap_or(0);
    let count = if kind == "playlist" {
        format!(r#", "playlist_count": {n}"#)
    } else {
        String::new()
    };
    if whole {
        println!(r#"{{"_type": "playlist", "title": "A fake {kind}"{count}, "entries": []}}"#);
        return ExitCode::SUCCESS;
    }
    let (from, to) = items
        .as_deref()
        .and_then(|r| r.split_once(':'))
        .map(|(a, b)| (a.parse().unwrap_or(1), b.parse().unwrap_or(n)))
        .unwrap_or((1, n));
    for i in from..=to.min(n) {
        if fail == Some(i) && from == 1 {
            eprintln!(
                "ERROR: [youtube:tab] Unable to download API page: HTTP Error 429: Too Many Requests"
            );
            return ExitCode::FAILURE;
        }
        let title = if i % 7 == 0 {
            "[Private video]".to_owned()
        } else {
            format!("Video {i}")
        };
        println!(
            r#"{{"_type": "url", "url": "https://video.example/v{i}", "title": "{title}", "duration": {}, "playlist_index": {i}, "playlist_title": "A fake {kind}"{count}}}"#,
            60 + i % 600
        );
        if i % 100 == 0 {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    ExitCode::SUCCESS
}

/// What the fake MP4s start with (not MPEG-TS): not counted as what they play.
const MARK: &[u8] = b"FAKEMP4 ";

/// How fast the fake recordings play: what ffprobe (the fake) says of their length.
const BYTES_PER_SECOND: f64 = 100_000.0;

/// MDM's ffprobe asked how long a file plays (its last argument, `file:<path>`), as JSON: its
/// bytes (after [`MARK`]) at [`BYTES_PER_SECOND`], the file's and each stream's (an MP4: a picture
/// and a sound; a recording's part: one). `ffprobe-mode.txt` beside it: `short` (an MP4 plays
/// half as long), `short-sound` (an MP4's sound does), `estimate` (said estimated from the
/// bitrate) or `fail` (it can't tell).
fn fake_ffprobe() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let last = args.last().expect("a file");
    let path = PathBuf::from(last.strip_prefix("file:").unwrap_or(last));
    let mode = std::fs::read_to_string(path.with_file_name("ffprobe-mode.txt")).unwrap_or_default();
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("{}: No such file or directory", path.display());
        return ExitCode::FAILURE;
    };
    if mode.trim() == "fail" {
        eprintln!(
            "{}: Invalid data found when processing input",
            path.display()
        );
        return ExitCode::FAILURE;
    }
    if mode.trim() == "estimate" {
        eprintln!("[mpegts @ 0000] Estimating duration from bitrate, this may be inaccurate");
    }
    let plays = bytes.strip_prefix(MARK).unwrap_or(&bytes).len() as f64 / BYTES_PER_SECOND;
    let mp4 = path.extension().is_some_and(|e| e == "mp4");
    let half = |short: bool| if short && mp4 { plays / 2.0 } else { plays };
    let (file, sound) = (half(mode.trim() == "short"), half(mode.trim() != ""));
    let stream =
        |kind: &str, d: f64| format!(r#"{{"codec_type": "{kind}", "duration": "{d:.6}"}}"#);
    let mut streams = vec![stream("video", file)];
    if mp4 {
        let sound = if mode.trim() == "short-sound" {
            sound
        } else {
            file
        };
        streams.push(stream("audio", sound));
    }
    println!(
        r#"{{"streams": [{}], "format": {{"duration": "{file:.6}"}}}}"#,
        streams.join(", ")
    );
    ExitCode::SUCCESS
}

/// MDM's ffmpeg making a live stream's recording an MP4 (`-i file:<.part> [-i file:<.part>…] …
/// file:<out>`): the recordings copied one after the other after [`MARK`], taking a second. Its
/// runs counted in `ffmpeg-runs.txt` beside the (first) recording; `ffmpeg-mode.txt` there:
/// `fail` (half written, then an error), `tiny` (a few bytes, no error), `short` (four fifths, no
/// error) or `linger` (its file there 2 s before it ends).
fn fake_ffmpeg() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let file = |a: &str| PathBuf::from(a.strip_prefix("file:").unwrap_or(a));
    let inputs: Vec<PathBuf> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "-i")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|a| file(a))
        .collect();
    let input = inputs.first().expect("-i").clone();
    let output = file(args.last().expect("an output"));
    let dir = input.parent().expect("a folder").to_owned();
    let runs = dir.join("ffmpeg-runs.txt");
    let n = std::fs::read_to_string(&runs)
        .ok()
        .and_then(|t| t.trim().parse::<u32>().ok())
        .unwrap_or(0);
    std::fs::write(&runs, (n + 1).to_string()).unwrap();
    let mode = std::fs::read_to_string(dir.join("ffmpeg-mode.txt")).unwrap_or_default();
    let recorded: Vec<u8> = inputs
        .iter()
        .flat_map(|i| std::fs::read(i).expect("the recording"))
        .collect();
    std::thread::sleep(Duration::from_secs(1));
    match mode.trim() {
        "fail" => {
            std::fs::write(&output, &recorded[..recorded.len() / 2]).unwrap();
            eprintln!("[mp4 @ 0000] Could not write header: Invalid argument");
            ExitCode::FAILURE
        }
        "tiny" => {
            std::fs::write(&output, MARK).unwrap();
            ExitCode::SUCCESS
        }
        "short" => {
            let mut made = MARK.to_vec();
            made.extend(&recorded[..recorded.len() * 4 / 5]);
            std::fs::write(&output, made).unwrap();
            ExitCode::SUCCESS
        }
        mode => {
            let mut made = MARK.to_vec();
            made.extend(recorded);
            std::fs::write(&output, made).unwrap();
            // Its file there a while before it ends (`linger`): long enough for another program
            // to open it.
            if mode == "linger" {
                std::thread::sleep(Duration::from_secs(2));
            }
            ExitCode::SUCCESS
        }
    }
}


/// The fake, as a copy of this program named `fake-ytdlp` in `dir`.
fn fake_in(me: &Path, dir: &Path) -> PathBuf {
    let exe = dir.join(format!("fake-ytdlp{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(me, &exe).expect("copy the fake");
    exe
}

/// #1521: yt-dlp ended, but a process it started still holds its output open (an ffmpeg that
/// shares it; here for 90 s): the look-up and the download end with yt-dlp, not with that
/// process, and it is ended with them.
fn held_output_checks(me: &Path) {
    // Far below the holder's 90 s, far above the 2 s the output is read after yt-dlp's end.
    let soon = Duration::from_secs(30);
    // The holder writes to its file every 100 ms until it is ended.
    let ended = |file: &Path, what: &str| {
        let size = || std::fs::metadata(file).map_or(0, |m| m.len());
        std::thread::sleep(Duration::from_millis(800));
        let after = size();
        std::thread::sleep(Duration::from_millis(1500));
        assert_eq!(size(), after, "{what}: what yt-dlp left was ended with it");
    };
    // A look-up: yt-dlp's `-J` output read to its end.
    let tools = tempfile::tempdir().unwrap();
    let exe = fake_in(me, tools.path());
    std::fs::write(tools.path().join("hold-output"), "").unwrap();
    let started = Instant::now();
    let looked =
        mdm_tools::resolve::look_up(&exe, "https://video.example/talk", &Default::default());
    let took = started.elapsed();
    assert!(looked.is_ok(), "look-up: {:?}", looked.err());
    assert!(
        took < soon,
        "the look-up waited {took:?} for what yt-dlp left"
    );
    ended(&tools.path().join("linger.log"), "look-up");
    // A download that ends well, and one that falls over.
    for mode in ["held", "held-crash"] {
        let started = Instant::now();
        let (p, files, out) = download_as(me, mode, &Subs::default(), "b", "Talk");
        let took = started.elapsed();
        assert!(
            took < soon,
            "{mode}: the download waited {took:?} for what yt-dlp left"
        );
        match &p.state {
            State::Done(_) if mode == "held" => assert_eq!(files, ["Talk.mp4"]),
            State::Failed(e) if mode == "held-crash" => {
                assert!(e.contains("HTTP Error 500"), "{e}");
            }
            other => panic!("{mode}: {other:?}"),
        }
        ended(&out.path().join("linger.log"), mode);
    }
}

/// Download with `subs` while the fake behaves as `mode`; the end state, its log and folder.
fn download(me: &Path, mode: &str, subs: &Subs) -> (mdm_tools::video::Progress, Vec<String>) {
    let (p, files, _out) = download_as(me, mode, subs, "b", "Talk");
    (p, files)
}

/// The same with a format `selector` and a file `stem`; also hands back the output folder.
fn download_as(
    me: &Path,
    mode: &str,
    subs: &Subs,
    selector: &str,
    stem: &str,
) -> (mdm_tools::video::Progress, Vec<String>, tempfile::TempDir) {
    let tools = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    std::fs::write(out.path().join("mode.txt"), mode).unwrap();
    let exe = fake_in(me, tools.path());
    let job = VideoJob::start(
        &exe,
        "https://video.example/talk",
        selector,
        out.path(),
        stem,
        None,
        1,
        None,
        None,
        &Default::default(),
        subs,
        None,
    )
    .expect("started");
    let end = Instant::now() + Duration::from_secs(120);
    let p = loop {
        let p = job.progress();
        if p.state != State::Running {
            break p;
        }
        assert!(Instant::now() < end, "still running: {p:?}");
        std::thread::sleep(Duration::from_millis(100));
    };
    let mut files: Vec<String> = std::fs::read_dir(out.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.ends_with(".txt") && !n.ends_with(".log"))
        .collect();
    files.sort();
    (p, files, out)
}

fn log_of(p: &mdm_tools::video::Progress) -> Vec<String> {
    p.events.iter().map(|(_, t)| t.clone()).collect()
}

/// #736: while yt-dlp's ffmpeg joins a video and its sound, the job says so, and how far it is
/// from the joined file's size; it says no more once the file is there.
fn merge_checks(me: &Path) {
    let tools = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    std::fs::write(out.path().join("mode.txt"), "merge").unwrap();
    let exe = fake_in(me, tools.path());
    let job = VideoJob::start(
        &exe,
        "https://video.example/talk",
        "bv*+ba",
        out.path(),
        "Talk",
        None,
        1,
        None,
        None,
        &Default::default(),
        &Subs::default(),
        None,
    )
    .expect("started");
    let end = Instant::now() + Duration::from_secs(60);
    let mut seen = Vec::new();
    let p = loop {
        let p = job.progress();
        if p.state != State::Running {
            break p;
        }
        if let Some(m) = p.merging {
            seen.push(m);
        }
        assert!(Instant::now() < end, "still running: {p:?}");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(matches!(p.state, State::Done(_)), "merge: {:?}", p.state);
    assert_eq!(p.merging, None, "done: no longer merging");
    let known: Vec<f64> = seen.iter().filter_map(|m| *m).collect();
    assert!(!seen.is_empty(), "merging was said");
    assert!(
        known.iter().any(|f| *f > 0.0 && *f < 1.0),
        "a share done was said: {seen:?}"
    );
    assert!(
        known.windows(2).all(|w| w[0] <= w[1]),
        "it only goes forward: {known:?}"
    );
}

/// #79 and #94: the job never gives up while yt-dlp works, never leaves it running when it does
/// give up, and mends what a new look-up or a nearby quality can mend.
fn job_checks(me: &Path) {
    let none = Subs::default();

    // Silent for 30 s (ffmpeg merging a long video): no timeout of MDM's own ends it.
    let (p, files, _o) = download_as(me, "quiet:30", &none, "b", "Talk");
    assert!(matches!(p.state, State::Done(_)), "quiet: {:?}", p.state);
    assert_eq!(files, ["Talk.mp4"]);

    // Lines that aren't UTF-8, the final path among them: read through, the file found by name.
    let (p, files, _o) = download_as(me, "odd-lines", &none, "b", "Şirin ğüş");
    match &p.state {
        State::Done(f) => assert_eq!(f.file_name().unwrap().to_string_lossy(), "Şirin ğüş.mp4"),
        other => panic!("odd lines: {other:?}"),
    }
    assert_eq!(files, ["Şirin ğüş.mp4"]);

    // It falls over mid-way: the failure is told, and what it left running is ended with it
    // (the job).
    let (p, _files, out) = download_as(me, "crash", &none, "b", "Talk");
    match &p.state {
        State::Failed(e) => assert!(e.contains("HTTP Error 500"), "{e}"),
        other => panic!("crash: {other:?}"),
    }
    let size = || std::fs::metadata(out.path().join("linger.log")).map_or(0, |m| m.len());
    std::thread::sleep(Duration::from_millis(800));
    let after = size();
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(
        size(),
        after,
        "the helper it left was ended with the download"
    );

    // The chosen height isn't offered (a reroute left it out): the nearest below, said so.
    let (p, files, _o) = download_as(
        me,
        "unavailable",
        &none,
        "bv*[height=1080]+ba/b[height=1080]",
        "Talk",
    );
    assert!(
        matches!(p.state, State::Done(_)),
        "unavailable: {:?}",
        p.state
    );
    assert_eq!(files, ["Talk.mp4"]);
    let log = log_of(&p);
    assert!(
        log.iter()
            .any(|t| t.starts_with("The chosen quality isn't offered")),
        "{log:?}"
    );
    assert!(
        log.iter().any(|t| t == "Downloaded 18 640x360 instead"),
        "{log:?}"
    );

    // A media server that stopped resolving: looked up again without it, then saved.
    let (p, files, _o) = download_as(me, "dns-once", &none, "b", "Talk");
    assert!(matches!(p.state, State::Done(_)), "dns: {:?}", p.state);
    assert_eq!(files, ["Talk.mp4"]);
    let log = log_of(&p);
    assert!(
        log.iter().any(|t| t
            == "The server rr1---sn-dead.googlevideo.com couldn't be reached; looking the page up again without it"),
        "{log:?}"
    );
}

/// #69: a list is told from a video, listed whole as it comes (thousands, in order), a channel a
/// page at a time, and a listing stops when asked, ending yt-dlp.
fn list_checks(me: &Path) {
    use mdm_tools::playlist::{Listing, PAGE};
    use mdm_tools::resolve::{Looked, look_up};
    let tools = tempfile::tempdir().unwrap();
    let exe = fake_in(me, tools.path());
    let none = Default::default();
    let wait = |l: &Listing| {
        let end = Instant::now() + Duration::from_secs(60);
        loop {
            let p = l.progress();
            if p.done {
                return p;
            }
            assert!(Instant::now() < end, "still listing: {p:?}");
            std::thread::sleep(Duration::from_millis(20));
        }
    };

    match look_up(&exe, "https://list.example/playlist/6000", &none).unwrap() {
        Looked::List(l) => assert_eq!((l.title.as_str(), l.total), ("A fake playlist", Some(6000))),
        Looked::Video(_) => panic!("a playlist taken for a video"),
    }
    // Upstream issue 852: yt-dlp's warnings reach the look-up. A video looked up without a
    // JavaScript runtime says so (yt-dlp's own words, `fixtures/ytdlp-js-runtime`); another doesn't.
    let runtime_said = |url: &str| match look_up(&exe, url, &none).unwrap() {
        Looked::Video(v) => v.needs_js_runtime,
        Looked::List(_) => panic!("a video taken for a list"),
    };
    assert!(!runtime_said("https://video.example/talk"));
    assert!(runtime_said("https://nojs.example/watch"));

    // #352: cookies that leave yt-dlp nothing to offer: the same page without them.
    let page = "https://cookies.example/none";
    let cookies = mdm_tools::proc::FromBrowser {
        cookies: Some("LOGIN_INFO=l; SAPISID=s".into()),
        ..Default::default()
    };
    assert!(matches!(
        look_up(&exe, page, &cookies).unwrap(),
        Looked::Video(_)
    ));
    let (args, _file) = cookies.args(page).unwrap();
    let out =
        mdm_tools::proc::look_up(&exe, &["-J".to_owned()], page, Duration::from_secs(30)).unwrap();
    assert!(!out.without_cookies, "no cookies, so nothing to retry");
    let first = ["-J".to_owned()]
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();
    let out = mdm_tools::proc::look_up(&exe, &first, page, Duration::from_secs(30)).unwrap();
    assert!(out.without_cookies, "the second look-up, without them");

    // Six thousand, whole: all of them, in order, the private ones marked.
    let t = Instant::now();
    let l = Listing::start(&exe, "https://list.example/playlist/6000", &none, 1, None).unwrap();
    let p = wait(&l);
    let all = l.entries_from(0);
    assert_eq!((all.len(), p.total, p.error), (6000, Some(6000), None));
    assert!(all.iter().enumerate().all(|(i, e)| e.index == i as u64 + 1));
    assert!(all[6].unavailable && !all[7].unavailable);
    assert_eq!(l.entries_from(5990).len(), 10);
    println!("fake_ytdlp: 6000 listed in {:?}", t.elapsed());

    // A channel (no size said): a page, then the next from where it stopped, then the end.
    let url = "https://list.example/channel/120";
    let l = Listing::start(&exe, url, &none, 1, Some(PAGE)).unwrap();
    let p = wait(&l);
    assert_eq!((l.entries_from(0).len(), p.total), (PAGE as usize, None));
    let l = Listing::start(&exe, url, &none, 101, Some(150)).unwrap();
    wait(&l);
    let last = l.entries_from(0);
    assert_eq!((last.len(), last[0].index), (20, 101));

    // The site stops answering part-way: what was listed stays, and why it stopped is told;
    // listed again from the next one, it goes on to the end.
    let url = "https://list.example/playlist/300?fail=121";
    let l = Listing::start(&exe, url, &none, 1, None).unwrap();
    let p = wait(&l);
    assert_eq!(l.entries_from(0).len(), 120);
    assert!(p.error.as_deref().unwrap_or("").contains("429"), "{p:?}");
    let l = Listing::start(&exe, url, &none, 121, None).unwrap();
    let p = wait(&l);
    let rest = l.entries_from(0);
    assert_eq!((rest.len(), rest[0].index, p.error), (180, 121, None));

    // Stopped part-way: what was listed stays, and it's done at once.
    let l = Listing::start(&exe, "https://list.example/playlist/20000", &none, 1, None).unwrap();
    while l.progress().listed < 150 {
        std::thread::sleep(Duration::from_millis(10));
    }
    l.cancel();
    let p = l.progress();
    let kept = l.entries_from(0).len();
    assert!(p.done && (150..20000).contains(&kept), "{p:?}");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        l.entries_from(0).len(),
        kept,
        "nothing listed after the stop"
    );
}

fn checks(me: &Path) {
    let subs = |langs: &[&str], embed: bool| Subs {
        langs: langs.iter().map(|l| l.to_string()).collect(),
        automatic: true,
        embed,
    };

    // English refused every time: the video and the Russian ones are saved, English is told.
    let (p, files) = download(me, "refuse-always", &subs(&["en", "ru"], false));
    assert!(matches!(p.state, State::Done(_)), "{:?}", p.state);
    assert_eq!(files, ["Talk.mp4", "Talk.ru.srt"]);
    assert_eq!(p.sub_problems.len(), 1, "{:?}", p.sub_problems);
    let problem = &p.sub_problems[0];
    assert_eq!(
        (
            problem.code.as_str(),
            problem.name.as_str(),
            &problem.reason,
            problem.beside
        ),
        ("en", "English", &SubReason::TooManyRequests, false)
    );
    let log: Vec<&str> = p.events.iter().map(|(_, t)| t.as_str()).collect();
    assert!(
        log.iter()
            .any(|t| t.starts_with("Subtitles in en couldn't be downloaded")),
        "{log:?}"
    );

    // Refused once only: the second try, after the pause, brings them.
    let (p, files) = download(me, "refuse-once", &subs(&["en"], false));
    assert!(matches!(p.state, State::Done(_)), "{:?}", p.state);
    assert_eq!(files, ["Talk.en.srt", "Talk.mp4"]);
    assert!(p.sub_problems.is_empty(), "{:?}", p.sub_problems);
    assert_eq!(
        p.side_files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ["Talk.en.srt"]
    );

    // Asked into the video (no ffmpeg here, so beside it): still never the video's failure.
    let (p, files) = download(me, "refuse-always", &subs(&["en"], true));
    assert!(matches!(p.state, State::Done(_)), "{:?}", p.state);
    assert_eq!(files, ["Talk.mp4"]);
    assert_eq!(p.sub_problems.len(), 1);
}

/// #769: MDM's engine downloads what yt-dlp chose, from a lab server: each stream over several
/// connections, within yt-dlp's `http_chunk_size`, into the files yt-dlp then merges; a stopped
/// one goes on from where it stopped; an address that expires is asked of yt-dlp again; a new
/// speed limit applies at once; and a stream in pieces stays yt-dlp's to download, said so.
fn engine_checks(me: &Path) {
    use mdm_labserver::{Config, LabServer, content};
    let rt = tokio::runtime::Runtime::new().unwrap();
    let lab = rt.block_on(LabServer::start(Config::default())).unwrap();
    let engine = |connections: u32| mdm_tools::streams::Engine {
        handle: rt.handle().clone(),
        opts: mdm_engine::Options {
            max_connections: connections,
            retry_delay: Duration::from_millis(50),
            ..Default::default()
        },
    };
    const V: u64 = 3 << 20;
    const A: u64 = 1 << 20;
    let page = |video: &str, audio: &str, protocol: &str| {
        format!(
            r#"{{"id": "talk", "title": "Talk", "ext": "mp4", "formats": [
            {{"format_id": "137", "ext": "mp4", "protocol": "{protocol}", "filesize": {V},
              "vcodec": "avc1", "acodec": "none", "url": "{}",
              "http_headers": {{"User-Agent": "FakeTube/1"}},
              "downloader_options": {{"http_chunk_size": 524288}}}},
            {{"format_id": "140", "ext": "m4a", "protocol": "https", "filesize": {A},
              "vcodec": "none", "acodec": "mp4a", "url": "{}"}}]}}"#,
            lab.url(video),
            lab.url(audio)
        )
    };
    let expected = || {
        let mut v = vec![0; V as usize];
        content::fill(1, 0, &mut v);
        let mut a = vec![0; A as usize];
        content::fill(2, 0, &mut a);
        v.extend(a);
        v
    };
    let start = |tools: &Path, out: &Path, connections: u32| {
        let exe = fake_in(me, tools);
        VideoJob::start(
            &exe,
            "https://video.example/talk",
            "bv*+ba/b",
            out,
            "Talk",
            None,
            1,
            None,
            None,
            &Default::default(),
            &Subs::default(),
            Some(engine(connections)),
        )
        .expect("started")
    };
    // What each run said of its merge (#736), in order.
    let merges = std::cell::RefCell::new(Vec::new());
    let wait = |job: &VideoJob| {
        let end = Instant::now() + Duration::from_secs(120);
        let mut most_connections = 0;
        merges.borrow_mut().clear();
        loop {
            let p = job.progress();
            if let Some(e) = &p.engine {
                most_connections = most_connections.max(e.connections.len());
            }
            if let Some(m) = p.merging {
                merges.borrow_mut().push(m);
            }
            if p.state != State::Running {
                return (p, most_connections);
            }
            assert!(Instant::now() < end, "still running: {:?}", log_of(&p));
            std::thread::sleep(Duration::from_millis(50));
        }
    };

    // Merged: two streams, each over several connections, in requests of 512 KB at most.
    let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(
        tools.path().join("page.json"),
        page(
            "/gen/3M/v.mp4?seed=1&rate=8M",
            "/gen/1M/a.m4a?seed=2",
            "https",
        ),
    )
    .unwrap();
    let job = start(tools.path(), out.path(), 4);
    let (p, most) = wait(&job);
    let log = log_of(&p);
    assert!(matches!(p.state, State::Done(_)), "{:?}: {log:?}", p.state);
    assert!(p.helper.is_none());
    let saved = out.path().join("Talk.mp4");
    assert!(
        std::fs::read(&saved).unwrap() == expected(),
        "the streams merged"
    );
    assert!(most >= 2, "several connections: {most}");
    // #736: the engine downloaded the streams, yt-dlp's ffmpeg merges them: said, with how far.
    let known: Vec<f64> = merges.borrow().iter().filter_map(|m| *m).collect();
    assert!(
        known.iter().any(|f| *f > 0.0 && *f < 1.0),
        "the merge and its share done were said: {:?}",
        merges.borrow()
    );
    assert!(known.windows(2).all(|w| w[0] <= w[1]), "{known:?}");
    assert_eq!(p.merging, None, "done: no longer merging");
    assert!(
        log.iter()
            .any(|t| t == "yt-dlp's address for format 137: from 127.0.0.1"),
        "{log:?}"
    );
    rt.block_on(lab.settled());
    let video: Vec<_> = lab
        .log()
        .into_iter()
        .filter(|e| e.path.starts_with("/gen/3M/v.mp4"))
        .collect();
    assert!(video.iter().all(|e| {
        e.user_agent.as_deref() == Some("FakeTube/1")
            && e.range.as_deref().is_some_and(|r| {
                let (a, b) = r.trim_start_matches("bytes=").split_once('-').unwrap();
                b.parse::<u64>().unwrap() - a.parse::<u64>().unwrap() < 524288
            })
    }));
    assert!(
        std::fs::read_dir(out.path())
            .unwrap()
            .filter_map(Result::ok)
            .all(|e| !e.file_name().to_string_lossy().contains(".mdm.")),
        "no part file or state left"
    );

    // Stopped half-way, then started again: it goes on from the bytes already there.
    lab.clear_log();
    let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(
        tools.path().join("page.json"),
        page(
            "/gen/3M/v.mp4?seed=1&rate=1M",
            "/gen/1M/a.m4a?seed=2",
            "https",
        ),
    )
    .unwrap();
    let job = start(tools.path(), out.path(), 2);
    while job.progress().downloaded < 1 << 20 {
        assert!(job.progress().state == State::Running);
        std::thread::sleep(Duration::from_millis(20));
    }
    // A new limit applies at once: 256 KB/s, where the server gives 2 MB/s over two.
    job.set_speed_limit(Some(256 * 1024));
    std::thread::sleep(Duration::from_millis(500));
    let (a, t) = (job.progress().downloaded, Instant::now());
    std::thread::sleep(Duration::from_secs(2));
    let rate = (job.progress().downloaded - a) as f64 / t.elapsed().as_secs_f64();
    assert!(rate < 512.0 * 1024.0, "the limit holds: {rate:.0} B/s");
    job.stop();
    let (p, _) = wait(&job);
    assert_eq!(p.state, State::Stopped);
    let state = mdm_engine::state::state_path(&out.path().join("Talk.f137.mp4"));
    assert!(state.exists(), "the resume state is kept");
    let job = start(tools.path(), out.path(), 2);
    job.set_speed_limit(None);
    let (p, _) = wait(&job);
    let log = log_of(&p);
    assert!(matches!(p.state, State::Done(_)), "{:?}: {log:?}", p.state);
    assert!(
        std::fs::read(out.path().join("Talk.mp4")).unwrap() == expected(),
        "resumed whole"
    );
    assert!(log.iter().any(|t| t.starts_with("Resuming: ")), "{log:?}");

    // Addresses that expire after 1.5 s, on connections dropped every 512 KB at 512 KB/s:
    // yt-dlp is asked for today's, and the stream goes on.
    let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(
        tools.path().join("page.json"),
        page(
            "/gen/3M/v.mp4?seed=1&rate=512K&drop_after=512K&expires={EXPIRES}",
            "/gen/1M/a.m4a?seed=2",
            "https",
        ),
    )
    .unwrap();
    let job = start(tools.path(), out.path(), 2);
    let (p, _) = wait(&job);
    let log = log_of(&p);
    assert!(matches!(p.state, State::Done(_)), "{:?}: {log:?}", p.state);
    assert!(
        std::fs::read(out.path().join("Talk.mp4")).unwrap() == expected(),
        "whole across new addresses"
    );
    assert!(
        log.iter()
            .any(|t| t == "Asking yt-dlp for a new address for format 137"),
        "{log:?}"
    );

    // A protocol the engine doesn't speak: yt-dlp downloads the choice itself, and says why.
    let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(
        tools.path().join("page.json"),
        page("/gen/3M/v.mp4?seed=1", "/gen/1M/a.m4a?seed=2", "rtmp"),
    )
    .unwrap();
    let job = start(tools.path(), out.path(), 2);
    let (p, _) = wait(&job);
    let log = log_of(&p);
    assert!(matches!(p.state, State::Done(_)), "{:?}: {log:?}", p.state);
    assert_eq!(
        p.helper,
        Some(mdm_tools::streams::Helper::Protocol("rtmp".into()))
    );
    assert!(
        log.iter()
            .any(|t| t == "yt-dlp downloads this video itself: its protocol (rtmp)"),
        "{log:?}"
    );
    pieces_checks(me, &rt, &lab, &engine, &expected_audio());
    println!("fake_ytdlp: the engine's checks ok");
}

/// The sound every page of the engine's checks has (format 140, the lab's `/gen/1M/a.m4a`).
fn expected_audio() -> Vec<u8> {
    let mut a = vec![0; 1 << 20];
    mdm_labserver::content::fill(2, 0, &mut a);
    a
}

/// Upstream issue 769: streams in pieces through the engine: an HLS playlist (plain, and
/// AES-128 with the playlist's key), DASH's pieces as yt-dlp lists them, and pieces whose
/// addresses expire, asked of yt-dlp again. Each comes whole, in order, joined as yt-dlp's own
/// downloaders join them, then merged with the sound.
fn pieces_checks(
    me: &Path,
    rt: &tokio::runtime::Runtime,
    lab: &mdm_labserver::LabServer,
    engine: &dyn Fn(u32) -> mdm_tools::streams::Engine,
    audio: &[u8],
) {
    let get = |path: &str| -> Vec<u8> {
        let url = lab.url(path);
        rt.block_on(async {
            reqwest::get(url)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .to_vec()
        })
    };
    let audio_format = format!(
        r#"{{"format_id": "140", "ext": "m4a", "protocol": "https", "filesize": {},
          "vcodec": "none", "acodec": "mp4a", "url": "{}"}}"#,
        1 << 20,
        lab.url("/gen/1M/a.m4a?seed=2")
    );
    let run = |video: String, connections: u32| {
        let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        std::fs::write(
            tools.path().join("page.json"),
            format!(
                r#"{{"id": "talk", "title": "Talk", "ext": "mp4", "formats": [{video}, {audio_format}]}}"#
            ),
        )
        .unwrap();
        let exe = fake_in(me, tools.path());
        let job = VideoJob::start(
            &exe,
            "https://video.example/talk",
            "bv*+ba/b",
            out.path(),
            "Talk",
            None,
            1,
            None,
            None,
            &Default::default(),
            &Subs::default(),
            Some(engine(connections)),
        )
        .expect("started");
        let end = Instant::now() + Duration::from_secs(120);
        let mut most = 0;
        let p = loop {
            let p = job.progress();
            if let Some(e) = &p.engine {
                most = most.max(e.connections.len());
            }
            if p.state != State::Running {
                break p;
            }
            assert!(Instant::now() < end, "still running: {:?}", log_of(&p));
            std::thread::sleep(Duration::from_millis(50));
        };
        let log = log_of(&p);
        assert!(matches!(p.state, State::Done(_)), "{:?}: {log:?}", p.state);
        assert!(p.helper.is_none(), "{:?}", p.helper);
        let State::Done(file) = &p.state else {
            unreachable!()
        };
        let saved = std::fs::read(file).unwrap();
        let left: Vec<String> = std::fs::read_dir(out.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".mdm"))
            .collect();
        assert!(left.is_empty(), "left behind: {left:?}");
        (saved, log, most)
    };
    let hls = |playlist: &str, extra: &str| {
        format!(
            r#"{{"format_id": "hls-720p", "ext": "mp4", "protocol": "m3u8_native",
              "vcodec": "avc1", "acodec": "none", "url": "{}"{extra}}}"#,
            lab.url(playlist)
        )
    };
    let plain: Vec<u8> = (0..3)
        .flat_map(|i| get(&format!("/hls/720p/seg{i}.ts")))
        .chain(audio.iter().copied())
        .collect();

    // HLS: three pieces, two at once, joined in order.
    let (saved, log, most) = run(hls("/hls/720p/index.m3u8", ""), 2);
    assert!(saved == plain, "HLS pieces joined in order");
    assert!(most >= 2, "pieces at once: {most}");
    assert!(
        log.iter()
            .any(|t| t.starts_with("Format hls-720p: 3 pieces, 2 at once")),
        "{log:?}"
    );

    // AES-128: the playlist's key, the first piece's own IV and the others' sequence numbers.
    let (saved, log, _) = run(hls("/hls/720p/enc.m3u8", ""), 3);
    assert!(saved == plain, "decrypted as the plain pieces: {log:?}");

    // DASH: yt-dlp's list (an init section and three pieces, from its base address).
    let dash = format!(
        r#"{{"format_id": "dash-720p", "ext": "mp4", "protocol": "http_dash_segments",
          "vcodec": "avc1", "acodec": "none", "url": "{}",
          "fragment_base_url": "{}",
          "fragments": [{{"path": "init.mp4"}}, {{"path": "seg1.m4s"}}, {{"path": "seg2.m4s"}},
                        {{"url": "{}"}}]}}"#,
        lab.url("/dash/manifest.mpd"),
        lab.url("/dash/720p/"),
        lab.url("/dash/720p/seg3.m4s")
    );
    let want: Vec<u8> = ["init.mp4", "seg1.m4s", "seg2.m4s", "seg3.m4s"]
        .iter()
        .flat_map(|f| get(&format!("/dash/720p/{f}")))
        .chain(audio.iter().copied())
        .collect();
    let (saved, _, _) = run(dash, 4);
    assert!(saved == want, "DASH pieces joined in order");

    // Pieces whose addresses expire 1.5 s after each look-up, one at a time at 256 KB/s: yt-dlp
    // is asked for the list again, and the pieces go on from the new one.
    let pieces: Vec<String> = (1..=4)
        .map(|i| {
            format!(
                r#"{{"url": "{}"}}"#,
                lab.url(&format!(
                    "/gen/256K/p{i}?seed={}&rate=256K&expires={{EXPIRES}}",
                    20 + i
                ))
            )
        })
        .collect();
    let expiring = format!(
        r#"{{"format_id": "dash-exp", "ext": "mp4", "protocol": "http_dash_segments",
          "vcodec": "avc1", "acodec": "none", "url": "{}", "fragments": [{}]}}"#,
        lab.url("/dash/manifest.mpd"),
        pieces.join(", ")
    );
    let want: Vec<u8> = (1..=4u64)
        .flat_map(|i| {
            let mut b = vec![0; 256 * 1024];
            mdm_labserver::content::fill(20 + i, 0, &mut b);
            b
        })
        .chain(audio.iter().copied())
        .collect();
    let (saved, log, _) = run(expiring, 1);
    let first = saved.iter().zip(&want).position(|(a, b)| a != b);
    assert!(
        saved == want,
        "whole across new lists: {} vs {} bytes, first difference at {first:?}: {log:?}",
        saved.len(),
        want.len()
    );
    assert!(
        log.iter()
            .any(|t| t == "Format dash-exp: Asking yt-dlp for the pieces' new addresses"),
        "{log:?}"
    );
}

/// A live stream's recording, stopped and saved (`VideoJob::save`): yt-dlp (the fake, `live`
/// modes) records `Talk.mp4.part` as MPEG-TS through MDM's engine's look-up path (the page says
/// `is_live`), MDM's ffmpeg (the fake, as `fake-ffmpeg`) makes it an MP4. What it records shows
/// as it grows; saved, it is a video and nothing else is left; whatever fails, nothing recorded
/// is lost, and yt-dlp never starts again once saving.
fn recording_checks(me: &Path) {
    use mdm_tools::video::LiveState;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = || mdm_tools::streams::Engine {
        handle: rt.handle().clone(),
        opts: mdm_engine::Options::default(),
    };
    const LIVE: &str = r#"{"id": "live", "title": "Live", "is_live": true, "ext": "mp4",
        "formats": [{"format_id": "96", "ext": "mp4", "protocol": "m3u8"}]}"#;
    // A folder for the fakes and one for the download, the fake recording as `mode`, MDM's
    // ffmpeg as `ffmpeg_mode`, its ffprobe beside it.
    let folders = |mode: &str, ffmpeg_mode: &str| {
        let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        std::fs::write(tools.path().join("page.json"), LIVE).unwrap();
        std::fs::write(out.path().join("mode.txt"), mode).unwrap();
        std::fs::write(out.path().join("ffmpeg-mode.txt"), ffmpeg_mode).unwrap();
        let exe = fake_in(me, tools.path());
        let ffmpeg = tools
            .path()
            .join(format!("fake-ffmpeg{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(me, &ffmpeg).unwrap();
        let ffprobe = tools
            .path()
            .join(format!("fake-ffprobe{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(me, &ffprobe).unwrap();
        (tools, out, exe, ffmpeg)
    };
    let start = |exe: &Path, out: &Path, ffmpeg: Option<&Path>, by_engine: bool| {
        VideoJob::start(
            exe,
            "https://video.example/live",
            "b",
            out,
            "Talk",
            None,
            1,
            ffmpeg,
            None,
            &Default::default(),
            &Subs::default(),
            by_engine.then(engine),
        )
        .expect("started")
    };
    let until = |job: &VideoJob, what: &str, ok: &dyn Fn(&VideoJob) -> bool| {
        let end = Instant::now() + Duration::from_secs(60);
        while !ok(job) {
            let p = job.progress();
            assert!(
                Instant::now() < end && p.state == State::Running,
                "{what}: {:?} {:?}",
                p.state,
                log_of(&p)
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    // Recording with something in: what the tests that save and expect an MP4 wait for (a
    // recording is shown from the moment the recorder opens its file, its first block later).
    let recording = |job: &VideoJob| {
        job.live_state() == Some(LiveState::Recording) && job.progress().recorded > 0
    };
    // Recording, and enough of it that what a check tells apart doesn't hang on how much was in
    // when it saved (the watcher reads the size once a second while the fake writes a block of
    // 65,424 bytes every tenth: anything from one block to ten). The fake ffprobe's `short` modes
    // say the MP4 (or its sound) plays half as long as the recording, and MDM takes an MP4 up to
    // a second shorter as whole (`SLACK` in video.rs): up to 2 s recorded (three blocks), the
    // missing half is within that second and the MP4 counts as whole. 3 s of the fake's playback
    // (five blocks) is past it, as past the 64 KiB `tiny` checks (two blocks) and the two rounds
    // `live-split` checks. What is recorded only grows until yt-dlp is stopped.
    let enough =
        |job: &VideoJob| recording(job) && job.progress().recorded as f64 >= 3.0 * BYTES_PER_SECOND;
    // Recording, whatever has been written.
    let opened = |job: &VideoJob| job.live_state() == Some(LiveState::Recording);
    // For a while (twice the watcher's second) nothing says it records: it isn't, while the
    // fake holds its file back. Not a wait for anything, the fake's files are what synchronize.
    let not_recording = |job: &VideoJob, what: &str| {
        let end = Instant::now() + Duration::from_millis(2500);
        while Instant::now() < end {
            let p = job.progress();
            assert_eq!(p.state, State::Running, "{what}: {:?}", log_of(&p));
            assert!(!p.recording, "{what}");
            assert_eq!(job.live_state(), None, "{what}");
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    let ended = |job: &VideoJob| {
        let end = Instant::now() + Duration::from_secs(120);
        loop {
            let p = job.progress();
            if p.state != State::Running {
                return p;
            }
            assert!(Instant::now() < end, "still running: {:?}", log_of(&p));
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    let count = |file: PathBuf| {
        std::fs::read_to_string(file)
            .ok()
            .and_then(|t| t.trim().parse::<u32>().ok())
            .unwrap_or(0)
    };
    let names = |out: &Path| {
        let mut n: Vec<String> = std::fs::read_dir(out)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| !n.ends_with(".txt") && !n.ends_with(".log"))
            .collect();
        n.sort();
        n
    };
    let ts = |len: usize| {
        let mut packet = [0u8; 188];
        packet[0] = 0x47;
        packet.repeat(len.div_ceil(188))
    };

    // Saved: an MP4, nothing else left. What it records shows, growing, and is gone once over;
    // saved twice and stopped while saving, it is saved once, and yt-dlp ran once.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    let first = job.progress().recorded;
    until(&job, "grows", &|j: &VideoJob| j.progress().recorded > first);
    assert_eq!(job.helper(), Some(mdm_tools::streams::Helper::Live));
    job.save();
    assert_eq!(job.live_state(), Some(LiveState::Saving));
    assert_eq!(job.progress().state, State::Running, "saving: not over yet");
    job.save();
    job.stop();
    assert_eq!(
        job.live_state(),
        Some(LiveState::Saving),
        "a Stop changes nothing"
    );
    let p = ended(&job);
    let log = log_of(&p);
    assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
    assert_eq!(names(out.path()), ["Talk.mp4"], "no .part, no temp");
    let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
    assert!(
        mp4.starts_with(b"FAKEMP4 ") && mp4[8] == 0x47,
        "made by ffmpeg"
    );
    assert!(mp4.len() as u64 > first, "all of it");
    assert_eq!(p.downloaded, mp4.len() as u64);
    assert_eq!(p.recorded, 0, "no longer recording");
    assert_eq!(job.live_state(), None);
    assert_eq!(count(out.path().join("tries.txt")), 1, "yt-dlp ran once");
    assert_eq!(count(out.path().join("ffmpeg-runs.txt")), 1, "saved once");
    job.stop();
    assert_eq!(job.progress().state, p.state, "done stays done");
    println!(
        "recording_checks: saved {} bytes as Talk.mp4 ({first} recorded at first)",
        mp4.len()
    );

    // `Talk.mp4` there already: the recording beside it, out of MDM's numbering.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    std::fs::write(out.path().join("Talk.mp4"), b"another").unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.save();
    let p = ended(&job);
    assert_eq!(
        p.state,
        State::Done(out.path().join("Talk (recording 1).mp4")),
        "{:?}",
        log_of(&p)
    );
    assert_eq!(
        std::fs::read(out.path().join("Talk.mp4")).unwrap(),
        b"another"
    );
    assert_eq!(names(out.path()), ["Talk (recording 1).mp4", "Talk.mp4"]);

    // ffmpeg fails: kept as the MPEG-TS it is, its half-made MP4 gone.
    let (_tools, out, exe, ffmpeg) = folders("live", "fail");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.save();
    let p = ended(&job);
    let log = log_of(&p);
    assert_eq!(p.state, State::Done(out.path().join("Talk.ts")), "{log:?}");
    assert_eq!(names(out.path()), ["Talk.ts"]);
    assert!(
        log.iter()
            .any(|t| t.starts_with("The recording couldn't be made an MP4 (")),
        "{log:?}"
    );

    // ffmpeg's file far smaller than the recording: not taken, the recording kept whole.
    let (_tools, out, exe, ffmpeg) = folders("live", "tiny");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "enough recorded", &enough);
    job.save();
    let p = ended(&job);
    assert_eq!(p.state, State::Done(out.path().join("Talk.ts")));
    assert_eq!(names(out.path()), ["Talk.ts"]);
    let kept = std::fs::read(out.path().join("Talk.ts")).unwrap();
    assert!(kept.len() >= 64 << 10 && kept[0] == 0x47, "{}", kept.len());

    // Not MPEG-TS, and ffmpeg failed: never a file that isn't one; the recording stays as it
    // is, and the error says where.
    let (_tools, out, exe, ffmpeg) = folders("live-raw", "fail");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.save();
    let p = ended(&job);
    let State::Failed(e) = &p.state else {
        panic!("{:?}", p.state)
    };
    assert!(
        e.contains("Talk.mp4.part") && e.contains("isn't MPEG-TS"),
        "{e}"
    );
    assert_eq!(names(out.path()), ["Talk.mp4.part"]);
    assert!(
        std::fs::metadata(out.path().join("Talk.mp4.part"))
            .unwrap()
            .len()
            > 0
    );

    // Saved before anything is recorded: a Stop.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    job.save();
    assert_eq!(ended(&job).state, State::Stopped);
    assert_eq!(count(out.path().join("ffmpeg-runs.txt")), 0);

    // The recorder connects first and opens its file later (`live-late`, the test says when with
    // `open.txt`, then `go.txt` for its first block): nothing is recording while there is no
    // file. A 0-byte `.part` an earlier run left (a stop in its first seconds) is removed before
    // yt-dlp starts, so the new recording shows once its file is made, though empty (and not
    // later, once it has grown).
    let (_tools, out, exe, ffmpeg) = folders("live-late", "");
    let part = out.path().join("Talk.mp4.part");
    std::fs::write(&part, b"").unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "started", &|_: &VideoJob| {
        count(out.path().join("tries.txt")) > 0
    });
    not_recording(&job, "an earlier empty .part, before open.txt");
    assert!(!part.exists(), "the earlier empty file is removed");
    assert!(
        log_of(&job.progress()).contains(
            &"Removed the empty unfinished file Talk.mp4.part left by an earlier run".to_owned()
        ),
        "{:?}",
        log_of(&job.progress())
    );
    std::fs::write(out.path().join("open.txt"), "").unwrap();
    until(&job, "opened", &opened);
    let p = job.progress();
    assert_eq!((p.recorded, p.recording), (0, true));
    assert_eq!(std::fs::metadata(&part).unwrap().len(), 0);
    std::fs::write(out.path().join("go.txt"), "").unwrap();
    until(&job, "recording", &recording);
    job.stop();
    assert_eq!(ended(&job).state, State::Stopped);

    // No file yet, then an empty one: recording at 0 bytes. Stop and save keeps nothing, leaves
    // the download stopped (to resume), the empty file as it is, no MP4, and ffmpeg unused.
    let (_tools, out, exe, ffmpeg) = folders("live-late", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "started", &|_: &VideoJob| {
        count(out.path().join("tries.txt")) > 0
    });
    not_recording(&job, "no file yet");
    assert!(!out.path().join("Talk.mp4.part").exists());
    std::fs::write(out.path().join("open.txt"), "").unwrap();
    until(&job, "opened", &opened);
    let p = job.progress();
    assert_eq!((p.recorded, p.recording), (0, true));
    assert_eq!(
        std::fs::metadata(out.path().join("Talk.mp4.part"))
            .unwrap()
            .len(),
        0
    );
    job.save();
    assert_eq!(
        job.live_state(),
        Some(LiveState::Saving),
        "saved, not just stopped"
    );
    let p = ended(&job);
    assert_eq!(p.state, State::Stopped, "{:?}", log_of(&p));
    assert_eq!(
        names(out.path()),
        ["Talk.mp4.part"],
        "no MP4, nothing deleted"
    );
    assert_eq!(
        std::fs::metadata(out.path().join("Talk.mp4.part"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        count(out.path().join("ffmpeg-runs.txt")),
        0,
        "ffmpeg not run"
    );
    assert_eq!(count(out.path().join("tries.txt")), 1, "yt-dlp ran once");
    assert_eq!((p.recorded, p.recording), (0, false));
    assert_eq!(job.live_state(), None);

    // Another one: the file opened, then its first block: what is recorded shows, and Stop and
    // save makes the MP4 as for any recording.
    let (_tools, out, exe, ffmpeg) = folders("live-late", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    std::fs::write(out.path().join("open.txt"), "").unwrap();
    until(&job, "opened", &opened);
    std::fs::write(out.path().join("go.txt"), "").unwrap();
    until(&job, "recorded", &recording);
    assert!(job.progress().recorded > 0);
    job.save();
    let p = ended(&job);
    assert_eq!(
        p.state,
        State::Done(out.path().join("Talk.mp4")),
        "{:?}",
        log_of(&p)
    );
    assert_eq!(names(out.path()), ["Talk.mp4"]);
    assert_eq!(count(out.path().join("ffmpeg-runs.txt")), 1);
    assert_eq!((p.recorded, p.recording), (0, false));

    // Without MDM's engine nothing says it's live: Save is a Stop, the recording stays.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), false);
    until(&job, "recorded", &|_: &VideoJob| {
        std::fs::metadata(out.path().join("Talk.mp4.part")).is_ok_and(|m| m.len() > 0)
    });
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(job.helper(), None);
    assert_eq!(job.live_state(), None);
    assert_eq!(job.progress().recorded, 0);
    job.save();
    assert_eq!(ended(&job).state, State::Stopped);
    assert_eq!(names(out.path()), ["Talk.mp4.part"]);
    assert_eq!(count(out.path().join("ffmpeg-runs.txt")), 0);

    // Saved while the page is looked up again between two runs (the first failed): a Stop,
    // and yt-dlp doesn't start again.
    let (tools, out, exe, ffmpeg) = folders("live-dns-once", "");
    std::fs::write(tools.path().join("slow-second-look-up"), "").unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "looking up again", &|j: &VideoJob| {
        log_of(&j.progress())
            .iter()
            .any(|t| t.ends_with("couldn't be reached; looking the page up again without it"))
    });
    assert_eq!(job.live_state(), None, "between runs: not recording");
    job.save();
    let p = ended(&job);
    assert_eq!(p.state, State::Stopped, "{:?}", log_of(&p));
    std::thread::sleep(Duration::from_secs(4));
    assert_eq!(
        count(out.path().join("tries.txt")),
        1,
        "yt-dlp didn't start again"
    );
    assert_eq!(names(out.path()), ["Talk.mp4.part"]);

    // An earlier recording left there (Cancel, then Resume): kept aside before yt-dlp records
    // over it, each time under a name of its own, never written over.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    let earlier = ts(100_000);
    std::fs::write(out.path().join("Talk.mp4.part"), &earlier).unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    let log = log_of(&job.progress());
    assert!(
        log.iter()
            .any(|t| t == "The earlier recording was kept as Talk (recording 1).ts"),
        "{log:?}"
    );
    job.stop();
    assert_eq!(ended(&job).state, State::Stopped);
    let second = std::fs::read(out.path().join("Talk.mp4.part")).unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.stop();
    assert_eq!(ended(&job).state, State::Stopped);
    let read = |n: &str| std::fs::read(out.path().join(n)).unwrap();
    assert_eq!(read("Talk (recording 1).ts"), earlier);
    assert_eq!(read("Talk (recording 2).ts"), second);
    assert_eq!(count(out.path().join("tries.txt")), 2);
    // Another download of the same name, numbered by MDM (`Talk (2)`), in the same folder:
    // none of these are its files, nor this download's own.
    std::fs::write(out.path().join("Talk (2).mp4"), b"another").unwrap();
    let theirs = mdm_tools::video::own_files(out.path(), "Talk (2)", true);
    assert_eq!(theirs, [out.path().join("Talk (2).mp4")]);
    let ours = mdm_tools::video::own_files(out.path(), "Talk", true);
    assert!(
        ours.iter()
            .all(|f| !f.to_string_lossy().contains("(recording")),
        "{ours:?}"
    );

    // The MP4 plays shorter than the recording (ffprobe says so): saved all the same, and the
    // recording kept beside it as MPEG-TS, byte for byte, the log saying why.
    let (_tools, out, exe, ffmpeg) = folders("live", "");
    std::fs::write(out.path().join("ffprobe-mode.txt"), "short").unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "enough recorded", &enough);
    job.save();
    let p = ended(&job);
    let log = log_of(&p);
    assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
    // Out of the download's own names: Replace would take `Talk.ts` with it.
    let kept = out.path().join("Talk (recording 1).ts");
    assert_eq!(names(out.path()), ["Talk (recording 1).ts", "Talk.mp4"]);
    let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
    assert_eq!(std::fs::read(&kept).unwrap(), mp4[8..]);
    assert!(
        log.iter().any(|t| t
            == "The MP4 doesn't hold all of the recording (it plays shorter, or one of its streams starts later or ends sooner), so the recording is kept beside it too: Talk (recording 1).ts"),
        "{log:?}"
    );
    assert!(!mdm_tools::video::own_files(out.path(), "Talk", true).contains(&kept));

    // The MP4 plays to the end but its sound doesn't: not all of it.
    // No ffprobe (or one that can't tell, or only estimated): whole when the MP4 is nine tenths
    // of the recording's size at least; four fifths isn't, and the recording stays beside it.
    for (ffprobe, ffmpeg_mode, whole) in [
        ("short-sound", "", false),
        ("none", "", true),
        ("none", "short", false),
        ("fail", "", true),
        ("fail", "short", false),
        ("estimate", "", true),
        ("estimate", "short", false),
    ] {
        let (tools, out, exe, ffmpeg) = folders("live", ffmpeg_mode);
        match ffprobe {
            "none" => std::fs::remove_file(
                tools
                    .path()
                    .join(format!("fake-ffprobe{}", std::env::consts::EXE_SUFFIX)),
            )
            .unwrap(),
            mode => std::fs::write(out.path().join("ffprobe-mode.txt"), mode).unwrap(),
        }
        let job = start(&exe, out.path(), Some(&ffmpeg), true);
        until(&job, "enough recorded", &enough);
        job.save();
        let p = ended(&job);
        let log = log_of(&p);
        assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
        let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
        if whole {
            assert_eq!(names(out.path()), ["Talk.mp4"], "{ffprobe} {ffmpeg_mode}");
        } else {
            assert_eq!(
                names(out.path()),
                ["Talk (recording 1).ts", "Talk.mp4"],
                "{ffprobe}"
            );
            let kept = std::fs::read(out.path().join("Talk (recording 1).ts")).unwrap();
            let shown = if ffmpeg_mode == "short" {
                kept.len() * 4 / 5
            } else {
                kept.len()
            };
            assert_eq!(mp4[8..], kept[..shown], "{ffprobe}");
            assert!(
                log.iter()
                    .any(|t| t.starts_with("The MP4 doesn't hold all of the recording (it plays shorter, or one of its streams starts later or ends sooner)")),
                "{log:?}"
            );
        }
    }

    // The picture and the sound recorded apart (each a stream's own part): one MP4 of both,
    // the parts gone, the log naming them.
    let (_tools, out, exe, ffmpeg) = folders("live-split", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "enough recorded", &enough);
    let (video, audio) = (
        out.path().join("Talk.f1.mp4.part"),
        out.path().join("Talk.f2.m4a.part"),
    );
    let mut own = mdm_tools::video::own_files(out.path(), "Talk", false);
    own.sort();
    assert_eq!(own, [video.clone(), audio.clone()], "yt-dlp's own files");
    job.save();
    let p = ended(&job);
    let log = log_of(&p);
    assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
    assert_eq!(names(out.path()), ["Talk.mp4"]);
    let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
    // The picture's then the sound's (a third of it), each in whole writes of the fake's.
    let (v, a) = (348 * 188, 116 * 188);
    assert!(
        [0, v].contains(&((mp4.len() - 8) % (v + a))) && mp4.len() > 8 + v + a,
        "both in: {}",
        mp4.len()
    );
    assert!(
        log.iter()
            .any(|t| t == "The recording came in 2 parts: Talk.f1.mp4.part, Talk.f2.m4a.part"),
        "{log:?}"
    );

    // Recorded apart, and ffmpeg fails: nothing renamed, both kept as they are, named.
    let (_tools, out, exe, ffmpeg) = folders("live-split", "fail");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.save();
    let p = ended(&job);
    let State::Failed(e) = &p.state else {
        panic!("{:?}", p.state)
    };
    assert!(
        e.starts_with("The recording's 2 parts couldn't be joined into an MP4 (")
            && e.contains("Talk.f1.mp4.part")
            && e.contains("Talk.f2.m4a.part"),
        "{e}"
    );
    assert_eq!(names(out.path()), ["Talk.f1.mp4.part", "Talk.f2.m4a.part"]);
    let kept: Vec<Vec<u8>> = ["Talk.f1.mp4.part", "Talk.f2.m4a.part"]
        .iter()
        .map(|part| std::fs::read(out.path().join(part)).unwrap())
        .collect();
    assert!(kept.iter().all(|b| !b.is_empty() && b[0] == 0x47));
    // Resumed once the stream had ended (a video now, not live, yt-dlp downloading it as any
    // other): both parts kept aside before anything runs, byte for byte.
    std::fs::write(out.path().join("mode.txt"), "").unwrap();
    let tools = tempfile::tempdir().unwrap();
    let job = VideoJob::start(
        &fake_in(me, tools.path()),
        "https://video.example/talk",
        "b",
        out.path(),
        "Talk",
        None,
        1,
        Some(&ffmpeg),
        None,
        &Default::default(),
        &Subs::default(),
        None,
    )
    .expect("started");
    let p = ended(&job);
    assert_eq!(
        p.state,
        State::Done(out.path().join("Talk.mp4")),
        "{:?}",
        log_of(&p)
    );
    assert_eq!(
        names(out.path()),
        ["Talk (recording 1).ts", "Talk (recording 2).ts", "Talk.mp4"]
    );
    assert_eq!(
        std::fs::read(out.path().join("Talk (recording 1).ts")).unwrap(),
        kept[0]
    );
    assert_eq!(
        std::fs::read(out.path().join("Talk (recording 2).ts")).unwrap(),
        kept[1]
    );

    // Recorded apart before (Cancel, then Resume): each part kept aside on its own.
    let (_tools, out, exe, ffmpeg) = folders("live-split", "");
    let (earlier_video, earlier_audio) = (ts(90_000), ts(30_000));
    std::fs::write(out.path().join("Talk.f1.mp4.part"), &earlier_video).unwrap();
    std::fs::write(out.path().join("Talk.f2.m4a.part"), &earlier_audio).unwrap();
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    until(&job, "recording", &recording);
    job.stop();
    assert_eq!(ended(&job).state, State::Stopped);
    let read = |n: &str| std::fs::read(out.path().join(n)).unwrap();
    assert_eq!(read("Talk (recording 1).ts"), earlier_video);
    assert_eq!(read("Talk (recording 2).ts"), earlier_audio);

    // Resumed once the stream had ended (a video now, downloaded as any other): its earlier
    // recording, a `.part` with no resume state of yt-dlp's, is kept aside, never downloaded on
    // to; with that state beside it, it is a download to go on with, left alone.
    for resumable in [false, true] {
        let (tools, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        std::fs::write(out.path().join("mode.txt"), "").unwrap();
        let exe = fake_in(me, tools.path());
        let earlier = ts(50_000);
        let part = out.path().join("Talk.mp4.part");
        std::fs::write(&part, &earlier).unwrap();
        if resumable {
            std::fs::write(out.path().join("Talk.mp4.ytdl"), b"{}").unwrap();
        }
        let job = VideoJob::start(
            &exe,
            "https://video.example/talk",
            "b",
            out.path(),
            "Talk",
            None,
            1,
            None,
            None,
            &Default::default(),
            &Subs::default(),
            None,
        )
        .expect("started");
        let p = ended(&job);
        let log = log_of(&p);
        assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
        if resumable {
            assert_eq!(std::fs::read(&part).unwrap(), earlier, "left alone");
            assert_eq!(
                names(out.path()),
                ["Talk.mp4", "Talk.mp4.part", "Talk.mp4.ytdl"]
            );
        } else {
            assert_eq!(
                std::fs::read(out.path().join("Talk (recording 1).ts")).unwrap(),
                earlier
            );
            assert_eq!(names(out.path()), ["Talk (recording 1).ts", "Talk.mp4"]);
            assert!(
                log.iter()
                    .any(|t| t == "The earlier recording was kept as Talk (recording 1).ts"),
                "{log:?}"
            );
        }
    }

    // The live stream ends by itself while recorded: yt-dlp's own file, no ffmpeg of MDM's.
    // The size shown (the larger of what came and what is recorded) never falls back to
    // nothing on the way from the recording to the end.
    let (_tools, out, exe, ffmpeg) = folders("live-ends", "");
    let job = start(&exe, out.path(), Some(&ffmpeg), true);
    let end = Instant::now() + Duration::from_secs(60);
    let (mut seen, mut dips, mut polls) = (false, 0, 0);
    let p = loop {
        let p = job.progress();
        if p.state != State::Running {
            break p;
        }
        if p.downloaded.max(p.recorded) > 0 {
            seen = true;
        } else if seen {
            dips += 1;
        }
        polls += 1;
        assert!(Instant::now() < end, "still running: {:?}", log_of(&p));
        std::thread::sleep(Duration::from_millis(2));
    };
    assert!(seen, "the recording was shown");
    assert_eq!(
        dips, 0,
        "the size shown fell to 0 in {dips} of {polls} looks"
    );
    assert_eq!(
        p.state,
        State::Done(out.path().join("Talk.mp4")),
        "{:?}",
        log_of(&p)
    );
    let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
    assert!(mp4.starts_with(b"FAKEMP4 "), "yt-dlp's own");
    assert_eq!((p.downloaded, p.recorded), (mp4.len() as u64, 0));
    assert_eq!(names(out.path()), ["Talk.mp4"]);
    assert_eq!(
        count(out.path().join("ffmpeg-runs.txt")),
        0,
        "MDM's ffmpeg not run"
    );
    assert_eq!(count(out.path().join("tries.txt")), 1);
    println!("recording_checks: a live stream that ended by itself, {polls} looks, no dip");

    // Windows: an earlier recording another program holds (it can't be renamed): yt-dlp isn't
    // started, the error names it, and it stays as it is.
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let (_tools, out, exe, ffmpeg) = folders("live", "");
        let part = out.path().join("Talk.mp4.part");
        std::fs::write(&part, &earlier).unwrap();
        // Shared for reading and writing, not for deleting (or renaming).
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(&part)
            .unwrap();
        let job = start(&exe, out.path(), Some(&ffmpeg), true);
        let p = ended(&job);
        drop(held);
        let State::Failed(e) = &p.state else {
            panic!("{:?}", p.state)
        };
        assert!(e.contains("Talk.mp4.part"), "{e}");
        assert_eq!(count(out.path().join("tries.txt")), 0, "yt-dlp not started");
        assert_eq!(std::fs::read(&part).unwrap(), earlier);
        assert_eq!(names(out.path()), ["Talk.mp4.part"]);
    }

    // Windows: another program holds the MP4 ffmpeg made (it can't be renamed, nor removed):
    // the recording is kept as the MPEG-TS it is, byte for byte, and the leftover stays. Held
    // by this program, not a child of ffmpeg's: ffmpeg runs in a job that ends all it started.
    #[cfg(windows)]
    {
        use std::io::Read;
        use std::os::windows::fs::OpenOptionsExt;
        let (_tools, out, exe, ffmpeg) = folders("live", "linger");
        let job = start(&exe, out.path(), Some(&ffmpeg), true);
        until(&job, "recording", &recording);
        job.save();
        let temp = out.path().join("Talk.temp.mp4");
        let end = Instant::now() + Duration::from_secs(60);
        let mut held = loop {
            if std::fs::metadata(&temp).is_ok_and(|m| m.len() > 8) {
                break std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(0x1 | 0x2)
                    .open(&temp)
                    .unwrap();
            }
            assert!(Instant::now() < end, "no MP4 made");
            assert_eq!(
                job.progress().state,
                State::Running,
                "ended before it was held"
            );
            std::thread::sleep(Duration::from_millis(5));
        };
        let p = ended(&job);
        let log = log_of(&p);
        let mut made = Vec::new();
        held.read_to_end(&mut made).unwrap();
        drop(held);
        assert_eq!(p.state, State::Done(out.path().join("Talk.ts")), "{log:?}");
        assert_eq!(
            std::fs::read(out.path().join("Talk.ts")).unwrap(),
            made[8..],
            "the recording as ffmpeg read it"
        );
        assert_eq!(names(out.path()), ["Talk.temp.mp4", "Talk.ts"]);
        assert!(
            log.iter()
                .any(|t| t.starts_with("The recording couldn't be made an MP4 (")
                    && t.contains("Talk.temp.mp4 couldn't be renamed")),
            "{log:?}"
        );
    }

    // Windows: another program holds the recording once its MP4 is made (it can't be
    // removed): saved all the same, the log says what is left.
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let (_tools, out, exe, ffmpeg) = folders("live", "");
        let job = start(&exe, out.path(), Some(&ffmpeg), true);
        until(&job, "recording", &recording);
        let part = out.path().join("Talk.mp4.part");
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(&part)
            .unwrap();
        job.save();
        let p = ended(&job);
        drop(held);
        let log = log_of(&p);
        assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
        assert!(
            log.iter().any(
                |t| t.starts_with("The recording is saved, but its unfinished file")
                    && t.contains("Talk.mp4.part couldn't be removed")
            ),
            "{log:?}"
        );
        let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
        assert_eq!(
            mp4[8..],
            std::fs::read(&part).unwrap(),
            "all of it in the MP4"
        );
        assert_eq!(names(out.path()), ["Talk.mp4", "Talk.mp4.part"]);
    }
    // Windows: the MP4 is shorter than the recording, and another program holds the recording
    // (it can't be renamed aside): saved all the same, the recording left as it is, never
    // removed, and the log says it couldn't be renamed.
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let (_tools, out, exe, ffmpeg) = folders("live", "");
        std::fs::write(out.path().join("ffprobe-mode.txt"), "short").unwrap();
        let job = start(&exe, out.path(), Some(&ffmpeg), true);
        until(&job, "enough recorded", &enough);
        let part = out.path().join("Talk.mp4.part");
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(&part)
            .unwrap();
        job.save();
        let p = ended(&job);
        drop(held);
        let log = log_of(&p);
        assert_eq!(p.state, State::Done(out.path().join("Talk.mp4")), "{log:?}");
        assert!(
            log.iter().any(|t| t
                .starts_with("The MP4 doesn't hold all of the recording (it plays shorter, or one of its streams starts later or ends sooner), and the recording ")
                && t.contains("Talk.mp4.part couldn't be renamed (")
                && t.ends_with("it stays as Talk.mp4.part")),
            "{log:?}"
        );
        assert!(
            !log.iter().any(|t| t.contains("kept beside it too")),
            "{log:?}"
        );
        let mp4 = std::fs::read(out.path().join("Talk.mp4")).unwrap();
        assert_eq!(
            std::fs::read(&part).unwrap(),
            mp4[8..],
            "the recording as it was"
        );
        assert_eq!(names(out.path()), ["Talk.mp4", "Talk.mp4.part"]);
    }
    println!("fake_ytdlp: the recordings' checks ok");
}
