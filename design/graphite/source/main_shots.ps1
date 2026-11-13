# The main window in fixed states, for plan 05's checks (plans/05-main-window.md): the same
# pictures from two builds can be compared pixel for pixel (source/pixel_diff.py), because
# nothing in them moves. A fixed download list (no download runs, no server, the dates written
# in), each theme switched to live with /show theme:<id>, every state opened with /show; both
# modes when -Mode is empty. Off the screens (MDM_OFFSCREEN), drawn as the active window
# (MDM_TEST_ACTIVE).
#
#   cargo build --release -p mdm-app
#   powershell -ExecutionPolicy Bypass -File design\graphite\source\main_shots.ps1 -Bin target\05-before -Out target\05a\before
#
# Pictures: <Out>\<theme>\<state>[-dark].png. States: plain (the list), empty (a first start),
# nocats (the Categories pane hidden), wide (the pane 220 wide, Graphite's 300), menu-0 .. menu-4 (Tasks, File,
# Downloads, View, Help), context (a download's right-click menu), and with -Extra the states
# 05b's /show commands open (blank, node, queue, columns, filter, filtered). The toolbar's
# drop-down menus have no /show: only chrome.rs's layout_tests check their place.
param(
  [Parameter(Mandatory)][string]$Bin,
  [Parameter(Mandatory)][string]$Out,
  [string[]]$Themes = @("graphite"),
  [string]$Mode = "",
  [string[]]$States = @(),
  [switch]$Extra,
  # The run's folder (must not exist; deleted at the end); its path shows in no main-window state.
  [string]$Work = (Join-Path $env:TEMP "mdm-main-shots"),
  [ValidateSet(100, 125, 150, 175, 200)][int]$InterfaceSize = 100
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$lab = Join-Path $root "tools\lab"
$Bin = (Resolve-Path $Bin).Path
$exe = Join-Path $Bin "misterdownloadmanager.exe"
if (-not (Test-Path $exe)) { throw "no $exe" }
if (Test-Path $Work) { throw "$Work exists: give -Work a folder that doesn't (the run deletes it)" }
$Themes = @($Themes | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$States = @($States | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$all = @("plain", "menu-0", "menu-1", "menu-2", "menu-3", "menu-4", "context", "empty", "nocats", "wide")
if ($Extra) { $all += @("blank", "node", "queue", "columns", "filter", "filtered") }
if ($States.Count) {
  $unknown = @($States | Where-Object { $all -notcontains $_ })
  if ($unknown.Count) { throw "no such state (or one of -Extra's without -Extra): $($unknown -join ', ')" }
  $all = @($all | Where-Object { $States -contains $_ })
}

Add-Type -TypeDefinition @"
using System; using System.Text; using System.Collections.Generic; using System.Runtime.InteropServices;
public static class MS {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
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
[MS]::SetProcessDPIAware() | Out-Null
function Titles([int]$procId) { [MS]::Of([uint32]$procId) | ForEach-Object { $_.Value } }
function Close-Menus([int]$procId) {
  [MS]::Of([uint32]$procId) | Where-Object { $_.Value -like "misterdownloadmanager menu*" } |
    ForEach-Object { [MS]::PostMessage($_.Key, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
}

# The fixed list: finished files of several kinds, one stopped part-way, one failed, one
# waiting in the main queue, one in the synchronization queue. Addresses on example.org.
function Items([string]$dl) {
  $e = { param($s) ($s | ConvertTo-Json) }
  $rows = @(
    @(1, "big-buck-bunny-1080p.mp4", "Video", 188743680, 188743680, "Complete", "Nov 09 21:05:12 2026", "Nov 09 21:04:00 2026", $null),
    @(2, "project-report-2026.pdf", "Documents", 4194304, 4194304, "Complete", "Nov 09 20:51:31 2026", "Nov 09 20:51:30 2026", $null),
    @(3, "holiday-photos.zip", "Compressed", 67108864, 67108864, "Complete", "Nov 09 19:30:44 2026", "Nov 09 19:30:20 2026", $null),
    @(4, "podcast-episode-42.mp3", "Music", 39845888, 39845888, "Complete", "Nov 09 18:02:19 2026", "Nov 09 18:02:01 2026", $null),
    @(5, "rust-1.99.1-x86_64-pc-windows-msvc.msi", "Programs", 96468992, 60775464, "Stopped", "Nov 10 08:41:50 2026", "Nov 10 08:40:09 2026", $null),
    @(6, "conference-keynote-2026.webm", "Video", 1288490188, 0, "Error", "Nov 10 08:57:02 2026", "Nov 10 08:55:14 2026", "The server answered 404 Not Found"),
    @(7, "noto-sans-cjk-regular.otf", "General", 17196646, 0, "Queued", $null, "Nov 10 08:38:22 2026", $null),
    @(8, "open-data-2026.csv", "Documents", 12582912, 12582912, "Complete", "Nov 09 17:45:08 2026", "Nov 09 17:45:07 2026", $null)
  )
  $json = foreach ($r in $rows) {
    $id, $name, $cat, $size, $got, $st, $last, $added, $err = $r
    $dir = if ($cat -eq "General") { $dl } else { Join-Path $dl $cat }
    '{"id":' + $id + ',"url":' + (& $e "https://downloads.example.org/files/$name") +
    ',"file_name":' + (& $e $name) + ',"dest":' + (& $e (Join-Path $dir $name)) +
    ',"size":' + $size + ',"downloaded":' + $got + ',"status":"' + $st + '","category":"' + $cat +
    '","description":"","last_try":' + $(if ($last) { & $e $last } else { "null" }) +
    ',"error":' + $(if ($err) { & $e $err } else { "null" }) + ',"resumable":true' +
    ',"added":' + (& $e $added) + '}'
  }
  "[" + ($json -join ",") + "]"
}

# A copy of the app with the fixed list (none: `empty`) and `more` settings (JSON members).
function Run-App([bool]$dark, [bool]$empty, [string]$more, [scriptblock]$body) {
  New-Item -ItemType Directory -Force "$Work\data", "$Work\Downloads" | Out-Null
  $d = $dark.ToString().ToLower()
  $items = if ($empty) { "[]" } else { Items "$Work\Downloads" }
  # Queue 0 (main) holds the waiting one, the synchronization queue (1) nothing.
  $queues = if ($empty) { "" } else { ',"queues":[{"name":"Main download queue","files_at_once":4,"items":[7],"sync":false},{"name":"Synchronization queue","files_at_once":4,"items":[],"sync":true}]' }
  Set-Content -Encoding utf8 "$Work\data\downloads.json" (
    '{"next_id":9,"items":' + $items + ',"version":2' + $queues + ',"settings":{"helpers_asked":true,"extension_never_ask":true' +
    ',"dark_mode_support":' + $d + ',"always_dark":' + $d + ',"update_check":false,"theme":"' + $Themes[0] + '"' +
    ',"interface_size":' + $InterfaceSize + ',"torrent_incoming":false,"torrent_upnp":false,"torrent_dht":false,"torrent_lsd":false' +
    ',"category_dirs":[["General","' + ("$Work\Downloads" -replace '\\', '\\') + '"]]' + $more + '}}')
  $env:MDM_DATA_DIR = "$Work\data"
  $env:MDM_OFFSCREEN = "1"
  $env:MDM_TEST_ACTIVE = "1"
  $started = Get-Date
  $app = Start-Process -PassThru $exe
  try {
    $main = $null
    for ($i = 0; $i -lt 60 -and -not $main; $i++) {
      Start-Sleep -Milliseconds 500
      $main = @(Titles $app.Id | Where-Object { $_ -like "Mister Download Manager*" })[0]
    }
    if (-not $main) { throw "no main window" }
    # The system's file icons come in the background: a first picture taken sooner missed some.
    Start-Sleep 6
    & $body $app $main
  } finally {
    Get-CimInstance Win32_Process -Filter "Name='$(Split-Path -Leaf $exe)'" |
      Where-Object { $_.ExecutablePath -eq $exe -and $_.CreationDate -ge $started } |
      ForEach-Object { Stop-Process -Id $_.ProcessId -ErrorAction SilentlyContinue }
    Remove-Item Env:MDM_DATA_DIR, Env:MDM_OFFSCREEN, Env:MDM_TEST_ACTIVE -ErrorAction SilentlyContinue
    Start-Sleep 1
    Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
  }
}

# cap-own.ps1 until two pictures in a row are the same (nothing still painting), up to 10 s;
# throws when none comes or it never settles.
function Capture([int]$procId, [string]$title, [string]$file) {
  $by = if ($title) { @{ Title = [WildcardPattern]::Escape($title) } } else { @{} }
  Remove-Item -LiteralPath $file -ErrorAction SilentlyContinue
  $prev = $null
  for ($try = 0; $try -lt 20; $try++) {
    Start-Sleep -Milliseconds 500
    try { & (Join-Path $lab "cap-own.ps1") -ProcessId $procId @by -Out $file | Out-Null } catch { continue }
    $hash = (Get-FileHash $file -Algorithm SHA256).Hash
    if ($hash -eq $prev) { return }
    $prev = $hash
  }
  if (-not (Test-Path $file)) { throw "$(Split-Path -Leaf $file): no picture" }
  throw "$(Split-Path -Leaf $file): still changing after 10 s"
}

# Each state: the /show commands that open it (none: the main window as it is) and whether the
# picture is every window of ours composed (a menu over the main window).
$how = @{
  "plain" = @(@(), $false); "empty" = @(@(), $false); "nocats" = @(@(), $false); "wide" = @(@(), $false)
  "menu-0" = @(@("menu:0"), $true); "menu-1" = @(@("menu:1"), $true); "menu-2" = @(@("menu:2"), $true)
  "menu-3" = @(@("menu:3"), $true); "menu-4" = @(@("menu:4"), $true)
  "context" = @(@("context"), $true)
  "blank" = @(@("context:blank"), $true); "node" = @(@("node:all"), $true)
  "queue" = @(@("node:queue"), $true); "columns" = @(@("columns"), $true)
  "filter" = @(@("filter:3"), $false); "filtered" = @(@("filtered"), $false)
}
$modes = if ($Mode) { @($Mode) } else { @("light", "dark") }
foreach ($m in $modes) {
  $dark = $m -eq "dark"
  $suffix = if ($dark) { "-dark" } else { "" }
  # The states that need a start of their own (another list or other settings), and the rest.
  $own = @{ "empty" = @($true, ""); "nocats" = @($false, ',"show_categories":false'); "wide" = @($false, ',"categories_width":220,"graphite_categories_width":300') }
  $runs = @(,@($false, "", @($all | Where-Object { -not $own.ContainsKey($_) })))
  foreach ($s in $all | Where-Object { $own.ContainsKey($_) }) { $runs += ,@($own[$s][0], $own[$s][1], @($s)) }
  foreach ($run in $runs) {
    $empty, $more, $mine = $run
    if (-not $mine.Count) { continue }
    Run-App $dark $empty $more {
      param($app, $main)
      foreach ($t in $Themes) {
        & $exe /show "theme:$t"
        Start-Sleep 2
        $dir = Join-Path $Out $t
        New-Item -ItemType Directory -Force $dir | Out-Null
        foreach ($s in $mine) {
          $cmds, $composed = $how[$s]
          foreach ($c in $cmds) { & $exe /show $c; Start-Sleep -Milliseconds 400 }
          if ($composed) {
            for ($i = 0; $i -lt 50 -and -not (@(Titles $app.Id) -like "misterdownloadmanager menu*"); $i++) { Start-Sleep -Milliseconds 200 }
            if (-not (@(Titles $app.Id) -like "misterdownloadmanager menu*")) { throw "${t} ${s}: no menu opened" }
          }
          Start-Sleep -Milliseconds 800
          $file = Join-Path $dir "$s$suffix.png"
          $title = if ($composed) { "" } else { $main }
          Capture $app.Id $title $file
          "  $t\$s$suffix.png"
          Close-Menus $app.Id
          Start-Sleep -Milliseconds 800
          Close-Menus $app.Id
          # filtered and filter leave the list filtered: clear it.
          if ($s -in "filter", "filtered") { & $exe /show "unfiltered"; Start-Sleep -Milliseconds 400 }
          Start-Sleep -Milliseconds 400
        }
      }
    }
  }
}
