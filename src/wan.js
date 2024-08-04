const { invoke } = window.__TAURI__.tauri;
const { ask } = window.__TAURI__.dialog;

let credentialsUsernameEl;
let credentialsPasswordEl;
let credentialsSubmitEl;
let credentialsStatusEl;

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
    kind: "warning",
    title: "Neueinwahl erforderlich",
  });

  if (reconnect) {
    await wanColdReconnect();
  }
}

window.addEventListener("DOMContentLoaded", () => {
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
