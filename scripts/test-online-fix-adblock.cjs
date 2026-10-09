const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const script = fs.readFileSync(
  path.join(__dirname, "..", "src-tauri", "src", "online_fix_adblock.js"), "utf8",
);
function simulate(url) {
  const handlers = new Map();
  class Element {
    constructor(href) { this.href = href; }
    closest() { return this; }
    getAttribute(key) { return key === "href" ? this.href : null; }
  }
  const window = {};
  window.top = window;
  const document = {
    readyState: "loading",
    addEventListener(type, fn) { handlers.set(type, fn); },
  };
  vm.runInNewContext(script, {
    URL, Element, window, document, location: new URL(url),
  });
  return { Element, handlers };
}
function click(context, href) {
  let prevented = false;
  const event = {
    target: new context.Element(href),
    preventDefault() { prevented = true; },
    stopImmediatePropagation() {},
  };
  context.handlers.get("click")?.(event);
  return prevented;
}

const webview = simulate("https://online-fix.me/games/adventures/test.html");
assert.equal(click(webview, "https://acscdn.com/advertising.js"), true);
assert.equal(click(webview, "https://sub.exoclick.com/ad"), true);
assert.equal(click(webview, "https://themoneytizer.com/ad"), true);
assert.equal(click(webview, "https://drive.online-fix.me:2053/Game"), false);
assert.equal(click(webview, "https://hosters.online-fix.me:2053/Game"), false);
assert.equal(click(webview, "https://acscdn.com.evil.test/fake"), false);

const other = simulate("https://unrelated.example/");
assert.equal(other.handlers.has("click"), false);
console.log("Online-Fix ad blocker tests passed.");
