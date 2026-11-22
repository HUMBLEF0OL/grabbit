# Connections and speed

[← Back to docs index](README.md)

## The progress window

Each running download has its own progress window. It opens when the download starts (unless you turned that off in **Options → Downloads → Customize "Download progress" dialog → Edit...**); to open it again, double-click the download in the list.

![The progress window](images/progress.webp)

It shows the file's name (and, when it isn't simply downloading, its state: **Stopped**, **Waiting**, **Error** and so on), then:

- How far it is, large, with the time left beside it.
- A strip of the whole file. The file is split into parts that download side by side; each part
  has its own place in the strip, as wide as its share of the file, filled as far as it has come,
  with a small mark where it is being written now. With many small parts the strip is one
  plain fill. A torrent's strip shows its pieces: the brighter a place, the more of its pieces
  are in. A video's strip is one plain fill.
- How much has come of the whole (**2.38 of 5.80 GB**), and the speed.

**Show details** widens the window to the right (it moves left if it would run past the screen's edge); **Hide details** makes it narrow again. On the left:

- **Saved to** — the folder; its button shows the download there.
- **From** — the download's address; its button copies it.
- **Server** — where the server is, and how near. See [Where the server is](#where-the-server-is).
- **Can resume** — **Yes** if the server lets Grabbit continue the download later, **No** if it doesn't.
- **Limit** — a limit for this download only. See [For one download](#for-one-download).
- **When done** — what happens when this download finishes: **Show the Done window**, **Do nothing**, **Exit Grabbit**, **Shut down**, **Sleep** or **Hibernate**. With the last three, **Force apps to close** keeps other programs from holding it up.

A video shows **Format** and **Downloaded by** in place of **Server** and **Can resume**.

On the right, **Speed, last 60 s**: a graph of the last minute, with the limit as a dashed line,
so you can see whether the limit holds the download back. It starts empty when the window opens.
Under it, **Connections**: one row per connection, with its number, how much it has received and
its speed, or what it is doing (receiving ones first, then those retrying, connecting, and done).

The buttons:

- **Stop** stops the download and keeps what it has; **Resume** continues it. When the server can't resume (**Can resume: No**), **Stop** asks first, since the download would start over: **Keep downloading** or **Stop anyway**.
- A failed download offers **Retry**, or **Refresh download address...** when its link expired or was refused. When the site sent a web page instead of the file, **Open in browser** gets past it.
- **Cancel** stops the download and closes the window. The download stays in the list and can be resumed.
- Closing the window with its close box, or pressing **Esc**, keeps the download running.
- **Hide to the notification area** (a short dash with an arrow down to the corner; on Windows in
  the title bar, left of Minimize) puts the window in the notification area; **Downloads in
  progress** in the tray icon's menu brings it back. See [The tray icon](getting-started.md#the-tray-icon).

## Where the server is

**Download file info**, the progress window and **Properties** show where a download's server is: its country's flag and name, then the time a round trip to it takes, such as **about 24 ms (near)**.

- **Near** (green): under 60 ms, usually the same country or the next one.
- **Medium** (amber): 60 to 150 ms, usually the same continent.
- **Far** (red): over 150 ms, usually another continent. A far server is often slower, and more connections help it most.

Hover over the line for the server's address and how the time was measured.

How Grabbit knows:

- **The country** comes from a table of address ranges that comes with Grabbit ([IP to Country Lite by DB-IP](https://db-ip.com), CC BY 4.0), looked up on your computer. Grabbit never sends the address anywhere to ask where it is. The table is updated with each release; an address that has moved since may show its old country.
- **The time** is measured from the download's own connections, as each one opens: Grabbit sends nothing extra to the server. It is an estimate (the time to open a connection, divided by its round trips: one for http, two for https), so it says **about**. The quickest connection counts.
- **A server on your computer or your local network** shows as such, with no flag.
- **Through a proxy**, Grabbit only sees the proxy, so it says **through a proxy** (with the proxy's country when it knows it) instead of guessing the server's. A VPN that works at the system level can't be seen: the country is then the server's, but the time includes the VPN's detour.
- **A content delivery network** (Cloudflare, Amazon CloudFront, Fastly, Akamai and others) serves a file from a server near you, not from the site owner's. The tooltip says so: the place is the network's server, not where the site is.
- Torrents and galleries have no single server of Grabbit's own (peers, or the many a gallery's pictures come from), so their windows don't show one. A video in one file shows the server it comes from.

**Properties** keeps what the last run saw, so a finished download still shows where it came from.

## When a download finishes

**Download complete** shows the file's size, its address and where it was saved. Choose **Open** (its arrow has **Open with...**), **Open folder** or **Close**. A download saved as a folder, such as a torrent of several files, opens its main file, the one it is mostly made of, and **Open folder** shows that file in the folder; when there is no main file, **Open folder** opens the folder and **Open file** lists its largest files. If the file was moved or deleted since, the window says so and **Open folder** becomes the default. On Windows you can also drag the small box at the bottom right into Explorer, an email or a chat to drop the file there. As when you drag a file in Explorer, a folder on the same drive gets the file moved there and another drive gets a copy (hold **Shift** to move it, **Ctrl** to copy it). The download follows a moved file: the list and **Open** find it in its new place. The window shows above your other programs until you click it, without taking the keyboard from the one you're typing in.

Tick **Don't show this dialog again** to stop seeing it. It can be turned back on in **Options → Downloads → Show download complete dialog**.

## How Grabbit uses connections

Grabbit splits a file into parts and downloads them over several connections at once:

- It uses **8 connections** by default. You can set 1 to 32 in **Options → Connection → Default maximum connections**, or up to 128 after ticking **Allow more than 32 connections** there. More connections help only when the server sends each one at a limited speed and allows that many from one address; most servers allow far fewer.
- When a connection finishes its part, it takes over half of the largest part still left, so no connection sits idle while another one is still working.
- A connection that is far slower than the others is dropped and opened again.
- Some servers allow only a few connections. Grabbit notices this and stops asking that server for more. What it has learned shows in **Options → Connection** under **Exceptions**, marked **(auto)**.
- You can set a number for a server yourself: **Options → Connection → New**, then type the server's name and the number of connections.

Each connection writes straight into the file's place on disk, in a part file named `<name>.mdm.part` next to the final file. When every part is done, Grabbit checks the size and renames it. There is no temporary folder and no joining step at the end.

## Resuming

A download resumes where it stopped: after you stop it, after the connection drops, or after Windows or Grabbit crashes. Grabbit saves how far each part has got as it goes.

- A connection that fails is retried on its own. The progress window's table shows **Retrying** and the reason.
- To continue a stopped download, select it and click **Resume** on the toolbar (or **File → Resume**).
- Before continuing, Grabbit checks that the file on the server is still the same. If it changed, the download stops with **the file changed on the server**. Use **Redownload** to start again from the beginning.
- If **Can resume** (under **Show details**) is **No**, the server can't send a file in parts. Grabbit then downloads it over one connection, and a download that stops has to start over.

[Resume a broken download](guide-resume-broken-download.md) covers expired links too.

## Through your VPN only: the network adapter and its kill switch

**Options → Proxy / SOCKS → Network adapter** holds every connection Grabbit opens to one network
adapter, usually your VPN's: downloads, FTP, torrents, video pages and galleries, Grabbit's own
downloads and the update check. It works as a kill switch: when the adapter disconnects, or the
VPN is switched off, nothing goes out through your normal connection. Running downloads stop with
**… is not connected**, and start again by themselves when the adapter is back. The line under
the list shows the adapter's address, or that it isn't connected. Details in
[Options → Proxy / SOCKS](options.md#proxy--socks) and, for torrents, in
[Torrents and your security](torrent-security.md).

## The DNS

**Options → Connection → DNS** chooses who turns the servers' names into addresses: the system's
DNS (the default), or an encrypted one (DNS over HTTPS or TLS) such as Cloudflare, Google, Quad9,
AdGuard, NextDNS or your own. With a network adapter chosen, Grabbit asks it through the adapter, so
the names stay inside your VPN too. See [Options → Connection](options.md#connection).

## Speed Limiter

The Speed Limiter keeps Grabbit from using all of your connection.

### For every download

1. Choose **Downloads → Speed Limiter → Settings**.
2. Set **Maximum download speed for one file** in KB/s. It starts at 10, so set it before you turn the limiter on.
3. Click **Save**. (The settings open in the main window, as the **Speed Limiter** section of [Options](options.md), listed under **Tools**; **← Downloads** goes back to the list.)
4. Choose **Downloads → Speed Limiter → Turn on**. **Turn off** gives full speed again.

**Turn on the Speed Limiter when Grabbit starts** turns it on each time Grabbit starts.

A torrent counts as one file here: each torrent gets the limit of its own. So do a video and an image gallery ([Videos](video.md#speed-limits)). A download with a limit of its own (see [For one download](#for-one-download)) keeps that one.

### For all downloads together

In the same place, tick **Limit all downloads together to** and set the speed. This caps Grabbit as a whole, however many files are downloading, torrents included. When torrents and other downloads run at the same time, each side gets at least half of the speed, and more when the other side needs less: a torrent going slowly leaves the rest to the other downloads, and the other way round. It applies whenever it is ticked, whether the per-file limiter is on or off.

Tick **Only while I'm using the computer** to apply it only while you use the mouse or keyboard. After a minute without input, Grabbit goes back to full speed.

![Speed Limiter settings](images/speed-limiter.webp)

### For one download

In the download's progress window, click **Show details**:

1. In the **Limit** row, tick **Limit to**.
2. Type the speed in KB/s. The new limit applies once typed.
3. Tick **Remember for this file** to keep this limit for the file: it applies again when you resume it later, even after restarting Grabbit.

A video or a gallery takes a limit of its own the same way. For a torrent, the limit is that torrent's alone; the others keep theirs. A torrent can't be held under 16 KB/s: a lower limit gives it 16.

> Some servers drop the connection when the speed is very low. Avoid a tight limit for servers that can't resume (**Can resume: No**).

## Next steps

- [Keep some speed for everything else →](guide-keep-speed-for-everything-else.md)
- [Resume a broken download →](guide-resume-broken-download.md)
- [Connection settings in Options →](options.md)
