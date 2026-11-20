# Publishes a Grabbit release from this computer: the same downloads, checksums and signature as
# .github/workflows/release.yml's tag run, without GitHub's runners (a release then costs no
# Actions time; only the website's deploy runs there). Its notes are the version's CHANGELOG
# section alone (release.yml also added the contributors' thanks).
#
#   tools/installer/publish.ps1            from a clean checkout of main at the release commit
#   tools/installer/publish.ps1 -NoBuild   the programs in target/release are already this commit's
#
# Steps: this month's IP-to-country table (put back after the build), build.ps1 (installer and
# zips), the symbols zip, the browser extensions, SHA256SUMS.txt signed with Grabbit's release key
# (7C8D515A…, from this computer's gpg keyring), then `gh release create v<version>` on main with
# the version's CHANGELOG section as its notes, and the website rebuilt (pages.yml).
param([switch]$NoBuild, [switch]$DryRun)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$key = "7C8D515A5E859D473CBC54C0D9E89ED61BBEC99D"
$repo = "HUMBLEF0OL/grabbit"

function Invoke-Step([string]$what, [scriptblock]$run) {
  Write-Host "== $what"
  & $run
  if ($LASTEXITCODE) { throw "$what failed ($LASTEXITCODE)" }
}

$v = [regex]::Match((Get-Content (Join-Path $root "Cargo.toml") -Raw), '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $v) { throw "no version in Cargo.toml" }
$tag = "v$v"

# Only main's own commit, with nothing changed, becomes a release.
Push-Location $root
try {
  if (git status --porcelain) { throw "the checkout has changes: commit or stash them first" }
  git fetch -q origin main
  if ((git rev-parse HEAD) -ne (git rev-parse origin/main)) { throw "HEAD isn't origin/main: check out main at the release commit" }
  if (git ls-remote --tags origin "refs/tags/$tag") { throw "$tag exists on origin already" }
  $log = Get-Content CHANGELOG.md -Raw
  $m = [regex]::Match($log, "(?ms)^## \[$([regex]::Escape($v))\][^\n]*\n(.*?)(?=^## |\z)")
  if (-not ($m.Success -and $m.Groups[1].Value.Trim())) { throw "CHANGELOG.md has no [$v] section" }

  $dist = Join-Path $root "target\dist"
  if (-not $NoBuild) {
    Invoke-Step "IP-to-country table" { python tools/geo/build.py --release }
    try {
      Invoke-Step "installer and zips" { & (Join-Path $PSScriptRoot "build.ps1") }
    } finally {
      git checkout -q -- crates/geo/data  # the month's table ships, it isn't committed
    }
  } else {
    Invoke-Step "installer and zips" { & (Join-Path $PSScriptRoot "build.ps1") -NoBuild }
  }
  $bin = Join-Path $root "target\release"
  $exes = "grabbit", "grabbit-nmhost", "grabbit-cli", "grabbit-mcp" | ForEach-Object { Join-Path $bin "$_.exe" }
  Invoke-Step "symbols" { python tools/installer/symbols.py windows-x64 $v $dist @exes }

  Invoke-Step "browser extensions" { node tools/extension/check-pack.mjs }  # pack.ps1 ran in build.ps1
  Copy-Item target/extension/mdm-chromium.zip (Join-Path $dist "Grabbit-$v-extension-chromium.zip")
  Copy-Item target/extension/mdm-firefox.xpi (Join-Path $dist "Grabbit-$v-extension-firefox.xpi")

  # The checksums of everything published, signed: Update installs only what this list names.
  Remove-Item (Join-Path $dist "SHA256SUMS*") -ErrorAction SilentlyContinue
  $files = Get-ChildItem $dist -File | Where-Object { $_.Name -like "Grabbit-$v-*" } | Sort-Object Name
  $sums = $files | ForEach-Object { "$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower())  $($_.Name)" }
  $list = Join-Path $dist "SHA256SUMS.txt"
  Set-Content -Encoding ascii $list $sums
  Invoke-Step "signature" { gpg --batch --yes --detach-sign --local-user $key --output "$list.sig" $list }
  Invoke-Step "signature check" { gpg --batch --verify "$list.sig" $list }

  $notes = $m.Groups[1].Value.Trim() -replace '\]\((?![a-z]+:|#)([^)\s]+)\)', "](https://github.com/$repo/blob/$tag/`$1)"
  $notesFile = Join-Path $dist "notes.md"
  [IO.File]::WriteAllText($notesFile, $notes)  # UTF-8 without a BOM (Set-Content adds one, and GitHub shows it)
  $all = @($files.FullName) + $list + "$list.sig"
  Write-Host "== $tag : $($all.Count) files"
  $all | ForEach-Object { Write-Host "   $(Split-Path -Leaf $_)" }
  if ($DryRun) { Write-Host "dry run: nothing published"; return }
  Invoke-Step "release" { gh release create $tag -R $repo --target main --latest --title "Grabbit $tag" --notes-file $notesFile @all }
  Invoke-Step "website" { gh workflow run pages.yml -R $repo --ref main }
} finally {
  Pop-Location
}
