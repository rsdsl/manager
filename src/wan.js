const { invoke } = window.__TAURI__.tauri;
const { ask, message } = window.__TAURI__.dialog;

let connectionStatusEl;
let connectionIpv4El;
let connectionIpv6El;

let credentialsUsernameEl;
let credentialsPasswordEl;
let credentialsSubmitEl;
let credentialsStatusEl;

let dhcpv6TimestampEl;
let dhcpv6SrvAddrEl;
let dhcpv6SrvIdEl;
let dhcpv6T1El;
let dhcpv6T2El;
let dhcpv6PrefixEl;
let dhcpv6WanAddrEl;
let dhcpv6PrefLftEl;
let dhcpv6ValidLftEl;
let dhcpv6Dns1El;
let dhcpv6Dns2El;
let dhcpv6AftrEl;

let dhcpv6DuidEl;
let dhcpv6SubmitEl;
let dhcpv6StatusEl;

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

async function refreshDhcpv6Status() {
  const dhcpv6Status = await invoke("dhcpv6_status", {});

  dhcpv6TimestampEl.innerText = dhcpv6Status.timestamp;
  dhcpv6SrvAddrEl.innerText = dhcpv6Status.srvaddr;
  dhcpv6SrvIdEl.innerText = dhcpv6Status.srvid;
  dhcpv6T1El.innerText = dhcpv6Status.t1;
  dhcpv6T2El.innerText = dhcpv6Status.t2;
  dhcpv6PrefixEl.innerText = dhcpv6Status.prefix;
  dhcpv6WanAddrEl.innerText = dhcpv6Status.wanaddr;
  dhcpv6PrefLftEl.innerText = dhcpv6Status.preflft;
  dhcpv6ValidLftEl.innerText = dhcpv6Status.validlft;
  dhcpv6Dns1El.innerText = dhcpv6Status.dns1;
  dhcpv6Dns2El.innerText = dhcpv6Status.dns2;
  dhcpv6AftrEl.innerText = dhcpv6Status.aftr;
}

async function loadDuid() {
  dhcpv6StatusEl.innerText = "Lade aktuellen Client-DUID...";
  document.body.style.cursor = "progress";

  const currentDuid = await invoke("load_duid", {});

  dhcpv6DuidEl.value = currentDuid.duid;
  dhcpv6StatusEl.innerText = currentDuid.status_text;
  document.body.style.cursor = "default";
}

async function changeDuid() {
  dhcpv6DuidEl.disabled = true;
  dhcpv6SubmitEl.disabled = true;
  dhcpv6StatusEl.innerText = "Änderungsanfrage...";
  document.body.style.cursor = "progress";

  const statusText = await invoke("change_duid", { duid: dhcpv6DuidEl.value });

  dhcpv6DuidEl.disabled = false;
  dhcpv6SubmitEl.disabled = false;
  dhcpv6StatusEl.innerText = statusText;
  document.body.style.cursor = "default";

  if (statusText === "Änderung erfolgreich") {
    const apply = await ask("Zum Übernehmen des neuen Client-DUID muss der DHCPv6-Client neu gestartet werden. Dies dauert ca. 30 Sekunden, sollte die Internetverbindung aber nicht unterbrechen. Dabei wird eine Verlängerung mit Serversuche durchgeführt. Möchten Sie den DHCPv6-Client jetzt neu starten?", {
      kind: "info",
      title: "DHCPv6-Client-Neustart erforderlich",
    });

    if (apply) {
      await killDhcpv6();
    }
  }
}

async function killDhcpv6() {
  const error = await invoke("kill", { process: "rsdsl_dhcp6", signal: "term" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "DHCPv6-Client-Neustart nicht erfolgt",
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  connectionStatusEl = document.querySelector("#connection-status");
  connectionIpv4El = document.querySelector("#connection-ipv4");
  connectionIpv6El = document.querySelector("#connection-ipv6");

  refreshConnectionStatus();

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

  dhcpv6TimestampEl = document.querySelector("#dhcpv6-timestamp");
  dhcpv6SrvAddrEl = document.querySelector("#dhcpv6-srvaddr");
  dhcpv6SrvIdEl = document.querySelector("#dhcpv6-srvid");
  dhcpv6T1El = document.querySelector("#dhcpv6-t1");
  dhcpv6T2El = document.querySelector("#dhcpv6-t2");
  dhcpv6PrefixEl = document.querySelector("#dhcpv6-prefix");
  dhcpv6WanAddrEl = document.querySelector("#dhcpv6-wanaddr");
  dhcpv6PrefLftEl = document.querySelector("#dhcpv6-preflft");
  dhcpv6ValidLftEl = document.querySelector("#dhcpv6-validlft");
  dhcpv6Dns1El = document.querySelector("#dhcpv6-dns1");
  dhcpv6Dns2El = document.querySelector("#dhcpv6-dns2");
  dhcpv6AftrEl = document.querySelector("#dhcpv6-aftr");

  refreshDhcpv6Status();

  document.querySelector("#dhcpv6-kill").addEventListener("click", (e) => {
    e.preventDefault();
    killDhcpv6();
  });

  dhcpv6DuidEl = document.querySelector("#dhcpv6-duid");
  dhcpv6SubmitEl = document.querySelector("#dhcpv6-submit");
  dhcpv6StatusEl = document.querySelector("#dhcpv6-status");

  document.querySelector("#dhcpv6-form").addEventListener("submit", (e) => {
    e.preventDefault();
    changeDuid();
  });

  loadDuid();
});

setInterval(refreshConnectionStatus, 3000);
setInterval(refreshDhcpv6Status, 3000);
