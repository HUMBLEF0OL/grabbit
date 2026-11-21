// Mister Download Manager's link watcher (a content script, in every page and frame):
//  - a link clicked with Alt held is the browser's to download (background.js leaves its download
//    alone), the key the app's help names for it. The background hears of it as the button goes
//    down, before the browser can start a download. The link is then followed as a plain click
//    follows it, once the background knows, in every browser (upstream issues 332, 1494): the
//    browser's own Alt+click is stopped. Chromium would save the link without opening it
//    ("download link"), which saves a bot check's page ("Just a moment...") instead of letting
//    its script run; followed, the check runs in the tab and the download that comes after it
//    stays the browser's. Firefox does nothing with that click (it saves the link only with
//    browser.altClickSave set, off by default).
(() => {
  // One watcher per page: a copy cut off from the extension (after an update) gives way.
  const prev = globalThis.__mdmPage;
  if (prev && typeof prev === "object" && prev.alive()) return;
  if (prev && typeof prev.stop === "function") prev.stop();
  const ext = globalThis.browser ?? globalThis.chrome;

  const tell = (message) => {
    try {
      return ext.runtime.sendMessage(message).catch(() => null);
    } catch {
      // The extension was updated or removed since this page loaded: nothing to tell.
      return Promise.resolve(null);
    }
  };

  // The web link a person's click (or press) is on, else null.
  const linkOf = (e) => {
    if (!e.isTrusted || !(e.target instanceof Element)) return null;
    const a = e.target.closest("a[href]");
    return a && /^https?:/i.test(a.href) ? a : null;
  };

  // Alt held as the button goes down on a link: said before the click starts any download.
  const press = (e) => {
    if (!e.altKey || e.button > 1) return;
    const a = linkOf(e);
    if (a) tell({ type: "bypass", url: a.href });
  };

  // An Alt+click on a link the page left alone (no script of its own took it) is followed as a
  // plain click once the background has it; the browser's own handling of it (Chromium's and
  // Safari's save of the link, Firefox's nothing, or its save with browser.altClickSave) is
  // stopped, so it happens once. Alt with Ctrl, Cmd or Shift (a new tab or window), the middle
  // button: the browser's own, as they are. A `download` link is followed as such (the browser
  // downloads it, under its name).
  const follow = (e) => {
    if (!e.altKey || e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey) return;
    if (e.defaultPrevented) return;
    const a = linkOf(e);
    if (!a) return;
    e.preventDefault();
    // Not trusted: `note` and `follow` leave this click alone.
    tell({ type: "bypass", url: a.href }).then(() => a.click());
  };

  const note = (e) => {
    const a = linkOf(e);
    if (a && e.altKey) tell({ type: "bypass", url: a.href });
  };
  addEventListener("pointerdown", press, true);
  addEventListener("click", note, true);
  addEventListener("auxclick", note, true);
  // After the page's own listeners on the link and its document: a click bubbles to the window
  // last.
  addEventListener("click", follow, false);
  globalThis.__mdmPage = {
    alive: () => Boolean(ext.runtime?.id),
    stop: () => {
      removeEventListener("pointerdown", press, true);
      removeEventListener("click", note, true);
      removeEventListener("auxclick", note, true);
      removeEventListener("click", follow, false);
    },
  };
})();
