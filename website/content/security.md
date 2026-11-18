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

## Code signing policy

Grabbit's Windows programs and installer are to be signed through the **[SignPath Foundation](https://signpath.org)**'s free code signing for open-source projects. Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

- **What is signed:** only files built by GitHub Actions from this project's public repository: `grabbit.exe`, `grabbit-cli.exe`, `grabbit-mcp.exe`, `grabbit-nmhost.exe` and the setup program. Nothing from anyone else is signed under this certificate; the helpers Grabbit downloads (yt-dlp, ffmpeg) keep their own developers' signatures and checks.
- **Who decides:** the project's maintainer, [HUMBLEF0OL](https://github.com/HUMBLEF0OL), is its committer, reviewer and approver. Changes from anyone else come in by pull request and are reviewed before they are merged; every signing request is approved by hand, release by release.
- **Accounts:** the maintainer's GitHub and SignPath accounts use two-factor sign-in.
- **Publisher name:** Windows shows the publisher as "SignPath Foundation", the holder of the certificate.
- **Privacy:** Grabbit sends nothing about you to us or anyone else. Besides the servers you download from, it talks only to GitHub to check for updates (which you can turn off) and to the services you choose; the [privacy policy](/privacy.html) lists every connection.

The technical policy, with the code each point refers to, is in [SECURITY.md]({{REPO}}/blob/main/SECURITY.md).
