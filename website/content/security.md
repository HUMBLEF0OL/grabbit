---
title: Security
description: How to report a security problem in Grabbit privately, how releases are built and verified, and what the project does to keep you safe.
lede: Found a security problem? Please tell us privately first, so it can be fixed before it's public.
priority: 0.3
---

## Report a vulnerability

Use GitHub's private **[Report a vulnerability]({{REPO}}/security/advisories/new)** form. Only the maintainers can read it. Say what you found, how to reproduce it, and which version you used (**Help → About** shows it). Please don't open a public issue for security problems.

You'll get an answer as soon as possible. A fix is released before the details are made public, and you're credited in the advisory and the release notes unless you'd rather not be. During the 0.x versions, only the latest release gets security fixes.

## How releases are made

- Every release is built from the public source code by **GitHub Actions**, on GitHub's servers; the build logs are public.
- Each release lists the **SHA-256** checksum of every file in `SHA256SUMS.txt`. Check your download against it: see [Download](/download.html#check-your-download).
- Get Grabbit **only from this site or the project's GitHub releases**. Copies elsewhere may be altered.
- The Windows installer isn't code-signed yet, so Windows SmartScreen may warn before the first start. Code signing will be added.

## What protects you

- **Helpers are verified before they run.** yt-dlp is checked against its developers' signing key, and ffmpeg against its published checksums.
- **Nothing listens on the network.** The automation API is off by default and, when on, answers only on `127.0.0.1` with a secret token. The browser helper answers only Grabbit's own extension.
- **Downloaded file names are cleaned** before they become paths, so a server can't write outside the folder you chose.
- **Credentials stay with their site.** Logins and cookies are sent only to the host they belong to, even across redirects.
- **No telemetry**: see the [privacy policy](/privacy.html).

## Code signing

Grabbit's Windows programs and installer aren't code-signed yet, so Windows SmartScreen may warn the first time you run the setup (**More info → Run anyway**). We're looking for a code-signing certificate for an open-source project; until then, check what you downloaded against the release's signed checksums:

- **What is checked:** each release's `SHA256SUMS.txt` lists the SHA-256 of every file, and is signed with Grabbit's own release key. Grabbit's **Check for updates** verifies that signature before it installs an update, and you can check a download yourself: see [Download](/download.html#check-your-download). The helpers Grabbit downloads (yt-dlp, ffmpeg) keep their own developers' signatures and checks.
- **Who decides:** the project's maintainer, [HUMBLEF0OL](https://github.com/HUMBLEF0OL), is its committer, reviewer and approver. Changes from anyone else come in by pull request and are reviewed before they are merged; each release is built and signed by the maintainer.
- **Accounts:** the maintainer's GitHub account uses two-factor sign-in.
- **Privacy:** Grabbit sends nothing about you to us or anyone else. Besides the servers you download from, it talks only to GitHub to check for updates (which you can turn off) and to the services you choose; the [privacy policy](/privacy.html) lists every connection.

The technical policy, with the code each point refers to, is in [SECURITY.md]({{REPO}}/blob/main/SECURITY.md).
