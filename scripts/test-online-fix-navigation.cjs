const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const path = require("node:path");

const script = fs.readFileSync(
  path.join(__dirname, "..", "src-tauri", "src", "online_fix_navigation.js"),
  "utf8",
);

function createWebview(initialUrl) {
  const opened = [];
  const navigated = [];
  const listeners = [];
  const location = new URL(initialUrl);
  location.assign = (url) => navigated.push(url);

  class Element {
    constructor(href, target = "_blank") {
      this.href = href;
      this.target = target;
    }
    closest() { return this.target === "_blank" ? this : null; }
    hasAttribute(name) { return name === "download" ? false : this[name] !== undefined; }
    getAttribute(name) { return name === "href" ? this.href : this[name]; }
  }
  const window = { location };
  window.top = window;
  window.open = (url) => { opened.push(url); return null; };
  const document = {
    addEventListener(event, handler) { listeners.push({ event, handler }); },
  };
  vm.runInNewContext(script, { window, location, document, Element, URL });
  return { window, Element, listeners, opened, navigated };
}

function click(model, href) {
  const element = new model.Element(href);
  let prevented = false;
  const event = {
    target: element,
    button: 0,
    defaultPrevented: false,
    ctrlKey: false, shiftKey: false, altKey: false, metaKey: false,
    preventDefault() { prevented = true; },
    stopImmediatePropagation() {},
  };
  for (const { event: name, handler } of model.listeners) {
    if (name === "click") handler(event);
  }
  return prevented;
}

const listing = createWebview("https://online-fix.me/games/adventures/example.html");
assert.equal(click(listing, "https://hosters.online-fix.me:2053/Game"), true);
assert.equal(listing.navigated[0], "https://hosters.online-fix.me:2053/Game");
assert.equal(click(listing, "https://drive.online-fix.me:2053/Game"), true);
assert.equal(listing.navigated[1], "https://drive.online-fix.me:2053/Game");
assert.equal(click(listing, "https://advertisement.example/ad"), false);
assert.equal(listing.navigated.length, 2);

listing.window.open("https://drive.online-fix.me:2053/Game", "_blank");
assert.equal(listing.navigated.at(-1), "https://drive.online-fix.me:2053/Game");
listing.window.open("https://advertisement.example/ad", "_blank");
assert.equal(listing.opened.length, 1);

const unrelated = createWebview("https://not-online-fix.example/");
assert.equal(click(unrelated, "https://drive.online-fix.me:2053/Game"), false);
assert.equal(unrelated.navigated.length, 0);
console.log("Online-Fix navigation tests passed.");
