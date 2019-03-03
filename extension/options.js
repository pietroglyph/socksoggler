function saveOptions(e) {
    e.preventDefault();
    browser.storage.sync.set({
        proxyType: document.querySelector("#settings").elements["proxyType"].value,
        command: document.querySelector("#command").value,
        socksAddress: document.querySelector("#socksAddress").value,
    });
}

function restoreOptions() {

    function setElements(result) {
        document.querySelector("#settings").querySelector('input[value="' + (result.proxyType || "manual") + '"]').checked = true;
        document.querySelector("#command").value = result.command || "";
        document.querySelector("#socksAddress").value = result.socksAddress || "";
    }

    browser.storage.sync.get().then(
        setElements, (e) => console.error(e)
    );
}

document.addEventListener("DOMContentLoaded", restoreOptions);
document.querySelector("form").addEventListener("submit", saveOptions);