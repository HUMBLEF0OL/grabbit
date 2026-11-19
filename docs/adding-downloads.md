# Adding downloads

[← Back to docs index](README.md)

There are several ways to give Grabbit something to download. Most of them end in the same window, **Download File Info**, where you choose where the file goes.

## Add

**Add** on the toolbar takes anything Grabbit downloads: a file's address, a magnet link, a `.torrent` file or its address, a video's page, a gallery's page, or an address with a `*` for a batch download.

1. Copy the address (in a browser: right-click the link → **Copy link**). Several at once are fine.
2. Click **Add** on the toolbar, choose **Tasks → Add new download**, or press **Ctrl+N**.
3. The **Add** window opens with the clipboard's addresses already in its box, one per line. Type or paste more, or drop links or a `.torrent` file on the window. **Open a .torrent file...** picks one from the disk.
4. Under the box, **Found** lists each line with what it is (**Link**, **Magnet**, **Torrent file**, **Video**, **Gallery**, **Batch**) and what comes next. A line that isn't an address says why and is left out.
5. If the server needs a user name and password, tick **Use authorization** and type them.
6. Click **Add** (it says how many), or press **Enter**. **Shift+Enter** starts a new line in the box; **Esc** closes the window without adding.

![The Add window: its box, what was found, the drop area, Use authorization and Add](images/help/add-url.webp)
*The numbers are the steps above: 3 the addresses, 4 what was found, 5 a login if the server needs one, 6 Add.*

Each item then goes where it always has: a file's address to **Download File Info**, a magnet link or a `.torrent` to **Add torrent** (to choose its files), a video's page to the video window (to choose its quality), a gallery's page to the gallery window, and a `*` address to **Add batch download**. Several of one kind open one after another.

**Ask where to save each one** (ticked at first; Grabbit remembers how you leave it) shows **Download File Info** for every file's address. Untick it to add the files at once to their category's folder and start them. **Later** adds the files without starting them: they wait in a queue, as **Download Later** does. Torrents, videos and galleries always open their own window, as they need your choice.

Grabbit takes `http://`, `https://`, `ftp://`, `ftps://`, `ftpes://` and `magnet:` addresses, and `.torrent` files on this computer.

The arrow beside **Add** opens its menu: **Add links or a magnet...** (**Ctrl+N**), **Open a .torrent file...** (**Ctrl+O**), **Add from the clipboard** (**Ctrl+V**) and **Batch download...**.

**Dropping on the main window.** Links, text with links, magnet links and `.torrent` files dropped anywhere on the main window open the **Add** window with them, so you see what each is before anything is added. A list file (`.txt`, `.ef2`, `.htm`, `.csv`) still opens **Import**.

See [Video downloads](video.md), [Torrents](torrents.md) and [Image galleries](galleries.md) for those windows.

## Paste bar

The paste bar is the address box under the toolbar. It is there unless you turned it off: **View → Paste bar** shows or hides it.

1. Paste an address, a magnet link or a video's page into the box.
2. A chip beside it says what Grabbit found (**Link · example.org**, **Magnet**, **Torrent file**, **Video · youtube.com**, **3 links**, or **Not an address**), and the button says what it will do: **Download**, **Add torrent**, **Choose quality**, or **Add 3...**.
3. Press **Enter** or click the button. Nothing is added before that.

A file's address opens **Download File Info**, a magnet link or a `.torrent` opens **Add torrent**, a video's page the video window and a gallery's page the gallery window. Several links open the **Add** window, listed. Text that isn't an address greys the button; **Enter** then shows **Invalid address** and leaves the text in the box for you to fix.

The paste bar keeps a list of the last 20 addresses added. The arrow at the end of the box (or **Alt+Down** or **F4**) opens it, and **Up** and **Down** in the box step through the same addresses.

**Ctrl+V** while nothing else has the keyboard puts the clipboard's address in the paste bar. It adds nothing until you press **Enter**. With the paste bar hidden, or the clipboard holding several links, **Ctrl+V** opens the **Add** window with them instead. **Esc** leaves the box, and **Tab** moves between the paste bar, the categories and the download list.

## FTP and FTPS

FTP downloads work like web downloads: several connections at once, pause and resume, and the
same progress window.

- `ftp://` — plain FTP.
- `ftpes://` — FTP secured with TLS from the start of the session (explicit FTPS, `AUTH TLS`, usually port 21). Use it when the server says it supports TLS.
- `ftps://` — implicit FTPS, port 990 unless the address gives another.

The server's certificate is checked against Windows' trusted certificates, as for `https://`.

**Logging in.** Write the login in the address (`ftp://user:password@server/folder/file.iso`),
tick **Use authorization**, or save it in **Options → Sites Logins**. Without one, Grabbit logs in as
`anonymous`. The path is relative to the folder the login starts in; to start from the server's
root, write `%2F` after the server (`ftp://server/%2Fpub/file.iso`).

**Resuming.** Grabbit keeps the file's size and modification time. If either has changed when you
resume, it says the file changed on the server instead of mixing two versions
(**Options → Downloads → Ignore file modification time changes** skips the time check). The
password is not written to the resume file next to the download: Grabbit gives it again when it
resumes.

**Connections.** Passive mode is used by default. **Options → Proxy / Socks → Use FTP in PASV
mode** switches it off for servers that only take active connections. FTP connects to the
server directly, not through the HTTP proxy.

Browsers no longer download `ftp://` links themselves. Right-click one and choose **Download with
Grabbit**, or copy it and use **Add**.

## Google Drive links

Paste a Google Drive link as it is, the one **Share → Copy link** gives:

- `https://drive.google.com/file/d/…/view?usp=sharing`, with or without `/view`;
- `https://drive.google.com/uc?id=…&export=download` and `https://drive.google.com/open?id=…`;
- `https://docs.google.com/document/d/…/edit` (and Sheets, Slides, Drawings).

Grabbit downloads the file itself, as it was uploaded (a video comes in its own format, not the
player's copy), with its real name and size, over several connections, and it can resume. For a
large file, Drive shows a page that says it **can't scan this file for viruses**; Grabbit goes past
it as **Download anyway** does.

**Google Docs, Sheets and Slides** come as Word (`.docx`), Excel (`.xlsx`) and PowerPoint
(`.pptx`) files, and Drawings as `.png`. These files are made when they are asked for, so their
size is unknown until they arrive and they can't be resumed: a cut-off download starts again.

**Files that aren't shared with everyone.** Right-click the link in the browser and choose
**Download with Grabbit**, or use Drive's own **Download** button: the extension
passes your browser's Google sign-in with the download. Grabbit never asks for or keeps your
password. Without a sign-in, Grabbit says the file **needs you to sign in**.

**When Drive won't give a file.** Drive limits how often a file can be downloaded: after many
downloads in a day it answers "too many users have viewed or downloaded this file recently".
Grabbit shows Drive's own words and writes nothing. Try again later (it can take up to a day), or
sign in to Google in your browser and download from there, which often gets past the limit. Grabbit
doesn't try to get around it. A file whose owner turned off downloading, or that your account
can't open, gets **no access**.

**The video button over a Drive video** downloads the original file, not one of the player's
smaller copies.

**Folders** aren't downloaded yet: a folder's link gets **this is a Google Drive folder, not a
file**. Open the folder in the browser and download its files from there.

## Download File Info

Grabbit asks the server for the file's name and size, and shows them in **Download File Info**:

- **URL** — the address.
- **Category** — chosen from the file's type (Compressed, Documents, Music, Programs, Video or General). Each category has its own folder. **+** beside it makes a new category.
- **Save As** — the folder and file name. You can type another, pick one of the last 20 folders you saved to from the arrow at its end, or choose a folder with **...**.
- **Remember this path for "…" category** — tick it to make the folder you typed the category's folder from now on.
- **Description** — a note of your own. It shows in the list's **Description** column and Find can search it.
- **Server** — where the server is: its country's flag and name, and how long a round trip to it takes. See [Where the server is](connections-and-speed.md#where-the-server-is). It stays blank when the browser handed the download over with its name and size, since Grabbit didn't need to ask the server.

![Download File Info for a ZIP file: its address, the Compressed category, the folder it goes to, its size, and the Download Later, Start Download and Cancel buttons](images/help/download-file-info.webp)
*1 the category, 2 where the file goes, 3 start now, 4 or put it in a queue.*

Then choose:

- **Start Download** (or **Enter**) — download now. The progress window opens.
- **Download Later** — add it to a queue without starting. See below.
- **Cancel** (or **Esc**) — add nothing.

Nothing is downloaded until you choose **Start Download** or **Download Later**. To have Grabbit start while this window is still open, so no time is lost while you decide, turn on **Options → Downloads → Start downloading immediately while displaying "Download File Info" dialog**; **Cancel** then drops what it fetched.

If the folder doesn't exist yet, Grabbit asks **Do you want to create it?** before it starts.

## Download Later

**Download Later** puts the download in a queue instead of starting it. By default Grabbit asks which one: pick the queue under **Add to queue**, and tick **Start the queue now** if you want it to run straight away. **Cancel** leaves the download in the list, in no queue.

To skip that question and always use the **Main download queue**, turn off **Options → Downloads → Show queue selection panel on pressing "Download Later" button**.

More about queues in [Queues and the Scheduler](queues-and-scheduler.md).

## From the clipboard

With **Options → General → Offer to download links copied to the clipboard** ticked (it is, unless you turned it off), when you copy a link to a file type in **Options → File types** (ZIP, EXE, MP4, PDF and many more), **Download File Info** opens with it filled in, ready for **Start Download**. Links to other pages are left alone. With **Options → Downloads → Show start download dialog** off, the download starts at once instead. Only links copied while Grabbit runs are taken, never one that was already there.

## Drag and drop

1. Choose **Tasks → Show drop target**. A small box with the Grabbit icon appears and stays on top of other windows.
2. Drag a link from your browser onto it. Grabbit adds it as if you had used **Add**.

You can also drop magnet links and `.torrent` files on it. Drag the box to move it, and double-click it to bring the main window back. Right-click it for **Show Grabbit**, **Add new download** and **Hide drop target**.

## Batch download: many numbered files

For files whose names differ only by a number or a letter, such as `photo001.jpg` to `photo150.jpg`:

1. Choose **Tasks → Add batch download**. (**Tasks → Add batch download from clipboard** fills in a copied address.)
2. In **Address**, write the address with one asterisk where the number or letter changes: `https://example.com/photo*.jpg`.
3. Under **Replace asterisk to**, choose **Numbers** or **Letters**, then **From** and **To**.
4. For numbers, set **Wildcard size** to the number of digits. With 3, the files are `photo001.jpg`, `photo002.jpg` and so on.
5. If the server needs a login, tick **Use authorization**.
6. Check **First file**, **Second file** and **Last file**, then click **OK**.

The downloads go to a queue. As with Download Later, Grabbit asks which queue unless you turn off **Options → Downloads → Show queue selection panel on closing batch download dialogs**.

![Batch download: an address with an asterisk, what replaces it, and the first, second and last file](images/help/batch.webp)
*One asterisk in the address, then numbers or letters from–to.*

## A list of links

To add every link in a text file:

1. Choose **Tasks → Import → From text file**.
2. Pick the file. **Link grabber** lists every web address Grabbit found in it, in packages. While Grabbit reads a long file, the strip under the list of downloads says **Reading** and the file's name; you can go on using Grabbit meanwhile. Import reads files of up to 64 MB, far more than any list of links holds: a larger file (a video, an archive, a backup picked by mistake) is refused with a message.
3. Untick the addresses you don't want (**Check all** and **Uncheck all** help), then click **Add to queue**.

![Link grabber listing the links of a page in packages, with their names, sizes and whether they are online](images/help/link-grabber.webp)
*Link grabber: untick what you don't want, then Add to queue.*

The downloads join the **Main download queue**, ready to start (**Start** starts it too). **Tasks → Import → From IDM export file** reads an `.ef2` file, with each download's referrer and cookies.

**Tasks → Link grabber...** opens the same window empty: paste text with links in it (**Ctrl+V**). See [Link grabber and archives](link-grabber.md) for the packages, the archive password and extracting archives when they are downloaded.

To save your own list, use **Tasks → Export → To text file** (one address per line) or **To IDM export file**.

## When the address is already in the list

By default Grabbit asks what to do when you add an address it already has. You can choose a fixed answer in **Options → Downloads → If a duplicate download link is added**:

- **Show a dialog and ask what to do.**
- **Add the duplicate with a numbered file name**
- **Add the duplicate and overwrite the existing file**
- **If existing file is complete, show download complete dialog; otherwise resume it.**

## Next steps

- [Send browser downloads to Grabbit →](browser-integration.md)
- [Watch a download in the progress window →](connections-and-speed.md)
- [Queue downloads for later →](queues-and-scheduler.md)
