---
name: Windows 10 and 11
title: Download manager for Windows 10 and 11, installed or portable
headline: Made for Windows.
line: Stable on Windows 10 and 11, installed or portable.
lede: MDM is built for 64-bit Windows 10 and 11, as an installer, a portable copy or a plain zip.
description: Mister Download Manager runs on Windows 10 and 11, as an installer, a portable copy or a plain zip.
group: app
order: 340
since: 0.1.0
shot: /assets/screens/main-light.webp
shot_alt: The main window on Windows 11: the toolbar, the categories on the left and the download list on the right
where: The download page: pick your system's file
help: getting-started.md#choose-a-download
---

## Windows: three ways to have it

On 64-bit Windows 10 and 11, MDM is the stable, recommended build. The [download
page](/download.html) offers three files:

- **The installer** (`setup.exe`), the one to pick unless you have a reason not to. It installs
  for you alone without administrator rights, or for everyone, and can set up browser
  integration and open `.torrent` files and magnet links with MDM.
- **The portable copy** (`portable.zip`), which runs from any folder.
- **The plain zip** (`x64.zip`), the same programs without an installer.

**Help → Check for updates → Update now** downloads a new version, checks its signature, puts it
in place and restarts, for the setup and both zips.

## Portable: everything in one folder

Unzip the portable copy anywhere, a USB stick included. The download list, the settings, the
video helpers and torrent data stay in a `data` folder next to the program, and nothing is
written to the registry. Two things need the installer instead: starting with Windows, and
taking downloads from the browser, since both go through the registry. [Carry MDM on a USB
stick](guide-usb-stick.md) shows the set-up.
