const { invoke } = window.__TAURI__.tauri;
const { message } = window.__TAURI__.dialog;

let passwordOldEl;
let passwordNewEl;
let passwordRepeatEl;
let passwordSubmitEl;
let passwordStatusEl;

async function reboot() {
  const error = await invoke("reboot", {});

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "Neustart nicht erfolgt"
    });
  }

  const successTime = Date.now();

  while ((Date.now() - successTime) < 60) {
    await message("Neustart läuft. Bitte ca. 1 Minute warten.", {
      kind: "info",
      title: "Neustart im Gange"
    });
  }
}

async function shutdown() {
  const error = await invoke("shutdown", {});

  if (error !== "") {
    await message("Befehl konnte nicht erteilt werden: " + error, {
      kind: "error",
      title: "Herunterfahren nicht erfolgt"
    });
  }

  await message("Router erfolgreich heruntergefahren. Bitte Stromkabel abziehen.", {
    kind: "info",
    title: "Herunterfahren erfolgreich"
  });

  window.location = "index.html";
}

function showPassword() {
  switch (passwordNewEl.type) {
    case "password":
      passwordNewEl.type = "text";
      passwordRepeatEl.type = "text";
      break;
    case "text":
      passwordNewEl.type = "password";
      passwordRepeatEl.type = "password";
      break;
  }
}

async function changePassword() {
  passwordOldEl.disabled = true;
  passwordNewEl.disabled = true;
  passwordRepeatEl.disabled = true;
  passwordSubmitEl.disabled = true;
  passwordStatusEl.innerText = "Änderungsanfrage...";
  document.body.style.cursor = "progress";

  passwordStatusEl.innerText = await invoke("change_sys_password", {
    old: passwordOldEl.value,
    to: passwordNewEl.value,
    repeat: passwordRepeatEl.value,
  });

  passwordOldEl.disabled = false;
  passwordNewEl.disabled = false;
  passwordRepeatEl.disabled = false;
  passwordSubmitEl.disabled = false;
  document.body.style.cursor = "default";

  if (passwordStatusEl.innerText === "Änderung erfolgreich") {
    await message("Passwort erfolgreich geändert. Melden Sie sich neu an, um das Verwaltungswerkzeug weiter benutzen zu können.", {
      kind: "info",
      title: "Neuanmeldung erforderlich",
    });

    window.location = "index.html";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#power-reboot").addEventListener("click", (e) => {
    e.preventDefault();
    reboot();
  });
  document.querySelector("#power-shutdown").addEventListener("click", (e) => {
    e.preventDefault();
    shutdown();
  });

  passwordOldEl = document.querySelector("#password-old");
  passwordNewEl = document.querySelector("#password-new");
  passwordRepeatEl = document.querySelector("#password-repeat");
  passwordSubmitEl = document.querySelector("#password-submit");
  passwordStatusEl = document.querySelector("#password-status");

  document.querySelector("#password-show").addEventListener("click", (e) => {
    e.preventDefault();
    showPassword();
  });
  document.querySelector("#password-form").addEventListener("submit", (e) => {
    e.preventDefault();
    changePassword();
  });
});
