# Run MDM on a server

Run Mister Download Manager without a window on a Windows computer that stays on (a home server,
a spare PC), and send it downloads from your phone or your browser: it downloads them into its
own folders, such as a network share.

It is the same MDM as on a desktop: its list, categories, queues, Scheduler, speed limits and
torrents. You control it with the [remote control's page](remote-control.md) (from any phone or
computer on your network), the browser extension pointed at it, or its command line.

> **New:** the server mode is new in this version. Please
> [tell us how it goes](https://github.com/HUMBLEF0OL/grabbit/issues).

## Start it

```powershell
misterdownloadmanager --headless --data C:\mdm-server --downloads D:\Downloads
```

| Flag | Or the variable | What it sets |
|---|---|---|
| `--data <folder>` | `MDM_DATA_DIR` | the list, settings, certificate and paired devices |
| `--downloads <folder>` | `MDM_DOWNLOADS` | where downloads go (each category's folder inside) |
| `--port <number>` | `MDM_REMOTE_PORT` | the remote control's port (8463) |
| `--listen <address>` | `MDM_REMOTE_LISTEN` | where it listens: every local network address unless given |
| `--hosts <names>` | `MDM_REMOTE_HOSTS` | names besides its addresses the page may be opened by |
| | `MDM_REMOTE_URL` | the page's address as you open it, shown with the pairing code |

The remote control is on in this mode (`MDM_REMOTE=off` turns it off). It stops cleanly, saving
its list, when it is stopped (Ctrl+C).

## Pair your phone

It prints the address to open and, while no device is paired, a pairing code with a QR code:

```text
The remote control listens at: https://192.168.1.10:8463/
Its certificate's SHA-256 fingerprint: DE:26:7F:…
No device is paired yet. Open the address (or scan this) and enter the code 1037 1606 …
```

Then pair as [Control MDM from your phone](remote-control.md#pair-your-phone) says. For another
code, at any time, with the same data folder:

```powershell
$env:MDM_DATA_DIR = "C:\mdm-server"; misterdownloadmanager-cli pair
```

To open the page by the computer's name (`https://homeserver.local:8463/`), give MDM that name:
`--hosts homeserver.local` (several: separated by commas). Other names are refused, so that no
web page can reach MDM through a name of its own.

The command line talks to it as to a desktop MDM, with the same data folder:
`misterdownloadmanager-cli status` (once `api on`), `pair`, and adding an address:
`misterdownloadmanager /d https://example.com/file.zip`.

## Send downloads from your browser

In the browser extension's menu, **Send downloads to → Another Mister Download Manager (a server)**: type the server's address,
open it once in that browser to accept its certificate, then enter a pairing code (`pair`
above). Downloads the extension catches, and **Download with Mister Download Manager** on a link,
then go to the server instead of this computer. The browser's cookies are not sent to the server:
a download that needs you to be signed in to a site stays with this computer.

## What a server doesn't do

- Nothing opens a window: a download that would ask a question (a duplicate, files already
  there) takes the choice set in the settings, or isn't added.
- Don't publish the port on the internet; reach your home network through a VPN instead.
