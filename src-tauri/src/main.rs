// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::State;

use reqwest::{Client, Response};
use reqwest::{StatusCode, Url};

#[derive(Debug)]
struct Session {
    client: Client,
    instance: Option<Instance>,
}

#[derive(Debug)]
struct Instance {
    url: Url,
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

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Session {
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("error creating http client"),
            instance: None,
        }))
        .invoke_handler(tauri::generate_handler![connect, disconnect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
