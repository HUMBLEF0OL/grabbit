# The baseline screenshots for the Graphite phases (plans/01-prepare.md, section 6): today's app
# in every window and state the mockup shows that it can open, Classic theme, light and dark,
# interface size 100 %. Each file is named after its mock state (`main-plain.png`,
# `progress-status.png`; `-dark` before `.png` for the dark one), so a PR pairs it with its
# after-shot. INDEX.md beside them lists every mock state, its file or why there is none.
#
# Nothing is clicked or typed: tools/lab/docs-shots.ps1 opens each window off the screens with
# the app's own /show command, in a copy of its own (its own data folder, lab server and
# downloads); tools/lab/progress-shots.ps1 takes the torrent window's tabs (with -Torrents).
#
#   cargo build --release -p mdm-app -p mdm-labserver -p mdm-cli
#   cargo build --release -p mdm-torrent --example swarm        # for -Torrents
#   powershell -ExecutionPolicy Bypass -File design\graphite\source\baseline.ps1 -Torrents
#
# The output (design/graphite/baseline/) is not committed: it goes to the pr-shots branch as
# baseline/. The raw pictures stay in target\graphite-baseline\{light,dark}.
param(
  # Where the release build is (CARGO_TARGET_DIR's release folder when it is set elsewhere).
  [string]$Bin = "",
  [string]$OutDir = "",
  # The run's folder: its path shows in a few windows, so short and neutral. It must not exist
  # (the shot scripts delete it at the start and the end).
  [string]$Work = "D:\Demo",
  # Also the torrent window's five tabs (needs the swarm example and mdm-cli built).
  [switch]$Torrents,
  # Also Help → Check for updates, which asks GitHub.
  [switch]$Updates,
  # Only light or only dark ("light", "dark"); both when empty.
  [string]$Mode = ""
)
$ErrorActionPreference = "Stop"
$graphite = Split-Path -Parent $PSScriptRoot
$root = Split-Path -Parent (Split-Path -Parent $graphite)
$lab = Join-Path $root "tools\lab"
if (-not $Bin) {
  $Bin = if ($env:CARGO_TARGET_DIR) { Join-Path $env:CARGO_TARGET_DIR "release" } else { Join-Path $root "target\release" }
}
$Bin = (Resolve-Path $Bin).Path
$exe = Join-Path $Bin "misterdownloadmanager.exe"
$server = Join-Path $Bin "labserver.exe"
foreach ($f in $exe, $server) { if (-not (Test-Path $f)) { throw "no ${f}: build it first (see the header)" } }
$out = if ($OutDir) { $OutDir } else { Join-Path $graphite "baseline" }
New-Item -ItemType Directory -Force $out | Out-Null
$raw = Join-Path $root "target\graphite-baseline"
if (Test-Path $Work) { throw "$Work exists: give -Work a folder that doesn't (the run deletes it)" }

# Every mock state (grabbit-design.html: its 60 windows, the states of the five that have them),
# in the page's order: file name, then how today's app shows it. A docs-shots.ps1 -Show value
# ("addurl", "@main", "@menu:menu:1", "@download:<path>"), or "empty" (the main window of a
# first start), "crash" (the crash report), "tprog:<tab>" (progress-shots.ps1), "updates"; or
# "-" and why there is nothing to shoot today.
$states = @(
  @("main-details", "@main", "No details pane today: the main window as it is"),
  @("main-plain", "@main", ""),
  # Before the menu bar's menus: /show menu (a row's menu) leaves an open menu-bar menu open.
  @("main-ctx", "@menu:menu", "The first download's right-click menu"),
  @("main-menubar", "@menu:menu:0", "Tasks menu"),
  @("main-menu-file", "@menu:menu:1", ""),
  @("main-menu-downloads", "@menu:menu:2", ""),
  @("main-menu-view", "@menu:menu:3", ""),
  @("main-menu-help", "@menu:menu:4", ""),
  @("main-blank", "-", "No /show for the right-click menu on blank space"),
  @("main-catmenu", "-", "No /show for a category's right-click menu"),
  @("main-qmenu", "-", "No /show for a queue's right-click menu"),
  @("main-cols", "-", "No /show for the column chooser"),
  @("main-filter", "-", "No /show for a column filter"),
  @("main-empty", "empty", "A first start: nothing in the list"),
  @("main-filtered", "-", "No /show for filters that hide every row"),
  @("addurl", "addurl", ""),
  @("fileinfo", "@download:gen/250M/project-files-2026.zip", ""),
  @("later", "-", "Opened from Download File Info's Download Later; no /show"),
  @("dup", "@download:gen/4M/project-report-2026.pdf", "The address of a download already in the list"),
  @("progress-status", "progress:status", ""),
  @("progress-limit", "progress:limit", ""),
  @("progress-done", "progress:done", ""),
  @("progress-menu", "-", "No /show for the tab row's menu"),
  @("limiter", "speedlimit", ""),
  @("complete", "@complete:gen/2M/release-notes-2026.pdf", ""),
  @("props", "properties", ""),
  @("torrent", "@torrent", "tools/lab/Open Movie Night.torrent"),
  @("tprog-general", "tprog:general", ""),
  @("tprog-files", "tprog:files", ""),
  @("tprog-peers", "tprog:peers", ""),
  @("tprog-trackers", "tprog:trackers", ""),
  @("tprog-pieces", "tprog:pieces", ""),
  @("tbusy", "-", "A magnet link being looked up; no /show"),
  @("trackers", "-", "/show add-trackers needs a torrent as the list's first download"),
  @("already", "existing-files", ""),
  @("tset", "torrent-settings", ""),
  @("helpers", "helpers", ""),
  @("video", "video-subs", "The sample video with subtitles"),
  @("playlist", "video-playlist:7", ""),
  @("many", "-", "No /show for Add many downloads"),
  @("gallery", "gallery", ""),
  @("helperswin", "video-helpers", ""),
  @("batch", "batch", ""),
  @("grabber", "grabber", ""),
  @("rss", "rss", ""),
  @("sched-sched", "scheduler", ""),
  @("sched-sched-blocked", "-", "Depends on this computer's power state; no /show"),
  @("sched-sched-off", "-", "Depends on this computer's power state; no /show"),
  @("sched-sched-plugged", "-", "Depends on this computer's power state; no /show"),
  @("sched-sched-noalarm", "-", "Depends on this computer's power state; no /show"),
  @("sched-sched-linux", "-", "Not on Windows (MDM is Windows only)"),
  @("sched-sched-mac", "-", "Not on Windows (MDM is Windows only)"),
  @("sched-files", "-", "No /show for the Scheduler's Files in the queue tab"),
  @("sched-limits", "-", "No /show for the Scheduler's Download limits tab"),
  @("qname", "-", "No /show for Enter queue name"),
  @("qdone", "-", "Shown when a queue finishes; no /show"),
  @("options-general", "options:general", ""),
  @("options-file-types", "options:types", ""),
  @("options-save-to", "options:saveto", ""),
  @("options-downloads", "options:downloads", ""),
  @("options-connection", "options:connection", ""),
  @("options-remote", "options:remote", ""),
  @("options-proxy-socks", "options:proxy", ""),
  @("options-sites-logins", "options:logins", ""),
  @("options-accounts", "options:accounts", ""),
  @("options-sounds", "options:sounds", ""),
  @("options-regional", "options:regional", ""),
  @("options-archives", "options:archives", ""),
  @("dns", "-", "No /show for DNS settings"),
  @("acct", "sub:options:account", "The dialog over Options"),
  @("pair", "sub:options:pair", "The dialog over Options"),
  @("testbi", "sub:options:test", "The dialog over Options"),
  @("panels", "options:panels", ""),
  @("leftb", "-", "No /show for Addresses left to the browser"),
  @("keys", "-", "No /show for Keys"),
  @("newlogin", "-", "No /show for New login"),
  @("newexc", "-", "No /show for New exception"),
  @("socks", "-", "A proposal: no such window today"),
  @("virus", "-", "A proposal: no such window today"),
  @("find", "find", ""),
  @("cat", "category:music", "Today's nearest: a category's Properties (Add category has no /show)"),
  @("delcat", "category-delete:music", ""),
  @("delete", "delete", ""),
  @("redl", "-", "No /show for Redownload"),
  @("error", "-", "Needs a failed download; no /show"),
  @("tray", "-", "The notification area's menu; no /show"),
  @("drop", "-", "The drop target is off in a test copy; no /show"),
  @("ext", "extension", ""),
  @("upd", "updates", "Asks GitHub: only with -Updates"),
  @("about", "about", ""),
  @("sum", "checksum", ""),
  @("crash", "crash", ""),
  @("archpw", "-", "Asked while extracting a locked archive; no /show"),
  @("zip", "-", "Asked when 7-Zip is missing; no /show"),
  @("mkfolder", "-", "No /show for Create folder"),
  @("limits", "-", "Shown when a download limit is reached; no /show"),
  @("generic", "-", "A notice with nothing of its own to open it"),
  @("shortcuts", "shortcuts", ""),
  @("friend", "tell-friend", ""),
  @("safari", "-", "macOS only: not on Windows")
)

# docs-shots.ps1's -Show list: the plain triggers first, then the addresses handed over, then
# Download complete last of all (it adds a row at the list's end, and the progress window
# shows the list's last download).
$show = @()
$late = @()
$last = @()
$prog = @()
foreach ($s in $states) {
  $name, $what = $s[0], $s[1]
  if ($what -in "-", "empty", "crash" -or $what -like "tprog:*" -or $what -like "sub:*") { continue }
  if ($what -eq "updates" -and -not $Updates) { continue }
  if ($what -like "@download:*") { $late += "$name=$what"; continue }
  if ($what -like "@complete:*") { $last += "$name=$what"; continue }
  if ($what -like "progress:*") { $prog += , "$name=$what"; continue }
  $show += "$name=$what"
}
# The progress window's tabs apart, other windows between them: a /show progress right after
# the last one's window was closed can find it still closing and open nothing.
$step = [math]::Floor($show.Count / ($prog.Count + 1))
for ($i = $prog.Count - 1; $i -ge 0; $i--) {
  $at = [math]::Min($show.Count, ($i + 1) * $step)
  $show = @($show[0..($at - 1)]) + @($prog[$i]) + @(if ($at -lt $show.Count) { $show[$at..($show.Count - 1)] })
}
$show += $late + $last

# DPI-aware, so the window rectangles the shot scripts read are the real pixels: without it, on
# a display above 100 % every picture comes out black. The scripts run in this process to
# inherit it.
Add-Type -TypeDefinition @"
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public static class GB {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern uint GetDpiForSystem();
  public delegate bool P(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(P f, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static List<KeyValuePair<IntPtr,string>> Of(uint pid) {
    var r = new List<KeyValuePair<IntPtr,string>>();
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && IsWindowVisible(h)) {
        var sb = new StringBuilder(512); GetWindowText(h, sb, 512);
        if (sb.Length > 0) r.Add(new KeyValuePair<IntPtr,string>(h, sb.ToString()));
      }
      return true; }, IntPtr.Zero);
    return r;
  }
}
"@
[GB]::SetProcessDPIAware() | Out-Null
$dpi = [GB]::GetDpiForSystem()

# A copy of the app of its own (data folder $Work\data), off the screens; runs $body with it.
function With-App([bool]$dark, [scriptblock]$body) {
  New-Item -ItemType Directory -Force "$Work\data", "$Work\Downloads" | Out-Null
  $d = $dark.ToString().ToLower()
  Set-Content -Encoding utf8 "$Work\data\downloads.json" (
    '{"next_id":1,"items":[],"version":2,"settings":{"helpers_asked":true,"extension_never_ask":true' +
    ',"dark_mode_support":' + $d + ',"always_dark":' + $d + ',"update_check":false,"theme":"classic"' +
    ',"interface_size":100,"torrent_incoming":false,"torrent_upnp":false,"torrent_dht":false,"torrent_lsd":false' +
    ',"category_dirs":[["General","' + ("$Work\Downloads" -replace '\\', '\\') + '"]]}}')
  $env:MDM_DATA_DIR = "$Work\data"
  $env:MDM_OFFSCREEN = "1"
  $started = Get-Date
  $app = Start-Process -PassThru $exe
  try {
    # Up to 30 s for its main window; MDM may start itself again (a lost renderer, /updated):
    # then the new copy is the one.
    $main = $null
    for ($i = 0; $i -lt 60 -and -not $main; $i++) {
      Start-Sleep -Milliseconds 500
      if ($app.HasExited) {
        $n = @(Get-CimInstance Win32_Process -Filter "Name='$(Split-Path -Leaf $exe)'" |
          Where-Object { $_.ExecutablePath -eq $exe -and $_.CommandLine -like "*/updated*" })[0]
        if ($n) { $app = Get-Process -Id $n.ProcessId }
      }
      $main = @(Titles $app.Id | Where-Object { $_ -like "Mister Download Manager*" })[0]
    }
    if (-not $main) { throw "no main window" }
    Start-Sleep 3
    & $body $app $main
  } finally {
    # Every copy this started (a restarted one and its crash monitor too).
    Get-CimInstance Win32_Process -Filter "Name='$(Split-Path -Leaf $exe)'" |
      Where-Object { $_.ExecutablePath -eq $exe -and $_.CreationDate -ge $started } |
      ForEach-Object { Stop-Process -Id $_.ProcessId -ErrorAction SilentlyContinue }
    Remove-Item Env:MDM_DATA_DIR, Env:MDM_OFFSCREEN -ErrorAction SilentlyContinue
    Start-Sleep 1
    Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
  }
}

# cap-own.ps1 until the picture isn't one colour below the title bar (a window off the screens
# can take a moment to paint), up to 10 s.
function Capture([int]$procId, [string]$title, [string]$file) {
  Add-Type -AssemblyName System.Drawing
  for ($try = 0; $try -lt 20; $try++) {
    Start-Sleep -Milliseconds 500
    try { & (Join-Path $lab "cap-own.ps1") -ProcessId $procId -Title $title -Out $file | Out-Null } catch { continue }
    $bmp = [System.Drawing.Bitmap]::FromFile($file)
    try {
      $first = $bmp.GetPixel(4, 40).ToArgb(); $flat = $true
      for ($y = 40; $y -lt $bmp.Height - 4 -and $flat; $y += 8) {
        for ($x = 4; $x -lt $bmp.Width - 4; $x += 8) { if ($bmp.GetPixel($x, $y).ToArgb() -ne $first) { $flat = $false; break } }
      }
    } finally { $bmp.Dispose() }
    if (-not $flat) { return $true }
  }
  Remove-Item $file -ErrorAction SilentlyContinue
  return $false
}
function Titles([int]$procId) { [GB]::Of([uint32]$procId) | ForEach-Object { $_.Value } }
function Close-All([int]$procId, [string[]]$keep) {
  [GB]::Of([uint32]$procId) | Where-Object { $keep -notcontains $_.Value } |
    ForEach-Object { [GB]::PostMessage($_.Key, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
}

$modes = if ($Mode) { @($Mode) } else { @("light", "dark") }
foreach ($m in $modes) {
  $dark = $m -eq "dark"
  $dir = Join-Path $raw $m
  if (Test-Path $dir) { Remove-Item $dir -Recurse -Force }
  New-Item -ItemType Directory -Force $dir | Out-Null
  "== ${m}: docs-shots.ps1 ($($show.Count) shots and the crash report)"
  $crash = if ($dark) { "crash-report-dark" } else { "crash-report" }
  $p = @{
    Exe = $exe; Server = $server; Torrent = (Join-Path $lab "Open Movie Night.torrent")
    OutDir = $dir; Work = $Work; Only = @($crash); Show = $show; Dark = $dark
    Settings = '"theme":"classic","interface_size":100'
  }
  & (Join-Path $lab "docs-shots.ps1") @p
  $c = Join-Path $dir "$crash.png"
  if (Test-Path $c) { Move-Item -Force $c (Join-Path $dir "crash.png") }

  # A first start: the main window with nothing in the list.
  "== ${m}: the empty main window, the dialogs over Options"
  With-App $dark {
    param($app, $main)
    if (Capture $app.Id ([WildcardPattern]::Escape($main)) (Join-Path $dir "main-empty.png")) { "  main-empty.png" }
    else { Write-Warning "main-empty: no picture" }
    # A dialog over Options (docs-shots.ps1 shoots the first new window, Options itself): the
    # window that is neither the main window nor Options.
    foreach ($s in $states | Where-Object { $_[1] -like "sub:*" }) {
      $name, $what = $s[0], $s[1].Substring(4)
      $before = @(Titles $app.Id)
      & $exe /show $what
      $t = $null
      for ($i = 0; $i -lt 50 -and -not $t; $i++) {
        Start-Sleep -Milliseconds 200
        $t = @(Titles $app.Id | Where-Object { $before -notcontains $_ -and $_ -notlike "*Configuration" })[0]
      }
      if ($t -and (Capture $app.Id ([WildcardPattern]::Escape($t)) (Join-Path $dir "$name.png"))) {
        "  $name.png  ($t)"
      } else { Write-Warning "${name}: no dialog over Options" }
      Close-All $app.Id $before
      Start-Sleep -Milliseconds 800
      Close-All $app.Id $before
      Start-Sleep -Milliseconds 600
    }
  }

  if ($Torrents) {
    "== ${m}: the torrent window's tabs (progress-shots.ps1)"
    # progress-shots.ps1 takes -Bin relative to the repository.
    Push-Location $root
    try { $rel = Resolve-Path -Relative $Bin } finally { Pop-Location }
    $p = @{
      Bin = $rel; OutDir = $dir; Only = @("torrent"); Work = $Work; Dark = $dark
      TorrentTabs = @("general", "files", "peers", "trackers", "pieces")
    }
    try { & (Join-Path $lab "progress-shots.ps1") @p } catch { Write-Warning "progress-shots.ps1 failed ($m): $_" }
    foreach ($tab in "general", "files", "peers", "trackers", "pieces") {
      $f = Join-Path $dir "progress-torrent-$tab.png"
      if (Test-Path $f) { Move-Item -Force $f (Join-Path $dir "tprog-$tab.png") }
    }
    Remove-Item (Join-Path $dir "progress-torrent.png") -ErrorAction SilentlyContinue
  }
}

# Into the baseline folder: <state>.png light, <state>-dark.png dark, only for the modes this
# run took (a -Mode run keeps the other mode's pictures as they were); then the index.
foreach ($m in $modes) {
  $pattern = if ($m -eq "dark") { "*-dark.png" } else { "*.png" }
  Get-ChildItem $out -Filter $pattern -ErrorAction SilentlyContinue |
    Where-Object { $m -eq "dark" -or $_.BaseName -notlike "*-dark" } | Remove-Item
  foreach ($s in $states) {
    $f = Join-Path (Join-Path $raw $m) "$($s[0]).png"
    $to = if ($m -eq "dark") { "$($s[0])-dark.png" } else { "$($s[0]).png" }
    if (Test-Path $f) { Copy-Item $f (Join-Path $out $to) }
  }
}
$rows = @()
$taken = 0
foreach ($s in $states) {
  $name, $what, $note = $s
  $files = @(@("$name.png", "$name-dark.png") | Where-Object { Test-Path (Join-Path $out $_) })
  if ($files.Count) { $taken++ }
  $how = if ($what -eq "-") { "" } else { "``$what``" }
  $shot = if ($files.Count) { ($files | ForEach-Object { "``$_``" }) -join ", " } elseif ($what -eq "-") { "none today" } else { "**not taken**" }
  $rows += "| ``$name`` | $how | $shot | $note |"
}
$commit = (& git -C $root rev-parse --short HEAD).Trim()
$index = @(
  "# Graphite baseline: today's app",
  "",
  "Made by ``design/graphite/source/baseline.ps1`` from $commit; last run on $(Get-Date -Format yyyy-MM-dd) ($($modes -join ' and ')):",
  "Classic theme, interface size 100 %, light (``<state>.png``) and dark (``<state>-dark.png``).",
  "Windows' display scale was $([math]::Round($dpi / 96 * 100)) %, so the pictures are $([math]::Round($dpi / 96, 2))x the mockup's CSS pixels.",
  "$taken of $($states.Count) mock states have a picture; the rest have no equivalent today or no way to open it",
  "from the command line (the reason is in the last column).",
  "",
  "| Mock state | Opened with | Pictures | Note |",
  "| --- | --- | --- | --- |"
) + $rows
Set-Content -Encoding utf8 (Join-Path $out "INDEX.md") ($index -join "`n")
"$taken of $($states.Count) mock states taken; $out\INDEX.md"
