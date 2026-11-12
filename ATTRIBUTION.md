# Third-party assets

| Asset | Author | Licence | Where |
|---|---|---|---|
| flag-icons 7.6.0, all 250 country flags in one picture (where a download's server is: Download File Info, the progress window, Properties), drawn by `crates/geo/examples/flag_atlas.rs` | Panayiotis Lipiridis and contributors, <https://github.com/lipis/flag-icons> | MIT — `crates/geo/data/LICENSE-flag-icons` | `crates/geo/data/flags.png` |
| DB-IP IP to Country Lite, a monthly edition (which country a download's server is in, looked up on the computer, never sent anywhere), packed by `tools/geo/build.py` | DB-IP, <https://db-ip.com>: IP Geolocation by DB-IP | Creative Commons Attribution 4.0 International, <https://creativecommons.org/licenses/by/4.0/>; the edition used is in `crates/geo/data/SOURCE.txt` | `crates/geo/data/ipcountry.bin.xz` |
| Patrick Hand (the README's hand-drawn pictures, as outlines; not in the app) | Patrick Wagesreiter | SIL Open Font License 1.1 — `tools/readme/node_modules/@fontsource/patrick-hand/LICENSE` | `docs/images/races-*.svg` |
| *Big Buck Bunny* (the film in the README's pictures of the download button over a video; not in the app) | Blender Foundation, <https://peach.blender.org> | Creative Commons Attribution 3.0 | `docs/images/video-button-*.png` |
| Tabler Icons 3.31.0, its outline icons (every icon MDM draws: the 53 slots of Graphite's icon set, toolbar, categories, file types, statuses, the Scheduler's buttons and the title-bar buttons, and its 37 extras, the Options pages' headers, the video helpers window, Download complete's drag handle, the Scheduler's queues, the Server row, the menus, buttons and banners), drawn white at stroke 1.75 to be tinted, by `design/graphite/source/export_icons.py` | Paweł Kuna, <https://tabler.io/icons> | MIT — `crates/app/assets/themes/graphite/icons/LICENSE` | `crates/app/assets/themes/graphite/icons/` (the design files' copy in `design/graphite/assets/icons/`, the package in `design/graphite/vendor/`) |
| Geist 1.900 (release v1.9.2), Regular, Medium and SemiBold (MDM's typeface, built into the app), unchanged | Vercel and the Geist Project Authors, <https://github.com/vercel/geist-font> | SIL Open Font License 1.1 — `crates/app/assets/fonts/geist/OFL.txt` | `crates/app/assets/fonts/geist/` (the design files' copy and its sources in `design/graphite/assets/fonts/`) |

The app icon (`art/Mister Download Manager.png`) was made for Mister Download Manager by its
author, Ahmed Nefzaoui, with ChatGPT (OpenAI's image generation); every icon size is built from
it by `tools/icon/build_icon.py`.

The README's block icons (`docs/images/icons/`: arrow, globe, gift, magnet) were made for it
with OpenAI's GPT-5 Image Mini by `tools/readme/icons.py`, given the app icon as the style to follow.

## Archive extraction (mdm-extract)

The libraries Link grabber's archive extraction is built on (all compatible with MDM's GPL-3.0):

| Library | Licence | For |
|---|---|---|
| [zip](https://crates.io/crates/zip) 9.1 | MIT | .zip, with [deflate64](https://crates.io/crates/deflate64) (MIT) and WinZip AES ([RustCrypto](https://github.com/RustCrypto) aes, hmac, pbkdf2: MIT OR Apache-2.0) |
| [lzma-rust2](https://crates.io/crates/lzma-rust2) 0.23 | Apache-2.0 | .xz |
| [tar](https://crates.io/crates/tar) 0.4 | MIT OR Apache-2.0 | .tar |

7z and RAR archives are extracted by [7-Zip](https://www.7-zip.org/) (GNU LGPL, with the
unRAR restriction for its RAR code), run as a separate program when the user has it installed.
It is neither bundled nor linked: UnRAR's licence doesn't allow its code in a GPL program, and a
built-in 7z decoder made MDM about 1 MB larger.

System resources used at run time and **not** bundled: Windows fonts (Segoe UI, Tahoma,
Arial) and the Windows shell's file-type icon (the picture of a file dragged out of Download
complete).

MDM contains no artwork, text or code from Internet Download Manager (see the README).
