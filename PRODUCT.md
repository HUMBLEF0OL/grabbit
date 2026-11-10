# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

The website's primary visitor is a Windows user who uses, or used, Internet Download Manager
(IDM), often on a trial that ran out or a cracked copy, and is looking for a free replacement
that works the same way. Many are not technical. The site and the app are in English.

Secondary visitors: people reading the help pages while using the app, and contributors (code, bug reports) arriving from GitHub.

## Product Purpose

Mister Download Manager (MDM) is a free, open-source (GPL-3.0) download manager for Windows,
laid out the way IDM users expect: same windows, menus and workflow, rebuilt from scratch in
Rust. The website's job is to get the right visitor to download it with confidence, then to
help them use it (help pages, FAQ, tutorials).

Success: an IDM user recognizes the window, believes it is faster and genuinely free, and
downloads it; a user with a question finds the answer in the help.

## Positioning

IDM's window, but faster and free. The layout is the one the visitor already knows; the speed
is measured, not claimed (a public, reproducible benchmark against IDM 6.42); the price is
nothing, with no ads, no trial and no sign-up. Torrents and video downloads are built in.

MDM is a separate program, written from scratch. It is never presented as IDM, as made by IDM's
maker (Tonec), or as a crack or copy of it.

## Operating Context

- Visitors compare against IDM, often with IDM installed or remembered.
- Download is from GitHub Releases: a Windows installer, a portable zip, a plain zip. MDM is
  Windows only: Linux and macOS builds are no longer made. The browser extension ships for Chrome, Edge, Brave, Opera,
  Vivaldi and Firefox.
- Help pages are read alongside the running app, often to find a menu or setting.
- Contact and support happen on GitHub only (issues, discussions); there is no email or form.

## Capabilities and Constraints

- Static site: `tools/website/build.py` builds `website/`, `docs/` (help) and `CHANGELOG.md`
  (news) into `target/site`; GitHub Pages serves it at misterdownloadmanager.com. Plain HTML,
  CSS and JavaScript, no framework, no external scripts except the Statcounter visit counter.
- Every page is in English only (the translated site is kept on the `archive/multiplatform-i18n`
  branch).
- Light and dark themes; the visitor's system choice is the default.
- Clean room: nothing from IDM (code, artwork, icons, text) appears; IDM is named only to
  compare.

## Evidence on Hand

- Real, unretouched screenshots of the app: `website/assets/screens/` (light and dark) and
  `docs/images/`.
- Benchmark results, final, 2026-10-29: `website/bench/results.json` (seven scenarios, alternating
  runs, IDM 6.42 vs MDM at defaults); the methodology is on the benchmark page.
- The app's logo (`website/assets/logo-*`), browser logos for the extension (`website/assets/browsers/`).
- No testimonials, reviews, user counts, press or customer logos exist; none may be invented.
  GitHub stars are few and shown only as the live count.

## Product Principles

1. Familiar first: the visitor should recognize the window before reading a word.
2. Prove, don't claim: every speed statement points to the benchmark; every image is the real app.
3. Free means free: no dark patterns, no upsell, no data collection beyond the visit counter.
4. Honest about limits: Windows only, missing features and IDM's own strengths are stated
   plainly.

## Accessibility & Inclusion

Non-technical visitors: plain words, readable sizes, full keyboard use, WCAG AA contrast in both
themes.
