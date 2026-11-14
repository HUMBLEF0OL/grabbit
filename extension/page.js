// Mister Download Manager's link watcher (a content script, in every page and frame):
//  - a link clicked with Alt held is the browser's to download (background.js leaves its download
//    alone), the key the app's help names for it. The background hears of it as the button goes
//    down, before the browser can start a download (Chromium downloads such a link itself at the
//    click). Firefox does nothing with that click (its Alt+click saves a link only with
//    browser.altClickSave set, off by default), so there the link is followed as a plain click
//    follows it, once the background knows (upstream issue 332).
(() => {
  // One watcher per page: a copy cut off from the extension (after an update) gives way.
  const prev = globalThis.__mdmPage;
  if (prev && typeof prev === "object" && prev.alive()) return;
  if (prev && typeof prev.stop === "function") prev.stop();
  const ext = globalThis.browser ?? globalThis.chrome;
  const FIREFOX = /^moz-extension:/i.test(ext.runtime.getURL?.("") || "");

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

  // Firefox: an Alt+click on a link the page left alone (no script of its own took it) is
  // followed as a plain click once the background has it; the browser's own handling of it
  // (nothing, or a save with browser.altClickSave) is stopped, so it happens once.
  const follow = (e) => {
    if (!FIREFOX || !e.altKey || e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey) return;
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
