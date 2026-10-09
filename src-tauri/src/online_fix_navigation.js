// Work around WebView2/Tauri dropping target="_blank" links in isolated
// external-page windows. Navigate trusted file-host links in the same webview.
// This has no access to Tauri's privileged application APIs.
(() => {
  "use strict";

  if (window !== window.top) return;

  const officialHosts = new Set([
    "online-fix.me",
    "www.online-fix.me",
    "drive.online-fix.me",
    "hosters.online-fix.me",
    "uploads.online-fix.me",
  ]);
  if (location.protocol !== "https:" || !officialHosts.has(location.hostname)) return;

  function officialDestination(value) {
    if (typeof value !== "string" || !value.trim()) return null;
    try {
      const url = new URL(value, location.href);
      const suffix = url.pathname.toLowerCase();
      const looksLikeArchive = [".zip", ".rar", ".7z", ".001"]
        .some((extension) => suffix.endsWith(extension));
      const blockedAdHost = [
        "exoclick.com", "exosrv.com", "magsrv.com", "realsrv.com",
        "juicyads.com", "adsterra.com", "popads.net", "popcash.net",
        "propellerads.com", "onclickads.net",
      ].some((name) => url.hostname === name || url.hostname.endsWith("." + name));
      // Signed archive links can originate from other HTTPS CDN domains.
      // Do not navigate to arbitrary third-party HTML/ad landing pages.
      return url.protocol === "https:"
        && !blockedAdHost
        && (officialHosts.has(url.hostname) || looksLikeArchive)
        ? url : null;
    } catch {
      return null;
    }
  }

  // Both plain <a target="_blank"> and scripted window.open() are common.
  // Keep these specific trusted links inside the same Dusk window instead
  // of depending on WebView2's sometimes missing NewWindowRequested callback.
  const nativeOpen = window.open.bind(window);
  window.open = function(url, target, features) {
    const destination = officialDestination(url);
    if (destination) {
      window.location.assign(destination.href);
      return window;
    }
    return nativeOpen(url, target, features);
  };

  document.addEventListener("click", (event) => {
    if (event.defaultPrevented || event.button !== 0 ||
        event.ctrlKey || event.shiftKey || event.altKey || event.metaKey) return;
    const element = event.target;
    if (!(element instanceof Element)) return;
    const anchor = element.closest('a[href][target="_blank"]');
    if (!anchor || anchor.hasAttribute("download")) return;
    const destination = officialDestination(anchor.getAttribute("href"));
    if (!destination) return;

    event.preventDefault();
    event.stopImmediatePropagation();
    window.location.assign(destination.href);
  }, true);
})();
