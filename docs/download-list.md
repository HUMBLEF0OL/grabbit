# The download list

[← Back to docs index](README.md)

Every download, finished or not, has a row in the list. The categories on the left choose which rows you see.

The strip under the list says how many downloads it shows and how many are selected, and while
downloads run, how many and how fast in all (the tray icon's tip says the same). When the list
is empty it says why: nothing in this category or queue, or the column filters hide everything
(with **Clear all filters**).

Grabbit opens where you left it: the window's place and size, the Categories pane, its open
branches and the category or queue you were looking at.

## Columns

![The download list with its columns: Source, File Name, Q, Size and Status, and more to the right](images/help/list-columns.webp)
*Click a column title to sort; its funnel filters.*

| Column | What it shows |
|---|---|
| **Source** | **Web** (a web server) or **Torrent**. |
| **File Name** | The file's name, after its type's icon: Windows Explorer's, or Grabbit's where the system has none. |
| **Q** | A queue icon when the download is in a queue. |
| **Size** | The file's size, when the server told it. |
| **Status** | How far it is, or **Complete**, **Error** and so on. |
| **Time left** | While it downloads. |
| **Transfer rate** | The current speed. |
| **Upload speed** | A torrent: what it sends to peers, while it runs. |
| **Peers** | A torrent's connected peers while it runs, seeders and leechers alike. |
| **Share ratio** | A torrent: what it has sent to peers against what it took. |
| **Last Try Date** | When it last ran. |
| **Added** | When you added it. |
| **Description** | Your note from **Download File Info** or **Properties**. |

- **Sort:** click a column title. Click again to reverse the order, and a third time to go back to the order of addition. **View → Arrange files** does the same from the menu.
- **Move a column:** drag its title sideways.
- **Resize a column:** drag the line between two titles; double-click it to fit the column to
  its contents.
- **Choose columns:** right-click any column title and tick the ones you want. **Reset columns**
  in the same menu puts them back as they first were.

Grabbit remembers the columns, their widths and the sort order.

## Filters

Each column title has a faint funnel. Click it to filter the list by that column:

- **Source**, **Q** and **Status** offer a few values to tick, such as only **Error** under **Status**, or only **Not in a queue** under **Q**.
- The other columns take text: type a word, and only rows whose cell contains it stay.

A filtered column's funnel turns blue. To remove every filter, right-click a column title and choose **Clear all filters**. Filters last until you close Grabbit.

## Categories

The pane on the left has four branches:

![The Categories pane: All Downloads with a category per file type, Unfinished, Finished and Queues](images/help/list-categories.webp)
*All Downloads, Unfinished, Finished and Queues, each by category.*

- **All Downloads** — everything, with a node per category below it.
- **Unfinished** — downloads that aren't complete yet, by category.
- **Finished** — complete downloads, by category.
- **Queues** — each queue; select one to see only its downloads.

Each download falls into a category by its file type: **General**, **Compressed**, **Documents**, **Music**, **Programs** or **Video**. Torrents have their own **Torrents** category. Each category has its own folder, by default inside your Downloads folder (`Downloads\Compressed`, `Downloads\Video` and so on; General uses `Downloads` itself).

Right-click a category for:

- **Browse** — open its folder in Explorer.
- **Properties** — change its name, its file types and its folder. General and Torrents only change their folder. A renamed category keeps its folder; a built-in one keeps its name until you rename it.
- **Add category** — make your own, for example **Books** for `EPUB MOBI`. Give it a name, the file types (separated by spaces, `*` allowed) and a folder.
- **Delete** — remove a category, your own or a built-in one (not General or Torrents). Grabbit asks what happens to its downloads: **Move its downloads to General**, or **Remove its downloads from the list** (a download that is running moves to General instead). Their files stay where they are either way. From then on, files of its types go to another category that has them, or to General.

![A category's properties: its name, file types and folder](images/help/category-properties.webp)
*Right-click a category → Properties: its name, file types and folder.*

To bring the built-in categories back as they came, with their names, file types and folders, use **Options → Save to → Restore the built-in categories**. Your own categories, General and Torrents stay as they are.

The top branches show how many downloads they hold. **View → Hide categories** gives the list
the whole window.

## Dropping links and files

Drag a link from a browser (or a page's text with links in it) onto the list to add it: one
link opens **Download File Info**, several open **Link grabber** to choose which to add. Drop a
`.torrent` file to add the torrent, or a list of addresses (`.txt`, `.ef2`) to import it. The
list is outlined while something you can drop is over it.

Right-click the list's blank space for **Add new download**, **Select all**, **Arrange files**,
**Clear all filters** and **Delete All Completed**.

## Opening a download

- **Double-click** or press **Enter** on an unfinished download to see its progress window.
- On a finished download, double-click shows its **Properties**. To open the file instead, right-click any download and choose **On Double click → Open**.
- **Alt+Enter** always opens **Properties**.

The right-click menu has everything you can do with a download: **Open**, **Open with...**, **Open folder**, **Move/Rename**, **Redownload**, **Resume Download**, **Stop Download**, **Copy address** (**Ctrl+C**; a torrent has **Copy magnet link** instead), **Refresh download address**, **Verify checksum...**, **Remove**, **Add to queue** and **Properties**. **Open folder** is there once the download is finished, or for a torrent while it downloads.

## Properties

**File Properties** shows a download's type, status, size and folder, its address, your description, the page it came from (**Referer**), where its server is ([country and round trip](connections-and-speed.md#where-the-server-is), as its last run saw them) and a login for the site. **Last try** and **Result** tell you how its last attempt went.

- **Move** — move a finished file to another folder, or choose another folder for a download that hasn't started.
- **Address** — for an unfinished download, you can paste a new address here. See [Resume a broken download](guide-resume-broken-download.md).
- **Open** — open the file.

![File Properties: the file's name, address, folder, size, status and description](images/help/properties.webp)
*File Properties: change the address or the folder of a download.*

**Move/Rename** (**Ctrl+M**) moves or renames the file of a finished download, or of one that hasn't started yet. With several downloads selected it asks for a folder and moves them all there, each under its own name.

## Details pane

**View → Details pane**, or **Details** at the end of the toolbar, shows a pane beside the list with the selected download. The same again hides it. It is off at first.

The pane shows the download's icon and name, how far it is, and 8 cells that fill as each eighth of the file comes in. While it runs, and after it stopped in this session, the cells say how many connections it has (a video or a torrent shows none). When its size is known, they also say how much of it has come, for example 57.96 MB of 92.00 MB. Below them come **Status**, **Transfer rate** and **Time left** (while it runs), **Resume capability**, **Save to**, **Address** (never with its password), **Queue** and **Last Try Date**, each when it has a value. The figures change as the download runs.

Its buttons are **Resume** or **Stop**, **Open folder** (greyed until the download is finished, as in the right-click menu; a torrent's folder is there while it downloads) and **Show progress window** (greyed for a finished download). A download that failed because its link expired, or because the server refused it (403) or says it is gone (410), also gets **Refresh download address...** above them: it opens the page the link came from and **Properties**, where you paste the new address.

With nothing selected the pane says **Select a download to see its details**; with several, how many they are and their total size.

Drag the pane's edge by the list to make it wider or narrower. It is never wider than half the window, keeps the list at least 200 points wide when the window has room, and Grabbit remembers the width. The list keeps all its columns and scrolls sideways when they don't fit.

## Verify checksum

Many sites publish a checksum next to the download. Right-click the download, choose **Verify checksum...** and paste it. Grabbit reads MD5, SHA-1, SHA-256, SHA-512 and CRC32. A finished file is checked at once; an unfinished one is checked when its download ends. Grabbit then says **The file matches** or **The file does NOT match**.

## Selecting several downloads

- **Ctrl+click** adds or removes one download.
- **Shift+click** selects every download up to the one you click.
- **Drag on blank space** to draw a box around downloads.
- **Ctrl+A** selects all.
- **Shift+Up / Shift+Down** makes the selection longer or shorter.

**Resume**, **Stop**, **Remove** and the queue commands then act on all the selected downloads. The other commands need one download. The full list of keys is in [Keyboard shortcuts](keyboard-shortcuts.md).

## Finding a download

Type in the **Find** field in the title bar (**Ctrl+F** takes you there): the list keeps only the downloads whose name has what you typed, and the status bar says how many it shows. **Enter** or **F3** selects the next match, **Shift+F3** the one before; **Esc** clears the field. When nothing matches, the list says so and offers **Clear search**. Its funnel searches descriptions and addresses too, and shows only some statuses or sources. Step by step in [Find a download in a long list](guide-find-in-long-list.md).

Typing the start of a name while the list has the focus jumps to the first download whose name starts with it.

![The Find field in the title bar, the list narrowed to what it matches](images/help/find.webp)
*Type in Find (Ctrl+F): the list keeps the matches.*

## Deleting downloads

Select one or more downloads and press **Delete**, click **Delete** on the toolbar, or choose **File → Remove**. Grabbit asks first:

- **Also move the file to the Recycle Bin** — unticked every time the question opens. Tick it to remove the files too: a finished download's file goes to the Recycle Bin, where you can restore it from. For an unfinished download (the box then says **Also delete the file from disk**), this deletes the part downloaded so far; with both kinds selected, the box says **Also delete the files (finished ones go to the Recycle Bin)**. When finished downloads are among the ones you delete, the question says how many finished files go to the Recycle Bin. A torrent's files are deleted, not moved, and the question says how many finished torrents' files it deletes. Leave it unticked to take the downloads off the list and keep the files.

  Windows decides what the Recycle Bin takes: on a network or removable drive without one, or for a file too big for it, Windows may delete the file outright. A file Windows won't move (open in another program, or with a path longer than 260 characters) stays where it is, and Grabbit names it.
- **Don't show this dialog again** — Grabbit stops asking: **Delete** then only takes downloads off the list and keeps their files. To be asked again, tick **Options → General → Confirm before deleting downloads**.

Click **Yes** to delete, or **No** to keep everything.

**Shift+Delete** (or **Remove with files**) opens the same question with the box already ticked, for that time only. It always asks, even after **Don't show this dialog again**: Grabbit never removes a file from the disk without asking. The box is never remembered: the next **Delete** starts unticked again.

After deleting, the next download in the list is selected, so you can press **Delete** again.

A download that is running isn't deleted. Stop it first. When running downloads are among the ones you select, the others are deleted and Grabbit says how many running ones it kept: in the question, or in a message when the question is turned off.

**Delete Completed** on the toolbar (or **Downloads → Delete All Completed**) removes every finished download from the list, with the same question. Their files stay unless you tick **Also move the files to the Recycle Bin** in that question.

![Confirm deletion of downloads, with Also move the file to the Recycle Bin](images/help/delete-confirm.webp)
*Tick Also move the file to the Recycle Bin only if you want the file gone too.*

## Next steps

- [Watch a download's connections →](connections-and-speed.md)
- [Put downloads in a queue →](queues-and-scheduler.md)
- [Find a download in a long list →](guide-find-in-long-list.md)
