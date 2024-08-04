const { invoke } = window.__TAURI__.tauri;

let connectUrlEl;
let connectPasswordEl;
let connectSubmitEl;
let connectStatusEl;

async function connect() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  connectUrlEl.disabled = true;
  connectPasswordEl.disabled = true;
  connectSubmitEl.disabled = true;
  connectStatusEl.innerText = "Verbindungsaufbau...";
  document.body.style.cursor = "progress";

  connectStatusEl.innerText = await invoke("connect", {
    url: connectUrlEl.value,
    password: connectPasswordEl.value,
  });

  if (connectStatusEl.innerText === "Verbindungsaufbau erfolgreich") {
    window.location = "dashboard.html";
  }

  connectUrlEl.disabled = false;
  connectPasswordEl.disabled = false;
  connectSubmitEl.disabled = false;
  document.body.style.cursor = "default";
}

window.addEventListener("DOMContentLoaded", () => {
  connectUrlEl = document.querySelector("#connect-url");
  connectPasswordEl = document.querySelector("#connect-password");
  connectSubmitEl = document.querySelector("#connect-submit");
  connectStatusEl = document.querySelector("#connect-status");
  document.querySelector("#connect-form").addEventListener("submit", (e) => {
    e.preventDefault();
    connect();
  });
});
