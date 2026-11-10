// Mister Download Manager's link watcher (a content script, in every page and frame):
//  - a link clicked with Alt held is the browser's to download (background.js leaves its download
//    alone), the key the app's help names for it.
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

  const note = (e) => {
    if (!e.isTrusted || !(e.target instanceof Element)) return;
    const a = e.target.closest("a[href]");
    if (!a || !/^https?:/i.test(a.href)) return;
    if (e.altKey) {
      tell({ type: "bypass", url: a.href });
    }
  };
  addEventListener("click", note, true);
  addEventListener("auxclick", note, true);
  globalThis.__mdmPage = {
    alive: () => Boolean(ext.runtime?.id),
    stop: () => {
      removeEventListener("click", note, true);
      removeEventListener("auxclick", note, true);
    },
  };
})();
