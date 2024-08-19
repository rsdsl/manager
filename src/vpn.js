const { invoke } = window.__TAURI__.tauri;
const { ask, message } = window.__TAURI__.dialog;

let clientTableEl;
let clientStatusEl;

let addNameEl;
let addPubkeyEl;
let addPskEl;
let addAllowedIpsEl;
let addSubmitEl;
let addStatusEl;

async function serverRestart() {
  const error = await invoke("kill", { process: "rsdsl_wgd", signal: "term" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "VPN-Server-Neustart nicht erfolgt"
    });
  }
}

async function deleteClient(name) {
  const statusText = await invoke("vpndel", { name: name });

  if (statusText === "Löschung erfolgreich") {
    await loadClients();
    await message("Zum Anwenden der Änderung muss der VPN-Server manuell neu gestartet werden.", {
      kind: "info",
      title: "VPN-Server-Neustart erforderlich",
    });
  } else {
    await message("Befehl konnte nicht erteilt werden: " + statusText, {
      kind: "error",
      title: "Clientlöschung nicht erfolgt",
    });
  }
}

async function loadClients() {
  const clients = await invoke("vpnclients", {});

  clientStatusEl.innerText = clients.status_text;

  let first = true;
  for (let child of clientTableEl.querySelectorAll("tr")) {
    if (first) {
      first = false;
      continue;
    }

    clientTableEl.removeChild(child);
    child.remove();
  }

  for (let client of clients.clients) {
    let name = document.createElement("td");
    name.innerText = client.name;

    let pubkey = document.createElement("td");
    pubkey.innerText = client.pubkey;

    let pskSpan = document.createElement("span");
    pskSpan.className = "hoverunhide";
    pskSpan.innerText = client.psk;

    let pskPlaceholder = document.createElement("span");
    pskPlaceholder.className = "hoverhide";
    pskPlaceholder.innerText = "(versteckt)";

    let psk = document.createElement("td");
    psk.className = "hider";
    psk.appendChild(pskSpan);
    psk.appendChild(pskPlaceholder);

    let allowedIps = document.createElement("td");
    allowedIps.innerText = client.allowed_ips;

    let deleter = document.createElement("button");
    deleter.innerText = "🗑";
    deleter.addEventListener("click", async function(e) {
      e.preventDefault();

      const confirmed = await ask("Möchten Sie die VPN-Zugangsberechtigung für " + client.name + " wirklich entfernen?", {
        kind: "warn",
        title: "Entfernen bestätigen"
      });

      if (confirmed) {
        await deleteClient(client.name);
      }
    });

    let row = document.createElement("tr");
    row.appendChild(name);
    row.appendChild(pubkey);
    row.appendChild(psk);
    row.appendChild(allowedIps);
    row.appendChild(deleter);

    clientTableEl.appendChild(row);
  }
}

function showPsk() {
  switch (addPskEl.type) {
    case "password":
      addPskEl.type = "text";
      break;
    case "text":
      addPskEl.type = "password";
      break;
  }
}

async function addClient() {
  addNameEl.disabled = true;
  addPubkeyEl.disabled = true;
  addPskEl.disabled = true;
  addAllowedIpsEl.disabled = true;
  addSubmitEl.disabled = true;
  addStatusEl.innerText = "Änderungsanfrage...";
  document.body.style.cursor = "progress";

  addStatusEl.innerText = await invoke("vpnadd", {
    name: addNameEl.value,
    pubkey: addPubkeyEl.value,
    psk: addPskEl.value,
    allowedIps: addAllowedIpsEl.value,
  });

  addNameEl.disabled = false;
  addPubkeyEl.disabled = false;
  addPskEl.disabled = false;
  addAllowedIpsEl.disabled = false;
  addSubmitEl.disabled = false;
  document.body.style.cursor = "default";

  await loadClients();
  await message("Zum Anwenden der Änderung muss der VPN-Server manuell neu gestartet werden.", {
    kind: "info",
    title: "VPN-Server-Neustart erforderlich",
  });
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#server-restart").addEventListener("click", (e) => {
    e.preventDefault();
    serverRestart();
  });

  clientTableEl = document.querySelector("#client-clients");
  clientStatusEl = document.querySelector("#client-status");

  loadClients();

  addNameEl = document.querySelector("#add-name");
  addPubkeyEl = document.querySelector("#add-pubkey");
  addPskEl = document.querySelector("#add-psk");
  addAllowedIpsEl = document.querySelector("#add-allowedips");
  addSubmitEl = document.querySelector("#add-submit");
  addStatusEl = document.querySelector("#add-status");

  document.querySelector("#add-show").addEventListener("click", (e) => {
    e.preventDefault();
    showPsk();
  });
  document.querySelector("#add-form").addEventListener("submit", (e) => {
    e.preventDefault();
    addClient();
  });
});

setInterval(loadClients, 10000);
