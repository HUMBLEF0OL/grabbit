# Graphite baseline: today's app

Made by `design/graphite/source/baseline.ps1` from 514674e on 2026-11-11:
Classic theme, interface size 100 %, light (`<state>.png`) and dark (`<state>-dark.png`).
Windows' display scale was 200 %, so the pictures are 2x the mockup's CSS pixels.
61 of 100 mock states have a picture; the rest have no equivalent today or no way to open it
from the command line (the reason is in the last column).

| Mock state | Opened with | Pictures | Note |
| --- | --- | --- | --- |
| `main-details` | `@main` | `main-details.png`, `main-details-dark.png` | No details pane today: the main window as it is |
| `main-plain` | `@main` | `main-plain.png`, `main-plain-dark.png` |  |
| `main-ctx` | `@menu:menu` | `main-ctx.png`, `main-ctx-dark.png` | The first download's right-click menu |
| `main-menubar` | `@menu:menu:0` | `main-menubar.png`, `main-menubar-dark.png` | Tasks menu |
| `main-menu-file` | `@menu:menu:1` | `main-menu-file.png`, `main-menu-file-dark.png` |  |
| `main-menu-downloads` | `@menu:menu:2` | `main-menu-downloads.png`, `main-menu-downloads-dark.png` |  |
| `main-menu-view` | `@menu:menu:3` | `main-menu-view.png`, `main-menu-view-dark.png` |  |
| `main-menu-help` | `@menu:menu:4` | `main-menu-help.png`, `main-menu-help-dark.png` |  |
| `main-blank` |  | none today | No /show for the right-click menu on blank space |
| `main-catmenu` |  | none today | No /show for a category's right-click menu |
| `main-qmenu` |  | none today | No /show for a queue's right-click menu |
| `main-cols` |  | none today | No /show for the column chooser |
| `main-filter` |  | none today | No /show for a column filter |
| `main-empty` | `empty` | `main-empty.png`, `main-empty-dark.png` | A first start: nothing in the list |
| `main-filtered` |  | none today | No /show for filters that hide every row |
| `addurl` | `addurl` | `addurl.png`, `addurl-dark.png` |  |
| `fileinfo` | `@download:gen/250M/project-files-2026.zip` | `fileinfo.png`, `fileinfo-dark.png` |  |
| `later` |  | none today | Opened from Download File Info's Download Later; no /show |
| `dup` | `@download:gen/4M/project-report-2026.pdf` | `dup.png`, `dup-dark.png` | The address of a download already in the list |
| `progress-status` | `progress:status` | `progress-status.png`, `progress-status-dark.png` |  |
| `progress-limit` | `progress:limit` | `progress-limit.png`, `progress-limit-dark.png` |  |
| `progress-done` | `progress:done` | `progress-done.png`, `progress-done-dark.png` |  |
| `progress-menu` |  | none today | No /show for the tab row's menu |
| `limiter` | `speedlimit` | `limiter.png`, `limiter-dark.png` |  |
| `complete` | `@complete:gen/2M/release-notes-2026.pdf` | `complete.png`, `complete-dark.png` |  |
| `props` | `properties` | `props.png`, `props-dark.png` |  |
| `torrent` | `@torrent` | `torrent.png`, `torrent-dark.png` | tools/lab/Open Movie Night.torrent |
| `tprog-general` | `tprog:general` | `tprog-general.png`, `tprog-general-dark.png` |  |
| `tprog-files` | `tprog:files` | `tprog-files.png`, `tprog-files-dark.png` |  |
| `tprog-peers` | `tprog:peers` | `tprog-peers.png`, `tprog-peers-dark.png` |  |
| `tprog-trackers` | `tprog:trackers` | `tprog-trackers.png`, `tprog-trackers-dark.png` |  |
| `tprog-pieces` | `tprog:pieces` | `tprog-pieces.png`, `tprog-pieces-dark.png` |  |
| `tbusy` |  | none today | A magnet link being looked up; no /show |
| `trackers` |  | none today | /show add-trackers needs a torrent as the list's first download |
| `already` | `existing-files` | `already.png`, `already-dark.png` |  |
| `tset` | `torrent-settings` | `tset.png`, `tset-dark.png` |  |
| `helpers` | `helpers` | `helpers.png`, `helpers-dark.png` |  |
| `video` | `video-subs` | `video.png`, `video-dark.png` | The sample video with subtitles |
| `playlist` | `video-playlist:7` | `playlist.png`, `playlist-dark.png` |  |
| `many` |  | none today | No /show for Add many downloads |
| `gallery` | `gallery` | `gallery.png`, `gallery-dark.png` |  |
| `helperswin` | `video-helpers` | `helperswin.png`, `helperswin-dark.png` |  |
| `batch` | `batch` | `batch.png`, `batch-dark.png` |  |
| `grabber` | `grabber` | `grabber.png`, `grabber-dark.png` |  |
| `rss` | `rss` | `rss.png`, `rss-dark.png` |  |
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
| `options-general` | `options:general` | `options-general.png`, `options-general-dark.png` |  |
| `options-file-types` | `options:types` | `options-file-types.png`, `options-file-types-dark.png` |  |
| `options-save-to` | `options:saveto` | `options-save-to.png`, `options-save-to-dark.png` |  |
| `options-downloads` | `options:downloads` | `options-downloads.png`, `options-downloads-dark.png` |  |
| `options-connection` | `options:connection` | `options-connection.png`, `options-connection-dark.png` |  |
| `options-remote` | `options:remote` | `options-remote.png`, `options-remote-dark.png` |  |
| `options-proxy-socks` | `options:proxy` | `options-proxy-socks.png`, `options-proxy-socks-dark.png` |  |
| `options-sites-logins` | `options:logins` | `options-sites-logins.png`, `options-sites-logins-dark.png` |  |
| `options-accounts` | `options:accounts` | `options-accounts.png`, `options-accounts-dark.png` |  |
| `options-sounds` | `options:sounds` | `options-sounds.png`, `options-sounds-dark.png` |  |
| `options-regional` | `options:regional` | `options-regional.png`, `options-regional-dark.png` |  |
| `options-archives` | `options:archives` | `options-archives.png`, `options-archives-dark.png` |  |
| `dns` |  | none today | No /show for DNS settings |
| `acct` | `sub:options:account` | `acct.png`, `acct-dark.png` | The dialog over Options |
| `pair` | `sub:options:pair` | `pair.png`, `pair-dark.png` | The dialog over Options |
| `testbi` | `sub:options:test` | `testbi.png`, `testbi-dark.png` | The dialog over Options |
| `panels` | `options:panels` | `panels.png`, `panels-dark.png` |  |
| `leftb` |  | none today | No /show for Addresses left to the browser |
| `keys` |  | none today | No /show for Keys |
| `newlogin` |  | none today | No /show for New login |
| `newexc` |  | none today | No /show for New exception |
| `socks` |  | none today | A proposal: no such window today |
| `virus` |  | none today | A proposal: no such window today |
| `find` | `find` | `find.png`, `find-dark.png` |  |
| `cat` | `category:music` | `cat.png`, `cat-dark.png` | Today's nearest: a category's Properties (Add category has no /show) |
| `delcat` | `category-delete:music` | `delcat.png`, `delcat-dark.png` |  |
| `delete` | `delete` | `delete.png`, `delete-dark.png` |  |
| `redl` |  | none today | No /show for Redownload |
| `error` |  | none today | Needs a failed download; no /show |
| `tray` |  | none today | The notification area's menu; no /show |
| `drop` |  | none today | The drop target is off in a test copy; no /show |
| `ext` | `extension` | `ext.png`, `ext-dark.png` |  |
| `upd` | `updates` | **not taken** | Asks GitHub: only with -Updates |
| `about` | `about` | `about.png`, `about-dark.png` |  |
| `sum` | `checksum` | `sum.png`, `sum-dark.png` |  |
| `crash` | `crash` | `crash.png`, `crash-dark.png` |  |
| `archpw` |  | none today | Asked while extracting a locked archive; no /show |
| `zip` |  | none today | Asked when 7-Zip is missing; no /show |
| `mkfolder` |  | none today | No /show for Create folder |
| `limits` |  | none today | Shown when a download limit is reached; no /show |
| `generic` |  | none today | A notice with nothing of its own to open it |
| `shortcuts` | `shortcuts` | `shortcuts.png`, `shortcuts-dark.png` |  |
| `friend` | `tell-friend` | `friend.png`, `friend-dark.png` |  |
| `safari` |  | none today | macOS only: not on Windows |
