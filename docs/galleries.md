# Image galleries

Grabbit downloads the pictures of a gallery (a Wikimedia Commons category, a Flickr album, a DeviantArt gallery, an Imgur album and many more) with the help of [gallery-dl](https://codeberg.org/mikf/gallery-dl), a free program that reads several hundred sites. Like yt-dlp for videos, it's an optional helper: Grabbit installs it the first time you need it, or uses your own copy.

## Download a gallery

1. Click **Add** (or use the paste bar) and paste the gallery's address, for example `https://commons.wikimedia.org/wiki/Category:Featured_pictures_of_birds`.
2. Grabbit opens **Download a gallery** instead of downloading the page as a file. The first time, click **Install gallery-dl**.
3. gallery-dl checks the address on your computer, without going online. When the line reads **gallery-dl can download this gallery's files**, you're ready.
4. Check **Save to** (the General category's folder by default). The pictures go into a folder of their own there, named after the gallery.
5. Optionally, type a number in **Files at most** to download only the first ones the site lists. On most sites those are the newest. Leave it empty for all of them.
6. Click **Download now**, or **Later** to add it to the list without starting.

The gallery gets one row in the list. Its size grows as the pictures arrive, and its log says how many were saved. Each picture is marked as downloaded from the internet, as any download is.

## How the pictures come down

gallery-dl lists the gallery's pictures: where each one comes from and the name it gets in the folder. Grabbit's own downloader then brings them in, 4 at a time (fewer when the site answers that it's busy, as Wikimedia's does), under your speed limits (a change applies at once); the progress window's **Show details** shows their speed over the last minute. gallery-dl then runs as before: it finds those pictures already there and fetches only what Grabbit couldn't have, such as a picture from a site that wants a sign-in or headers of its own. The log says how many each saved.

A gallery-dl older than 1.32 (a Linux distribution's, often) can't list the pictures this way: it downloads them all itself, one a second, as before.

## Get the new pictures later

Start a finished gallery again (**Resume**) to fetch only the pictures that aren't in its folder yet. The ones already there are skipped.

## Stop, delete

- **Stop** ends Grabbit's downloads of the pictures, and gallery-dl with everything it started. The pictures saved so far stay.
- **Delete** takes the gallery off the list. Its folder and pictures stay, even if you tick the box to delete files: delete the folder yourself if you want it gone.

## Which addresses open the gallery window

**Add** and the paste bar send these to the gallery window:
- Wikimedia Commons categories;
- Flickr photographers and albums;
- DeviantArt and ArtStation galleries;
- Imgur albums;
- Pixiv artists and works;
- Pinterest boards and pins;
- Danbooru and Gelbooru searches;
- Tumblr blogs;
- Bluesky profiles.

A picture's own address (one ending in `.jpg` or `.png`) downloads as a file, as before.

## gallery-dl itself

- **Where it comes from:** Grabbit downloads gallery-dl from its releases on Codeberg. It accepts it only when the release's checksum list is signed by gallery-dl's author's key, pinned in Grabbit, and the program matches that list. It keeps the previous version and never goes back to an older one.
- **Your own copy:** **Helpers...** (in the gallery window or the video window) lists yt-dlp, ffmpeg and gallery-dl, and **Use another program...** takes your own copy of each instead of Grabbit's.
- **How it asks:** gallery-dl and Grabbit's downloader ask sites as Grabbit; Grabbit brings 4 pictures at a time, and gallery-dl, for what's left to it, one a second at most. Wikimedia's servers refuse tools that ask much faster or without a name. gallery-dl uses no configuration file of yours.
- **Network adapter:** while one is chosen (**Options → Proxy / SOCKS → [Network adapter](options.md#proxy--socks)**), galleries go through it too, as video pages do: Grabbit gives gallery-dl a proxy of its own on this computer, opens every connection through the adapter and looks the names up with the DNS you chose. When the adapter disconnects, the gallery stops with the other downloads; nothing goes out through your normal connection.
- **An older gallery-dl of your own:** chosen in **Helpers...**, it works from 1.20 on: Grabbit passes only the options that version knows. One older than that is refused, saying which version Grabbit needs.

## On the command line

- `install-gallery-dl` installs or updates gallery-dl, verified in the same way.
- `gallery-get URL [-o folder] [--limit N]` downloads a gallery into a folder named after it. See [Command line and automation](command-line.md).
