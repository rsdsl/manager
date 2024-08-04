const { invoke } = window.__TAURI__.tauri;

let disconnectSubmitEl;

async function disconnect() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  await invoke("disconnect", {});
  window.location = "index.html";
}

window.addEventListener("DOMContentLoaded", () => {
  disconnectSubmitEl = document.querySelector("#disconnect-submit");
  document.querySelector("#disconnect-form").addEventListener("submit", (e) => {
    e.preventDefault();
    disconnect();
  });
});
