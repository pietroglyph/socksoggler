var isProxyOn = false;
updateBadge();

// Connect to the native switcher on startup
var port = browser.runtime.connectNative("socksoggler");

// Listen for messages
port.onMessage.addListener((response) => {
  console.log("Recieved native message: " + response);
});

browser.browserAction.onClicked.addListener(() => {
  browser.proxy.settings.get({}).then((r) => console.log(r.value));
  updateBadge();
  console.log("Sending off");
  port.postMessage("off");
});

function updateBadge() {
  browser.browserAction.setBadgeText({text: isProxyOn ? "On" : "Off"});
}
