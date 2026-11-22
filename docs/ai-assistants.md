# AI assistants (MCP)

[← Back to docs index](README.md)

Grabbit comes with an [MCP](https://modelcontextprotocol.io) server, `grabbit-mcp`, so an AI assistant that supports MCP (Claude Desktop, Claude Code, Cursor, VS Code and others) can work with the downloads in your running Grabbit. You ask in your own words:

- "What is Grabbit downloading right now, and how long is left?"
- "Download https://example.com/big.iso to D:\ISOs tonight, don't start it yet."
- "Pause everything that's downloading."
- "Remove the finished downloads from the list but keep the files."

The assistant starts the server itself; you only tell the assistant where it is, once. The server passes each request to the app through its [automation API](command-line.md#the-automation-api), on this computer only.

## What the assistant can do

| Tool | What it does |
|---|---|
| `list_downloads` | Lists your downloads, newest first: file, address, folder, status, size, how much is done, speed. It can list only one status (Queued, Downloading, Stopped, Complete, Error) or only names and addresses that contain some text. |
| `add_download` | Adds an http, https, ftp or ftps address and starts it, or only queues it. It can say which folder and file name to use. A magnet link opens **Add torrent** in Grabbit, for you to choose the files. |
| `pause_downloads` | Pauses running downloads. |
| `resume_downloads` | Starts or resumes stopped, queued or failed downloads. |
| `remove_downloads` | Removes downloads from the list and keeps their files, or deletes the files too when asked (a finished file goes to the Recycle Bin). A running download must be paused first. |
| `start_queue` | Starts the main download queue. |

The assistant can't change Grabbit's options, see your browser, or turn the automation API on by itself.

## Set it up

### 1. Turn on the automation API

Listing and acting on downloads needs Grabbit's automation API, which is off until you turn it on. With Grabbit running, run this once (it stays on):

```
grabbit-cli api on
```

`grabbit-cli api off` turns it off again. Without the API, the assistant can still add downloads, the same as Grabbit's [command line](command-line.md#the-apps-command-line).

### 2. Find the server

`grabbit-mcp` is next to the app:

| You installed | The server |
|---|---|
| Windows, for you only (the installer's default) | `%LOCALAPPDATA%\Programs\Grabbit\grabbit-mcp.exe` |
| Windows, for everyone | `C:\Program Files\Grabbit\grabbit-mcp.exe` |
| Windows, a zip or [portable copy](portable.md) | `grabbit-mcp.exe` in its folder |

A portable copy's server finds the portable copy's Grabbit by itself, from its own folder.

### 3. Tell your assistant

**Claude Desktop**: **Settings → Developer → Edit Config** opens `claude_desktop_config.json`. Add the server, then restart Claude Desktop:

```json
{
  "mcpServers": {
    "grabbit": {
      "command": "C:\\Users\\you\\AppData\\Local\\Programs\\Grabbit\\grabbit-mcp.exe"
    }
  }
}
```

**Claude Code**:

```
claude mcp add grabbit -- "C:\Users\you\AppData\Local\Programs\Grabbit\grabbit-mcp.exe"
```

**Cursor**: the same `mcpServers` entry as Claude Desktop's, in `~/.cursor/mcp.json` (or add it in Cursor's MCP settings).

**VS Code**: in `.vscode/mcp.json`, or **MCP: Add Server** in the command palette:

```json
{
  "servers": {
    "grabbit": {
      "type": "stdio",
      "command": "C:\\Users\\you\\AppData\\Local\\Programs\\Grabbit\\grabbit-mcp.exe"
    }
  }
}
```

Use the path from step 2 (in JSON, every `\` is written `\\`). Other assistants take the same thing: a command to start, no arguments.

## Privacy

Grabbit itself still sends nothing anywhere: the server talks only to the Grabbit on this computer, over the token-protected local channel. But what the assistant reads, it passes to its own AI service: when it lists your downloads, their file names, addresses and folders go to that service, under that service's privacy terms. Turn the automation API off, or remove the server from the assistant, when you don't want that.

## When it doesn't work

| The assistant says | What to do |
|---|---|
| Grabbit isn't running | Start Grabbit. The server can't start it for you. |
| The automation API is off | Run `grabbit-cli api on` with Grabbit running. |
| Grabbit didn't answer in time | A window in Grabbit may be waiting for you (a question, **Add torrent**); answer it and ask again. |
| Grabbit refused the request | Grabbit's channel file is from an earlier start; restart Grabbit. |
| The tools don't show up at all | Check the path in the assistant's settings, and restart the assistant. Started by hand, `grabbit-mcp --help` says what it is. |
