var port = browser.runtime.connectNative("socksoggle");

isProxyOn().then((isOn) => setProxy(isOn, true));

browser.browserAction.onClicked.addListener(() => toggleProxy());

browser.menus.create({
  id: "kill-process",
  title: "Kill Process",
  contexts: ["browser_action"],
  onclick: () => setProxy(false, true),
});


function toggleProxy() {
  isProxyOn().then((isOn) => setProxy(!isOn));
}

function setProxy(isOn, killProcess) {
  if (isOn) {
    browser.storage.sync.get().then((r) => {
      browser.proxy.settings.set({
        value: {
          proxyType: r.proxyType || "manual",
          socks: r.socksAddress || "",
          proxyDNS: true,
        }
      }).then(updateBadge);
      port.postMessage({"action": "on", "cmd": r.command});
    });
  } else {
    browser.proxy.settings.set({ value: { proxyType: "none" } }).then(updateBadge);
    if (killProcess) {
      port.postMessage({"action": "off"});
    }
  }
}

function updateBadge() {
  isProxyOn().then((isOn) => {
    browser.browserAction.setBadgeText({ text: isOn ? "On" : "Off" });
    browser.browserAction.setBadgeTextColor({ color: "white" });
    browser.browserAction.setBadgeBackgroundColor({ color: isOn ? "blue" : "red" });
  });
}

async function isProxyOn() {
  let proxySettings = await browser.proxy.settings.get({});
  return proxySettings.value.proxyType !== "none";
}
