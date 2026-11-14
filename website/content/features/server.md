---
name: Server mode
title: Run MDM on a home server
headline: Let the home server do the downloading.
line: MDM without a window on a Windows PC that stays on, fed from your phone or browser.
lede: Run MDM without a window on a Windows computer that stays on, and send it downloads from your phone or your browser. The files land in its own folders, such as a network share.
description: Run Mister Download Manager without a window on a Windows home server, and send it downloads from your phone or your browser.
group: network
order: 240
since: 0.5
where: The command line: `misterdownloadmanager --headless`
help: server.md
---

## The same MDM, without a window

`misterdownloadmanager --headless` runs MDM with its list, categories, queues, Scheduler, speed
limits and torrents, but no window. Its folders and port come from flags or the environment. It
stops cleanly, saving its list, when it is stopped.

Give it a folder for its settings and one for the downloads, where each category gets its own
folder. Point the downloads at a network share and the files land straight on it.

## Control it from your phone or your browser

The remote control is on in this mode. The log prints the address to open and a pairing code
with a QR code: pair your phone as with the [remote control](/features/remote.html), and
`misterdownloadmanager-cli pair` makes a new code at any time.

In the browser extension, **Send downloads to → Another Mister Download Manager (a server)** sends the downloads it catches, and
**Download with Mister Download Manager** on a link, to the server instead of your computer. The
browser's cookies stay behind, so a download that needs you signed in to a site stays with your
computer.

## What a server doesn't do

Nothing opens a window, so a download that would ask a question, such as a duplicate, takes the choice set in the
settings, or isn't added. Don't publish the port on the internet: reach your home network through
a VPN instead.

Server mode is new, so please [tell us how it goes](https://github.com/HUMBLEF0OL/grabbit/issues).
