# Troubleshooting

[← Back to docs index](README.md)

## First, look at what happened

Every download keeps a log of its recent events: retries, redirects, the server's answers and errors.

- Hover over the download's **Status** in the list to see them.
- Or open its progress window (double-click an unfinished download). The connection table shows what each connection is doing, and **Retrying** with the reason when one fails.
- **File Properties** (**Alt+Enter**) shows **Last try** and its **Result**.

## A download says Error

1. Read the reason (hover over **Status**).
2. Select the download and click **Resume**. Many errors are a server or connection having a bad moment.
3. If the link has expired (common for links from sites you sign in to, or links with a time limit in them), get a new one: right-click → **Refresh download address**. See [Resume a broken download](guide-resume-broken-download.md).
4. If it says **the file changed on the server**, the file isn't the one MDM started with. Right-click → **Redownload** to start again from the beginning.
5. If it says **Google Drive won't give this file now**, Drive's download limit for that file was reached. See [Google Drive links](adding-downloads.md#google-drive-links).

## A download starts over instead of resuming

- Look at **Resume capability** in the progress window. **No** means the server can't send a file in parts, so MDM can't continue it later.
- Some servers report a new date for the same file every time. Tick **Options → Downloads → Ignore file modification time changes when resuming a download**.

## Downloads are slow

- Check that the Speed Limiter is off: **Downloads → Speed Limiter → Turn off**. Also look at **Downloads → Speed Limiter → Settings... → Limit all downloads together to**, which applies even with the limiter off.
- Check the download's own **Speed Limiter** tab in its progress window.
- Check **Options → Connection**. A low **Default max. conn. number**, or an exception for the server, means fewer connections. Rows marked **(auto)** are servers that refused more connections; MDM can't do more there.
- Check **Download limits** in the Scheduler. When the limit is reached, downloads wait with **Waiting: the download limit is reached (Scheduler)**.

## The browser doesn't send downloads to MDM

1. Was MDM installed with the setup, with **Set up browser integration** ticked? A portable copy can't take browser downloads. Run the setup again to add it.
2. Is the extension loaded and turned on? Look for **Mister Download Manager** on the browser's extensions page. In Firefox, install it from addons.mozilla.org: a temporary add-on is gone after a restart. See [Browser integration](browser-integration.md).
3. Is **Options → General → Use advanced browser integration** on, and your browser ticked under **Capture downloads from the following browsers**?
4. Is the file's type in **Options → File types**? Only listed types are taken automatically. **Download with Mister Download Manager** on the right-click menu takes any link.
5. Is the site in **Don't start downloading automatically from the following sites**?

When none of this helps, the browser keeps the download, so nothing is lost. The extension's toolbar button says why it couldn't reach MDM; see [The browser and MDM aren't connected](browser-help.md).

## The Scheduler didn't start the queue

- MDM must be running at that time. Closing the window is fine (it stays in the tray); **Tasks → Exit** is not.
- Did you click **Apply** after setting the time?
- **Daily** starts only on the ticked days. **Once at** starts only on that date.
- A computer that is asleep doesn't run anything until it wakes, unless the queue has **Wake the computer to run this queue** ticked (on Windows it needs **Allow wake timers** in the power plan; the Scheduler says when it's off). See [Wake the computer to run a queue](queues-and-scheduler.md#wake-the-computer-to-run-a-queue).

## A video won't download

See [If a video won't download](video.md#if-a-video-wont-download).

## A magnet link takes a long time

A magnet link has to get the torrent's file list from other peers first. That can take a minute or two, and longer for torrents with few peers. If nobody answers in about three minutes, MDM gives up; try again later, or use the `.torrent` file if the site offers one.

## MDM uses a lot of memory

Task Manager shows several figures, and they mean different things:

- **Memory** (on the Processes tab) is what MDM holds for itself alone, its *private working set*. With the window open and nothing downloading, that is about 70 MB.
- **Working set** (Details tab, add the column) adds memory shared with other programs. MDM draws its windows with the graphics card, so the graphics driver's memory counts here too, about 120 MB in all.
- **Commit size** is memory MDM has reserved, whether in use or not.

Closed to the notification area, MDM gives its memory back to Windows: **Memory** drops to under 10 MB within seconds and stays low while it waits. Downloads, queues and the scheduler go on as before.

## The text is too small or blurry

MDM keeps Internet Download Manager's compact sizes. On a large or high-resolution screen, choose **View → Interface size** (125 %, 150 %, 175 % or 200 %), or press **Ctrl +** and **Ctrl −** in any MDM window, as in a browser; **Ctrl 0** goes back to 100 %. Every window follows at once, and MDM starts at that size next time. It comes on top of Windows' own display scale (**Settings → System → Display → Scale**).

A window larger than the screen at that size (Options at 200 % on a 1366 × 768 screen) opens as large as the screen allows, and the rest scrolls: with the scroll bars, the mouse wheel, or **Tab** to the next control.

MDM tells Windows it draws for each monitor's own scale, so Windows never stretches its windows into blur, and it draws Windows' fonts grid-fitted to whole pixels. If the text still looks blurred, tell us your screen's resolution and Windows' display scale when you [report it](https://github.com/HUMBLEF0OL/grabbit/issues).

## The window freezes or draws wrongly

MDM draws its windows with Direct3D 12. A graphics driver can still misbehave: a window that holds still for seconds, flickers, or shows wrong colours or blocks. Try the other one: choose **View → Draw with → OpenGL** (or **Direct3D**), then quit MDM (**Tasks → Exit**) and start it again. Choosing the first one goes back the same way.

![View → Draw with open: OpenGL and Direct3D](images/help/draw-with.webp)
*View → Draw with: the other way of drawing.*

- If Direct3D can't start on your computer (no driver for it, some virtual machines), or its first window doesn't show within 10 seconds of its own (the time MDM spends loading your list and its icons doesn't count, however slow the computer is just after it starts), MDM draws with OpenGL, and **View → Draw with** says so and why. Choosing it again tries it again at the next start.
- If neither can start, MDM draws with a software renderer (Microsoft's), so the window still opens (Remote Desktop, a virtual machine without a graphics driver). It works, but keeps the processor busier while a window changes.
- When OpenGL fails on a card without Direct3D 12, MDM draws with Direct3D on Microsoft's software renderer (WARP) from the next start, and **View → Draw with** says why: OpenGL couldn't start, it is too old to draw MDM (older than 3.0, such as Intel's first HD Graphics, whose OpenGL 2.1 drew MDM's windows as garbage), its first window didn't show within 10 seconds, or MDM froze inside the graphics driver while OpenGL showed a frame (the older Intel HD Graphics and AMD Radeon cards' drivers did). Choosing **OpenGL** there tries it again.
- If the graphics card is reset while MDM runs (a driver update), MDM starts again by itself. Lost twice in 10 minutes, it goes back to OpenGL, and **View → Draw with** says why.
- Updating the graphics driver from the card maker's site (Intel, AMD or NVIDIA) fixes many of these problems.
- A freeze whose report shows the window thread inside the graphics driver as it shows a frame (`ig4icd64.dll` or `ig9icd64.dll` for Intel, `nvoglv64.dll` for NVIDIA, `atio6axx.dll` for AMD, under `opengl32.dll`) is the driver's OpenGL stuck, not MDM: choose **View → Draw with → Direct3D** (the default) and restart MDM, and update the graphics driver. If **Draw with** says Direct3D didn't start there, the line under it says why; MDM then remembers the freeze and draws with WARP from the next start.
- A freeze or crash report says which one was drawing, on which graphics card and driver, and why that one (chosen, the default, or the other one failing), on its `Drawing with:` line near the top: include it when you report a problem. A freeze report also says whether MDM was busy, waiting, or hardly ran at all (a computer too busy to run it); see [Crash reports](crash-reports.md).

## The setup or MDM won't start

- If Windows says **Windows protected your PC**, click **More info**, check that the file is `MisterDownloadManager-<version>-setup.exe` from the [Releases](https://github.com/HUMBLEF0OL/grabbit/releases) page, then click **Run anyway**.
- MDM runs only once. Starting it again brings the running copy's window to the front. If nothing appears, look for MDM's icon in the notification area.

## My antivirus or VirusTotal flags MDM {#antivirus}

A few antivirus programs flag new installers just because they have rarely seen them, and machine-learning engines do this the most. On VirusTotal, a handful of small engines flagging a new setup while the big ones stay clean is usually a false positive. We report these to the vendors when we see them.

To check that your copy is the real one:

1. Download MDM only from [the website](https://misterdownloadmanager.com) or [GitHub Releases](https://github.com/HUMBLEF0OL/grabbit/releases).
2. Get your file's SHA-256: run `certutil -hashfile "MisterDownloadManager-0.5.0-setup.exe" SHA256`.
3. Compare it with the same file's line in that release's `SHA256SUMS.txt`.

If they match, it is the file we published. Every release is built on GitHub from the [public source code](https://github.com/HUMBLEF0OL/grabbit), so anyone can read what went into it.

If your antivirus quarantined MDM, restore it and add an exception for its folder. If it flags something other than the setup or MDM's own programs, or a copy from anywhere else, don't run it, and please [tell us](https://github.com/HUMBLEF0OL/grabbit/issues).

## Where your data is

- Installed or the plain zip: `%APPDATA%\misterdownloadmanager`. The list and settings are in `downloads.json`.
- Portable: the `data` folder next to `misterdownloadmanager.exe`.
- Passwords (Sites Logins, the proxy, debrid accounts) are in Windows Credential Manager, not in `downloads.json`; see [Options → Sites Logins](options.md#sites-logins).

To back up your list, quit MDM (**Tasks → Exit**) and copy `downloads.json`.

## Updates

**Help → Check for updates...** tells you whether a newer version is out, and what is new in it. **Check for updates at every start** in that window lets MDM look by itself: a few seconds after it starts, and once a day while it stays open. When there's a new version, its window opens in front.

When there is one, **Update now** downloads it, checks it, puts it in place and restarts MDM; your list, settings and downloads stay as they are:

| You installed | What Update now does |
|---|---|
| The setup | Runs the new setup without its windows. An installation for all users asks for the administrator's permission. |
| The portable or plain zip | Replaces the programs in their folder (a portable copy's `data` folder is kept). |

MDM installs an update only when the release's list of checksums is signed by MDM's release key and the file matches it; anything else is refused, and **Download** opens the [Releases](https://github.com/HUMBLEF0OL/grabbit/releases) page instead. Updating this way starts with version 0.2.1: earlier versions open the Releases page.

**Help → About Mister Download Manager** shows the version you have.

![About Mister Download Manager: its version, licence and links](images/help/about-updates.webp)
*Help → About: the version you have.*

## Reporting a problem

Choose **Help → Report a problem**. It opens the bug report form on GitHub with your MDM version, your system and its version, how MDM was installed already filled in. Describe what happened, what you expected and the steps that make it happen. For a download, include the address if you can share it, and the reason shown under **Status**.

You can also [open the form yourself](https://github.com/HUMBLEF0OL/grabbit/issues/new?template=bug.yml); it asks for the same details.

## Next steps

- [Resume a broken download →](guide-resume-broken-download.md)
- [Browser integration →](browser-integration.md)
- [Back to the docs index →](README.md)
