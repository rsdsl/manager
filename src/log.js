const { invoke } = window.__TAURI__.tauri;

function dashboard() {
  window.location = "dashboard.html";
}

function wanOpen() {
  window.location = "wan.html";
}

function lanOpen() {
  window.location = "lan.html";
}

function ddnsOpen() {
  window.location = "ddns.html";
}

function logOpen() {
  window.location = "log.html";
}

function sysOpen() {
  window.location = "sys.html";
}

async function disconnect() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  await invoke("disconnect", {});
  window.location = "index.html";
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#dashboard-form").addEventListener("submit", (e) => {
    e.preventDefault();
    dashboard();
  });
  document.querySelector("#wan-open-form").addEventListener("submit", (e) => {
    e.preventDefault();
    wanOpen();
  });
  document.querySelector("#lan-open-form").addEventListener("submit", (e) => {
    e.preventDefault();
    lanOpen();
  });
  document.querySelector("#ddns-open-form").addEventListener("submit", (e) => {
    e.preventDefault();
    ddnsOpen();
  });
  document.querySelector("#log-open-form").addEventListener("submit", (e) => {
    e.preventDefault();
    logOpen();
  });
  document.querySelector("#sys-open-form").addEventListener("submit", (e) => {
    e.preventDefault();
    sysOpen();
  });
  document.querySelector("#disconnect-form").addEventListener("submit", (e) => {
    e.preventDefault();
    disconnect();
  });
});
