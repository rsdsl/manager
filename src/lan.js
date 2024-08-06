const { invoke } = window.__TAURI__.tauri;
const { ask, message } = window.__TAURI__.dialog;

let dnsDomainEl;
let dnsUnsetEl;
let dnsSubmitEl;
let dnsStatusEl;

async function loadAllLeases() {
  for (let subnet of ["management", "trusted", "untrusted", "isolated", "exposed"]) {
    loadLeases(subnet);
  }
}

async function loadLeases(subnet) {
  let statusEl = document.querySelector("#dhcpv4-status-" + subnet);
  let tableEl = document.querySelector("#dhcpv4-clients-" + subnet);

  const leases = await invoke("leases", { subnet: subnet });

  statusEl.innerText = leases.status_text;

  let first = true;
  for (let child of tableEl.querySelectorAll("tr")) {
    if (first) {
      first = false;
      continue;
    }

    tableEl.removeChild(child);
    child.remove();
  }

  for (let lease of leases.clients) {
    let addr = document.createElement("td");
    addr.innerText = lease.addr;

    let clientId = document.createElement("td");
    clientId.innerText = lease.client_id;

    let hostname = document.createElement("td");
    hostname.innerText = lease.hostname;

    let expires = document.createElement("td");
    expires.innerText = lease.expires;

    let row = document.createElement("tr");
    row.appendChild(addr);
    row.appendChild(clientId);
    row.appendChild(hostname);
    row.appendChild(expires);

    tableEl.appendChild(row);
  }
}

async function killDns() {
  const error = await invoke("kill", { process: "rsdsl_dnsd", signal: "term" });

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "DNS-Forwarder-Neustart nicht erfolgt",
    });
  }
}

async function unsetDomain() {
  dnsDomainEl.disabled = true;
  dnsUnsetEl.disabled = true;
  dnsSubmitEl.disabled = true;
  dnsStatusEl.innerText = "Löschanfrage...";
  document.body.style.cursor = "progress";

  dnsStatusEl.innerText = await invoke("delete", { filePath: "/data/dnsd.domain" });

  dnsDomainEl.value = "";
  dnsDomainEl.disabled = false;
  dnsUnsetEl.disabled = false;
  dnsSubmitEl.disabled = false;
  document.body.style.cursor = "default";
}

async function loadDomain() {
  dnsStatusEl.innerText = "Lade aktuelle Domain...";
  document.body.style.cursor = "progress";

  const currentDomain = await invoke("load_domain", {});

  dnsDomainEl.value = currentDomain.domain;
  dnsStatusEl.innerText = currentDomain.status_text;
  document.body.style.cursor = "default";
}

async function changeDomain() {
  dnsDomainEl.disabled = true;
  dnsUnsetEl.disabled = true;
  dnsSubmitEl.disabled = true;
  dnsStatusEl.innerText = "Änderungsanfrage...";
  document.body.style.cursor = "progress";

  dnsStatusEl.innerText = await invoke("change_domain", { domain: dnsDomainEl.value });

  dnsDomainEl.disabled = false;
  dnsUnsetEl.disabled = false;
  dnsSubmitEl.disabled = false;
  document.body.style.cursor = "default";

  const reload = await ask("Zum Übernehmen der neuen lokalen Domain muss der DNS-Forwarder neu gestartet werden. Dies dauert ca. 30 Sekunden. Während dieser Zeit können abgehende Verbindungen ins Internet fehlschlagen und bestehende Verbindungen unterbrochen werden. Möglicherweis muss danach der Telefonadapter ebenfalls neu gestartet werden, um den Telefoniediesnt wiederherzustellen (falls beeinträchtigt). Möchten Sie den DNS-Forwarder jetzt neu starten?", {
    kind: "info",
    title: "DNS-Forwarder-Neustart erforderlich"
  });

  if (reload) {
    await killDns();
  }
}

window.addEventListener("DOMContentLoaded", () => {
  loadAllLeases();

  dnsDomainEl = document.querySelector("#dns-domain");
  dnsUnsetEl = document.querySelector("#dns-unset");
  dnsSubmitEl = document.querySelector("#dns-submit");
  dnsStatusEl = document.querySelector("#dns-status");

  document.querySelector("#dns-kill").addEventListener("click", (e) => {
    e.preventDefault();
    killDns();
  });
  document.querySelector("#dns-unset").addEventListener("click", (e) => {
    e.preventDefault();
    unsetDomain();
  });
  document.querySelector("#dns-form").addEventListener("submit", (e) => {
    e.preventDefault();
    changeDomain();
  });

  loadDomain();
});

setInterval(loadAllLeases, 10000);
