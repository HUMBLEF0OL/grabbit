# Graphite shots: the graphite theme

Made by `design/graphite/source/baseline.ps1`. Retaken from c3541d0 on 2026-11-11 (light and dark): main-plain, main-ctx, progress-status, complete, sched-sched; the other pictures are from earlier runs.
The `graphite` theme, interface size 100 %, light (`<state>.png`) and dark (`<state>-dark.png`).
Windows' display scale was 200 %, so the pictures are 2x the mockup's CSS pixels.
5 of 100 mock states have a picture; the rest have no equivalent today or no way to open it
from the command line (the reason is in the last column), or weren't asked for in a -Shots run.

| Mock state | Opened with | Pictures | Note |
| --- | --- | --- | --- |
| `main-details` | `@main` | not in this run | No details pane today: the main window as it is |
| `main-plain` | `@main` | `main-plain.png`, `main-plain-dark.png` |  |
| `main-ctx` | `@menu:menu` | `main-ctx.png`, `main-ctx-dark.png` | The first download's right-click menu |
| `main-menubar` | `@menu:menu:0` | not in this run | Tasks menu |
| `main-menu-file` | `@menu:menu:1` | not in this run |  |
| `main-menu-downloads` | `@menu:menu:2` | not in this run |  |
| `main-menu-view` | `@menu:menu:3` | not in this run |  |
| `main-menu-help` | `@menu:menu:4` | not in this run |  |
| `main-blank` |  | none today | No /show for the right-click menu on blank space |
| `main-catmenu` |  | none today | No /show for a category's right-click menu |
| `main-qmenu` |  | none today | No /show for a queue's right-click menu |
| `main-cols` |  | none today | No /show for the column chooser |
| `main-filter` |  | none today | No /show for a column filter |
| `main-empty` | `empty` | not in this run | A first start: nothing in the list |
| `main-filtered` |  | none today | No /show for filters that hide every row |
| `addurl` | `addurl` | not in this run |  |
| `fileinfo` | `@download:gen/250M/project-files-2026.zip` | not in this run |  |
| `later` |  | none today | Opened from Download File Info's Download Later; no /show |
| `dup` | `@download:gen/4M/project-report-2026.pdf` | not in this run | The address of a download already in the list |
| `progress-status` | `progress:status` | `progress-status.png`, `progress-status-dark.png` |  |
| `progress-limit` | `progress:limit` | not in this run |  |
| `progress-done` | `progress:done` | not in this run |  |
| `progress-menu` |  | none today | No /show for the tab row's menu |
| `limiter` | `speedlimit` | not in this run |  |
| `complete` | `@complete:gen/2M/release-notes-2026.pdf` | `complete.png`, `complete-dark.png` |  |
| `props` | `properties` | not in this run |  |
| `torrent` | `@torrent` | not in this run | tools/lab/Open Movie Night.torrent |
| `tprog-general` | `tprog:general` | not in this run |  |
| `tprog-files` | `tprog:files` | not in this run |  |
| `tprog-peers` | `tprog:peers` | not in this run |  |
| `tprog-trackers` | `tprog:trackers` | not in this run |  |
| `tprog-pieces` | `tprog:pieces` | not in this run |  |
| `tbusy` |  | none today | A magnet link being looked up; no /show |
| `trackers` |  | none today | /show add-trackers needs a torrent as the list's first download |
| `already` | `existing-files` | not in this run |  |
| `tset` | `torrent-settings` | not in this run |  |
| `helpers` | `helpers` | not in this run |  |
| `video` | `video-subs` | not in this run | The sample video with subtitles |
| `playlist` | `video-playlist:7` | not in this run |  |
| `many` |  | none today | No /show for Add many downloads |
| `gallery` | `gallery` | not in this run |  |
| `helperswin` | `video-helpers` | not in this run |  |
| `batch` | `batch` | not in this run |  |
| `grabber` | `grabber` | not in this run |  |
| `rss` | `rss` | not in this run |  |
| `sched-sched` | `scheduler` | `sched-sched.png`, `sched-sched-dark.png` |  |
| `sched-sched-blocked` |  | none today | Depends on this computer's power state; no /show |
| `sched-sched-off` |  | none today | Depends on this computer's power state; no /show |
| `sched-sched-plugged` |  | none today | Depends on this computer's power state; no /show |
| `sched-sched-noalarm` |  | none today | Depends on this computer's power state; no /show |
| `sched-sched-linux` |  | none today | Not on Windows (MDM is Windows only) |
| `sched-sched-mac` |  | none today | Not on Windows (MDM is Windows only) |
| `sched-files` |  | none today | No /show for the Scheduler's Files in the queue tab |
| `sched-limits` |  | none today | No /show for the Scheduler's Download limits tab |
| `qname` |  | none today | No /show for Enter queue name |
| `qdone` |  | none today | Shown when a queue finishes; no /show |
| `options-general` | `options:general` | not in this run |  |
| `options-file-types` | `options:types` | not in this run |  |
| `options-save-to` | `options:saveto` | not in this run |  |
| `options-downloads` | `options:downloads` | not in this run |  |
| `options-connection` | `options:connection` | not in this run |  |
| `options-remote` | `options:remote` | not in this run |  |
| `options-proxy-socks` | `options:proxy` | not in this run |  |
| `options-sites-logins` | `options:logins` | not in this run |  |
| `options-accounts` | `options:accounts` | not in this run |  |
| `options-sounds` | `options:sounds` | not in this run |  |
| `options-regional` | `options:regional` | not in this run |  |
| `options-archives` | `options:archives` | not in this run |  |
| `dns` |  | none today | No /show for DNS settings |
| `acct` | `sub:options:account` | not in this run | The dialog over Options |
| `pair` | `sub:options:pair` | not in this run | The dialog over Options |
| `testbi` | `sub:options:test` | not in this run | The dialog over Options |
| `panels` | `options:panels` | not in this run |  |
| `leftb` |  | none today | No /show for Addresses left to the browser |
| `keys` |  | none today | No /show for Keys |
| `newlogin` |  | none today | No /show for New login |
| `newexc` |  | none today | No /show for New exception |
| `socks` |  | none today | A proposal: no such window today |
| `virus` |  | none today | A proposal: no such window today |
| `find` | `find` | not in this run |  |
| `cat` | `category:music` | not in this run | Today's nearest: a category's Properties (Add category has no /show) |
| `delcat` | `category-delete:music` | not in this run |  |
| `delete` | `delete` | not in this run |  |
| `redl` |  | none today | No /show for Redownload |
| `error` |  | none today | Needs a failed download; no /show |
| `tray` |  | none today | The notification area's menu; no /show |
| `drop` |  | none today | The drop target is off in a test copy; no /show |
| `ext` | `extension` | not in this run |  |
| `upd` | `updates` | not in this run | Asks GitHub: only with -Updates |
| `about` | `about` | not in this run |  |
| `sum` | `checksum` | not in this run |  |
| `crash` | `crash` | not in this run |  |
| `archpw` |  | none today | Asked while extracting a locked archive; no /show |
| `zip` |  | none today | Asked when 7-Zip is missing; no /show |
| `mkfolder` |  | none today | No /show for Create folder |
| `limits` |  | none today | Shown when a download limit is reached; no /show |
| `generic` |  | none today | A notice with nothing of its own to open it |
| `shortcuts` | `shortcuts` | not in this run |  |
| `friend` | `tell-friend` | not in this run |  |
| `safari` |  | none today | macOS only: not on Windows |
