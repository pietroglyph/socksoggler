const DELIMITER = "¬";

updateBadge();

// Connect to the native client
var port = browser.runtime.connectNative("socksoggler");

// Listen for messages
port.onMessage.addListener((response) => {
  console.log("Got native message: " + response);
});

browser.proxy.settings.get({}).then((r) => console.log(r.value));

browser.browserAction.onClicked.addListener(() => {
  toggleProxy();
  browser.proxy.settings.get({}).then((r) => console.log(r.value));
});

function toggleProxy() {
  isProxyOn().then((isOn) => {
    if (isOn) {
      browser.proxy.settings.set({ value: { proxyType: "none" } }).then(updateBadge);
      port.postMessage("off" + DELIMITER);
    } else
      browser.storage.sync.get().then((r) => {
        browser.proxy.settings.set({
          value: {
            proxyType: r.proxyType || "manual",
            socks: r.socksAddress || "",
            proxyDNS: true,
          }
        }).then(updateBadge);
        port.postMessage("on " + r.command + DELIMITER);
      });
  });
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
  console.log(proxySettings.value.proxyType);
  return proxySettings.value.proxyType !== "none";
}
