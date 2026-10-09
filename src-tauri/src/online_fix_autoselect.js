// Runs only in Dusk's isolated Online-Fix download window.
// The injected source is confined to exact official file-host domains.
// It never invokes privileged Tauri APIs, interacts with ad tasks, or bypasses login.
(() => {
  "use strict";
  if (window !== window.top) return;
  const permittedHosts = new Set(["drive.online-fix.me", "hosters.online-fix.me"]);
  if (!permittedHosts.has(location.hostname) || location.protocol !== "https:") return;

  const gameTitle = __DUSK_TITLE__;
  const normalize = (text) => String(text || "").toLowerCase()
    .replace(/по\s+сети/gu, " ")
    .replace(/\b(online|multiplayer|coop|co-op)\b/g, " ")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
  const words = normalize(gameTitle).split(/\s+/).filter((word) => word.length >= 2);
  if (!words.length) return;

  const forbidden = /\b(fix|repair|updates?|patch|cracks?|redist|trainer|cheats?)\b/i;
  const archiveExt = /(?:\.part\d{1,4}\.rar|\.rar|\.7z(?:\.\d{3})?|\.zip)(?:$|[?#])/i;
  const selected = new Set();
  const navigated = new Set();
  const maxFiles = 24;
  let started = 0;
  let lastNavigation = 0;

  function safeLink(anchor) {
    let url;
    try {
      url = new URL(anchor.getAttribute("href"), location.href);
    } catch {
      return null;
    }
    if (url.protocol !== "https:" || !permittedHosts.has(url.hostname)) return null;
    const downloadName = anchor.getAttribute("download") || "";
    let path = url.pathname;
    try { path = decodeURIComponent(path); } catch {}
    const label = String(anchor.textContent || anchor.getAttribute("title") || "").trim();
    const candidate = [path.split("/").pop() || "", downloadName, label].join(" ");
    if (forbidden.test(candidate)) return null;
    const normalized = normalize(candidate);
    if (!words.every((word) => normalized.split(/\s+/).includes(word))) return null;
    return { url, candidate, downloadName };
  }

  function scan() {
    if (started >= maxFiles) return;
    const host = location.hostname;
    if (!permittedHosts.has(host)) return;
    const anchors = [...document.querySelectorAll("a[href]")];
    const files = [];
    const folders = [];
    for (const anchor of anchors) {
      const found = safeLink(anchor);
      if (!found) continue;
      if (anchor.closest('[class*="advert"],[id*="advert"],[class*="sponsor"]')) continue;
      if (archiveExt.test(found.candidate)) {
        if (!selected.has(found.url.href)) files.push({ anchor, ...found });
      } else {
        const text = String(anchor.textContent || "").toLowerCase();
        const leaf = found.url.pathname.split("/").filter(Boolean).pop() || "";
        const exactGameFolder = normalize(leaf) === normalize(gameTitle);
        const isGameFolder = /^(?:game|game files|full game|игра|файлы игры)$/i.test(text.trim());
        if ((exactGameFolder || isGameFolder) &&
            found.url.pathname !== location.pathname &&
            !navigated.has(found.url.href)) {
          folders.push({ anchor, ...found });
        }
      }
    }
    if (files.length) {
      // Start standard browser downloads. Avoid clicking advertisements and
      // skip anything not recognizably an archive for the selected game.
      files.sort((a, b) => a.url.href.localeCompare(b.url.href, undefined, { numeric: true }));
      const next = files[0];
      selected.add(next.url.href);
      started++;
      next.anchor.removeAttribute("target");
      next.anchor.click();
      return;
    }
    if (!started && folders.length && Date.now() - lastNavigation > 4000) {
      lastNavigation = Date.now();
      navigated.add(folders[0].url.href);
      folders[0].anchor.removeAttribute("target");
      folders[0].anchor.click();
    }
  }

  // Hosts can render a directory listing asynchronously after a user completes
  // authentication or a supported download gate. Keep checking only those links.
  function begin() {
    scan();
    const observer = new MutationObserver(() => { if (started < maxFiles) scan(); });
    observer.observe(document.documentElement, { subtree: true, childList: true });
    const interval = window.setInterval(scan, 1800);
    window.setTimeout(() => { observer.disconnect(); clearInterval(interval); }, 20 * 60 * 1000);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", begin, { once: true });
  } else {
    begin();
  }
})();
