---
name: Notification area
title: Run in the notification area: start hidden, hide progress windows
headline: Out of the way. Still downloading.
line: Starts with the computer in the notification area, and hides progress windows there while they run.
lede: Grabbit can start with your computer and wait in the notification area, its window closed and its downloads running. A download's progress window can go there too, with one click.
description: Grabbit starts with your computer in the notification area, hides progress windows there in one click, and uses little memory there.
group: app
order: 310
since: 0.5
shot: images/help/options-general.webp
shot_alt: Options on its General page: start with the system, the clipboard, browser integration and the browsers Grabbit takes downloads from
where: Options → General
help: getting-started.md#the-tray-icon
---

## Starts with the computer, without getting in front

Tick **Launch Grabbit on startup** in **Options → General** and Grabbit starts
when you sign in, hidden, its icon in the notification area. Its downloads, queues, the
**Scheduler** and downloads from the browser all run from there. Click the icon, or start Grabbit
again, and the window opens as usual.

The option is off until you tick it. A [portable copy](portable.md) can't start with Windows, since that needs the registry, so
the option is greyed there.

## One click hides a download's window

A download's progress window has a **Hide to the notification area** button: a short dash with
an arrow down to the corner. It sits in the title bar, left of Minimize, and in the
title bar's menu (Alt+Space). The download
carries on.

To see it again, right-click the tray icon and choose it under **Downloads in progress**, which
lists every running download. Double-clicking the download in the list brings its window back
too.

## Minimize sends windows to the tray as well

With **Minimize to the notification area** on in **Options → General** (it is on by default),
Minimize hides the main window, and a progress window, in the notification area instead of
leaving them on the taskbar. Closing the main window does the same: Grabbit keeps running until you
choose **Tasks → Exit** or **Exit** in the tray menu.

The tray menu also has **Add new download...**, **Add batch download...**, the **Speed Limiter**, the
**Scheduler**, the main download queue and **Options**.

## Light while it waits

Hidden in the notification area, Grabbit hands the memory it isn't using back to the system.
Task Manager's **Memory** for Grabbit drops from about 50 MB to under 10 MB there, and Grabbit gives
memory back again every 10 minutes while it stays hidden.

## Where there is no tray

All of this needs the notification area to show Grabbit's icon. Without one (Explorer not running),
Grabbit starts minimized instead, closing the window quits Grabbit, and **Minimize to the notification
area** is greyed.
