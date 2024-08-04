const { invoke } = window.__TAURI__.tauri;
const { ask, message } = window.__TAURI__.dialog;

let connectionStatusEl;
let connectionIpv4El;
let connectionIpv6El;

let credentialsUsernameEl;
let credentialsPasswordEl;
let credentialsSubmitEl;
let credentialsStatusEl;

async function refreshConnectionStatus() {
  const connectionStatus = await invoke("connection_status", {});

  connectionStatusEl.innerText = connectionStatus.session;
  connectionIpv4El.innerText = connectionStatus.ipv4;
  connectionIpv6El.innerText = connectionStatus.ipv6;
}

async function warmReconnect() {
  const error = await invoke("kill", { process: "rsdsl_pppoe3", signal: "hup" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "Neusynchronisation nicht erfolgt"
    });
  }
}

async function coldReconnect() {
  const error = await invoke("kill", { process: "rsdsl_pppoe3", signal: "term" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "Neueinwahl nicht erfolgt"
    });
  }
}

async function forceReconnect() {
  const error = await invoke("kill", { process: "rsdsl_pppoe3", signal: "kill" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "Neueinwahl nicht erfolgt"
    });
  }
}

function showCredentials() {
  switch (credentialsPasswordEl.type) {
    case "password":
      credentialsPasswordEl.type = "text";
      break;
    case "text":
      credentialsPasswordEl.type = "password";
      break;
  }
}

async function loadCredentials() {
  credentialsStatusEl.innerText = "Lade aktuelle Zugangsdaten...";
  document.body.style.cursor = "progress";

  const currentCredentials = await invoke("load_wan_credentials", {});

  credentialsUsernameEl.value = currentCredentials.username;
  credentialsPasswordEl.value = currentCredentials.password;
  credentialsStatusEl.innerText = currentCredentials.status_text;
  document.body.style.cursor = "default";
}

async function changeCredentials() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  credentialsUsernameEl.disabled = true;
  credentialsPasswordEl.disabled = true;
  credentialsSubmitEl.disabled = true;
  credentialsStatusEl.innerText = "Änderungsanfrage...";
  document.body.style.cursor = "progress";

  credentialsStatusEl.innerText = await invoke("change_wan_credentials", {
    credentials: {
      username: credentialsUsernameEl.value,
      password: credentialsPasswordEl.value
    }
  });

  credentialsUsernameEl.disabled = false;
  credentialsPasswordEl.disabled = false;
  credentialsSubmitEl.disabled = false;
  document.body.style.cursor = "default";

  const reconnect = await ask("Zum Übernehmen der neuen Zugangsdaten muss die Einwahl zum Internetanbieter neu aufgebaut werden. Dies dauert ca. 30 Sekunden. Möchten Sie die Einwahl jetzt neu herstellen?", {
    kind: "info",
    title: "Neueinwahl erforderlich"
  });

  if (reconnect) {
    await coldReconnect();
  }
}

window.addEventListener("DOMContentLoaded", () => {
  refreshConnectionStatus();

  connectionStatusEl = document.querySelector("#connection-status");
  connectionIpv4El = document.querySelector("#connection-ipv4");
  connectionIpv6El = document.querySelector("#connection-ipv6");

  document.querySelector("#connection-warm-reconnect").addEventListener("click", (e) => {
    e.preventDefault();
    warmReconnect();
  });
  document.querySelector("#connection-cold-reconnect").addEventListener("click", (e) => {
    e.preventDefault();
    coldReconnect();
  });
  document.querySelector("#connection-force-reconnect").addEventListener("click", (e) => {
    e.preventDefault();
    forceReconnect();
  });

  credentialsUsernameEl = document.querySelector("#credentials-username");
  credentialsPasswordEl = document.querySelector("#credentials-password");
  credentialsSubmitEl = document.querySelector("#credentials-submit");
  credentialsStatusEl = document.querySelector("#credentials-status");

  document.querySelector("#credentials-show").addEventListener("click", (e) => {
    e.preventDefault();
    showCredentials();
  });

  document.querySelector("#credentials-form").addEventListener("submit", (e) => {
    e.preventDefault();
    changeCredentials();
  });

  loadCredentials();
});

setInterval(refreshConnectionStatus, 3000);
