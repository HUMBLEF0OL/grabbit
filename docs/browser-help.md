# The browser and Grabbit aren't connected

[← Back to docs index](README.md)

When the browser can't hand a download to Grabbit, it keeps the download itself, so nothing is lost. The extension's toolbar button then shows a red **!**. Click it to see what's wrong, then **How to fix it** brings you to the matching section below.

To check Grabbit's side, open **Options → General** in Grabbit and click **Test browser integration...**. It checks that browser integration is on, that this is the installed copy, that the browsers can find Grabbit, and which browsers' extension has connected; **How to fix it** opens the section for the first that fails.

## Grabbit isn't installed, or browser integration isn't set up {#not-installed}

The browser can't find the small helper program it talks to. It is registered when Grabbit is installed.

1. Install Grabbit with the setup from [the download page](https://humblef0ol.github.io/grabbit/download.html), and keep **Set up browser integration** ticked.
2. Already installed? Open Grabbit, then **Options → General → Test browser integration**, and click **Set it up** if it offers to.
3. The portable copy can't be connected: it writes nothing outside its own folder. Use the installed copy for browser integration. See [Portable mode](portable.md).
4. Restart the browser after setting it up.
5. **Zen on Windows:** Zen 1.21.15b and later have a bug of their own that keeps add-ons from reaching their helper program ([zen-browser/desktop#15432](https://github.com/zen-browser/desktop/issues/15432)), with any add-on that uses one. Until Zen fixes it, Zen 1.21.14b works.

## Grabbit didn't start in time {#not-running}

When Grabbit isn't running, the browser starts it for each download and waits for it. It waited about 20 seconds this time and gave up.

- Start Grabbit yourself and try the download again.
- A first start, a slow disk, or an antivirus scanning Grabbit can take that long. Next time it is usually quick.
- With a VPN or proxy program, allow connections to `127.0.0.1` (your own computer): the browser's helper reaches Grabbit through one.
- If Grabbit never opens, see [The setup or Grabbit won't start](troubleshooting.md).

## The connection to Grabbit failed {#host-failed}

The browser started Grabbit's helper, but it stopped before answering. Security software is the usual cause.

- Allow `grabbit-nmhost.exe` (in Grabbit's folder) in your antivirus or anti-malware program.
- Reinstall Grabbit to restore its files.

## Grabbit doesn't accept this copy of the extension {#forbidden}

Grabbit's helper serves only its own extension. The browser says this copy isn't one it knows.

- **An extension from the Chrome Web Store or addons.mozilla.org?** Grabbit isn't in the stores yet: the **Mister Download Manager** extension there belongs to another project. Remove it and load the copy in Grabbit's folder instead (`extension\chromium`, see
  [the tutorial](guide-catch-browser-downloads.md#chromium)).
- **Two copies installed?** Open the browser's extensions page (`chrome://extensions`, `edge://extensions`, `brave://extensions`) and keep only one **Grabbit**.
- **A copy from elsewhere?** Remove it and add the extension as Grabbit's **Add Grabbit to your browser** window shows (see [Browser integration](browser-integration.md)).
- **An older Grabbit?** Update Grabbit: a newer extension may have an ID an old helper doesn't know.

## The extension is newer than Grabbit {#update-app}

Downloads still work, but some of the extension's features need a newer Grabbit. Update it with **Help → Check for updates...** in Grabbit.

If Grabbit says it's up to date, there's nothing for you to do. The extension updates by itself through its store and can get ahead of the app for a while. Everything keeps working, and the message goes away with the next version of Grabbit.

## A download's details were too large {#too-large}

A download carries its site's cookies (and a form's data) to Grabbit. A few sites set so many that together they exceed what Grabbit accepts from the browser (1 MB). The browser kept this one. Signing out and back in to the site usually trims them.

## Downloads stay with the browser although Grabbit is connected {#options}

The extension is connected, but Grabbit's own settings leave this download to the browser. Check, in Grabbit:

1. **Options → General → Use advanced browser integration** is on, and your browser is ticked under **Capture downloads from the following browsers**.
2. The file's type is listed in **Options → File types**.
3. The site isn't in **Don't start downloading automatically from the following sites**, nor the address in **Don't start downloading automatically from the following addresses**.
4. The file isn't smaller than **Don't take files smaller than** in **Options → File types**.

Also check the extension's toolbar button: with **Take downloads from this browser** off, everything stays with the browser.

## The browser opens PDFs and music instead of sending them {#opened}

Grabbit also takes the file types the browser would show or play itself (a PDF, an MP3), when they're listed in **Options → File types** and **Also take files the browser would open** is ticked in **Options → General → Keys...**. This needs a Chromium browser (Chrome, Edge, Brave, Opera, Vivaldi) version 128 or later, or Firefox. Sites in Grabbit's excluded sites keep opening them in the browser. Hold **Alt** while clicking a link to leave it to the browser: Firefox then follows it as a plain click does, while Chromium browsers download it themselves (Alt+click is their own "download link").

## No Download video button on YouTube in Chrome, Edge or Brave {#store-youtube}

The copy of the extension from the Chrome Web Store showed no **Download video** button on YouTube before version 0.5.1. From 0.5.1 it shows it. Your browser updates extensions by itself; to get it sooner, open its extensions page and update them there.

You can always save a video without the button: in Grabbit, open **Tasks → Download video from a page** and paste the video's address.

## The extension can't read this site {#site-access}

In Chrome, Edge and Brave you can limit an extension to some sites (right-click its toolbar button → **This can read and change site data**). On a site it may not read, the extension can't pass your sign-in on, so it leaves the download to the browser. Choose **On all sites** to let Grabbit take downloads everywhere.

## Firefox forgets the extension after a restart {#firefox-temporary}

An add-on loaded with **Load Temporary Add-on** (from `about:debugging`) is removed when Firefox closes. Grabbit's add-on isn't signed by Mozilla yet, so release Firefox can only load it that way: load it again after each restart, or install it for good in Firefox Developer Edition or Nightly with `xpinstall.signatures.required` set to `false` (see [the tutorial](guide-catch-browser-downloads.md#firefox)).

## Next steps

- [Browser integration →](browser-integration.md)
- [Troubleshooting →](troubleshooting.md)
- [Back to the docs index →](README.md)
