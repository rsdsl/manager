// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::State;

use reqwest::{Client, Response};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Session {
    client: Client,
    instance: Option<Instance>,
}

#[derive(Clone, Debug)]
struct Instance {
    url: Url,
    password: String,
}

#[derive(Debug, Serialize)]
struct WanCredentials {
    username: String,
    password: String,
    status_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WanCredentialFile {
    username: String,
    password: String,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn connect(
    url: &str,
    password: String,
    state: State<'_, Mutex<Session>>,
) -> Result<String, ()> {
    let instance = Instance {
        url: match url.parse() {
            Ok(url) => url,
            Err(e) => return Ok(format!("Ungültige URL: {}", e)),
        },
        password: password,
    };

    let response = state
        .lock()
        .unwrap()
        .client
        .get(instance.url.join("/proc/top").unwrap())
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_connect_response(response, instance, state),
        Err(e) => format!("Verbindungsaufbau fehlgeschlagen: {}", e),
    })
}

fn handle_connect_response(
    response: Response,
    instance: Instance,
    state: State<Mutex<Session>>,
) -> String {
    let status = response.status();
    if status.is_success() {
        state.lock().unwrap().instance = Some(instance);
        format!("Verbindungsaufbau erfolgreich")
    } else if status == StatusCode::UNAUTHORIZED {
        format!("Ungültiges Passwort")
    } else if status.is_client_error() {
        format!("Clientseitiger Fehler: {}", status)
    } else if status.is_server_error() {
        format!("Serverseitiger Fehler: {}", status)
    } else {
        format!("Unerwarteter Statuscode: {}", status)
    }
}

#[tauri::command]
fn disconnect(state: State<Mutex<Session>>) {
    state.lock().unwrap().instance = None;
}

#[tauri::command]
async fn load_wan_credentials(state: State<'_, Mutex<Session>>) -> Result<WanCredentials, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(WanCredentials {
                username: String::new(),
                password: String::new(),
                status_text: String::from(
                    "Keine Instanz ausgewählt, bitte melden Sie sich neu an!",
                ),
            })
        }
    };

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", "/data/pppoe.conf")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_load_wan_credentials_response(response).await,
        Err(e) => WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: format!("Abruf der aktuellen Zugangsdaten fehlgeschlagen: {}", e),
        },
    })
}

async fn handle_load_wan_credentials_response(response: Response) -> WanCredentials {
    let status = response.status();
    if status.is_success() {
        match response.json::<WanCredentialFile>().await {
            Ok(credentials) => WanCredentials {
                username: credentials.username,
                password: credentials.password,
                status_text: String::new(),
            },
            Err(e) => WanCredentials {
                username: String::new(),
                password: String::new(),
                status_text: format!(
                    "Fehlerhafte Konfigurationsdatei, bitte Zugangsdatenänderung vornehmen. Fehler: {}", e
                ),
            },
        }
    } else if status == StatusCode::UNAUTHORIZED {
        WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: String::from(
                "Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!",
            ),
        }
    } else if status == StatusCode::NOT_FOUND {
        WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: String::from("Keine Zugangsdaten eingestellt"),
        }
    } else if status.is_client_error() {
        WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: format!("Clientseitiger Fehler: {}", status),
        }
    } else if status.is_server_error() {
        WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: format!("Serverseitiger Fehler: {}", status),
        }
    } else {
        WanCredentials {
            username: String::new(),
            password: String::new(),
            status_text: format!("Unerwarteter Statuscode: {}", status),
        }
    }
}

#[tauri::command]
async fn change_wan_credentials(
    credentials: WanCredentialFile,
    state: State<'_, Mutex<Session>>,
) -> Result<String, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(String::from(
                "Keine Instanz ausgewählt, bitte melden Sie sich neu an!",
            ))
        }
    };

    let response = client
        .post(instance.url.join("/data/write").unwrap())
        .query(&[("path", "/data/pppoe.conf")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .json(&credentials)
        .send();

    Ok(match response.await {
        Ok(response) => handle_change_wan_credentials_response(response),
        Err(e) => format!("Änderung fehlgeschlagen: {}", e),
    })
}

fn handle_change_wan_credentials_response(response: Response) -> String {
    let status = response.status();
    if status.is_success() {
        String::from("Änderung erfolgreich")
    } else if status == StatusCode::UNAUTHORIZED {
        String::from("Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!")
    } else if status.is_client_error() {
        format!("Clientseitiger Fehler: {}", status)
    } else if status.is_server_error() {
        format!("Serverseitiger Fehler: {}", status)
    } else {
        format!("Unerwarteter Statuscode: {}", status)
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Session {
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("error creating http client"),
            instance: None,
        }))
        .invoke_handler(tauri::generate_handler![
            connect,
            disconnect,
            load_wan_credentials,
            change_wan_credentials
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
