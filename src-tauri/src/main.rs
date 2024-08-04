// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{Ipv4Addr, Ipv6Addr};
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

#[derive(Debug, Serialize)]
struct ConnectionStatus {
    session: String,
    ipv4: String,
    ipv6: String,
}

#[derive(Debug, Default, Deserialize)]
struct ConnectionFile {
    v4: Option<Ipv4Connection>,
    v6: Option<Ipv6Connection>,
}

impl ConnectionFile {
    fn session_summary(&self) -> String {
        if self.v4.is_some() && self.v6.is_some() {
            String::from("Einwahlstatus: ✅ Dual Stack")
        } else if self.v6.is_some() {
            String::from(
                r#"Einwahlstatus: ✅ IPv6 | ggf. DS-Lite-Status unter "DHCPv6" überprüfen"#,
            )
        } else if self.v4.is_some() {
            String::from(
                r#"Einwahlstatus: ⚠ IPv4 | Eigene Server nicht von außen erreichbar, kleine Teile des modernen Internets nicht erreichbar. Internetanbieter um Freischaltung von IPv6 (bevorzugt "Dual Stack" bzw. mit öffentlicher IPv4-Adresse <i>und</i> IPv6, aber nicht zwingend nötig) bitten."#,
            )
        } else {
            String::from("Einwahlstatus: ❌ Keine Einwahl | Router und Modem neu starten. Bei weiterem Bestehen Diagnoseprotokolle konsultieren oder Internetanbieter kontaktieren.")
        }
    }

    fn ipv4_summary(&self) -> String {
        if let Some(v4) = &self.v4 {
            format!("IPv4: 🟢 Verbunden | Öffentliche Adresse: {}/32 | Primärer DNS-Server (nicht verwendet): {} | Sekundärer DNS-Server (nicht verwendet): {}",v4.addr,v4.dns1,v4.dns2)
        } else if self.v6.is_some() {
            String::from(
                r#"IPv4: 🟡 Nicht verfügbar (ggf. DS-Lite-Status unter "DHCPv6" überprüfen)"#,
            )
        } else {
            String::from("IPv4: 🔴 Nicht verbunden")
        }
    }

    fn ipv6_summary(&self) -> String {
        if let Some(v6) = &self.v6 {
            format!(
                "IPv6: 🟢 Verbunden | Verbindungslokale Adresse: {}/128 | Standardgateway: {}",
                v6.laddr, v6.raddr
            )
        } else if self.v4.is_some() {
            String::from("IPv6: 🟡 Nicht verfügbar | Bitte freischalten lassen (s. oben)")
        } else {
            String::from("IPv6: 🔴 Nicht verbunden")
        }
    }
}

#[derive(Debug, Deserialize)]
struct Ipv4Connection {
    addr: Ipv4Addr,
    dns1: Ipv4Addr,
    dns2: Ipv4Addr,
}

#[derive(Debug, Deserialize)]
struct Ipv6Connection {
    laddr: Ipv6Addr,
    raddr: Ipv6Addr,
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

#[tauri::command]
async fn kill(
    process: String,
    signal: String,
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
        .post(instance.url.join("/proc/kill").unwrap())
        .query(&[("process", process), ("signal", signal)])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_kill_response(response),
        Err(e) => format!("Signalversand an Dienst fehlgeschlagen: {}", e),
    })
}

fn handle_kill_response(response: Response) -> String {
    let status = response.status();
    if status.is_success() {
        String::new()
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

#[tauri::command]
async fn connection_status(state: State<'_, Mutex<Session>>) -> Result<ConnectionStatus, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(ConnectionStatus {
                session: String::from("❗ Keine Instanz ausgewählt, bitte melden Sie sich neu an!"),
                ipv4: String::from("❗ Keine Instanz ausgewählt, bitte melden Sie sich neu an!"),
                ipv6: String::from("❗ Keine Instanz ausgewählt, bitte melden Sie sich neu an!"),
            })
        }
    };

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", "/tmp/pppoe.ip_config")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_connection_status_response(response).await,
        Err(e) => ConnectionStatus {
            session: format!("❗ Abfrage fehlgeschlagen: {}", e),
            ipv4: format!("❗ Abfrage fehlgeschlagen: {}", e),
            ipv6: format!("❗ Abfrage fehlgeschlagen: {}", e),
        },
    })
}

async fn handle_connection_status_response(response: Response) -> ConnectionStatus {
    let status = response.status();
    if status.is_success() {
        match response.json::<ConnectionFile>().await {
            Ok(connection) => ConnectionStatus {
                session: connection.session_summary(),
                ipv4: connection.ipv4_summary(),
                ipv6: connection.ipv6_summary(),
            },
            Err(e) => ConnectionStatus {
                session: format!("❗ Fehlerhafte Parameterdatei. Fehler: {}", e),
                ipv4: format!("❗ Fehlerhafte Parameterdatei. Fehler: {}", e),
                ipv6: format!("❗ Fehlerhafte Parameterdatei. Fehler: {}", e),
            },
        }
    } else if status == StatusCode::UNAUTHORIZED {
        ConnectionStatus {
            session: String::from(
                "❗ Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!",
            ),
            ipv4: String::from("❗ Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!"),
            ipv6: String::from("❗ Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!"),
        }
    } else if status == StatusCode::NOT_FOUND {
        let connection = ConnectionFile::default();
        ConnectionStatus {
            session: connection.session_summary(),
            ipv4: connection.ipv4_summary(),
            ipv6: connection.ipv6_summary(),
        }
    } else if status.is_client_error() {
        ConnectionStatus {
            session: format!("❗ Clientseitiger Fehler: {}", status),
            ipv4: format!("❗ Clientseitiger Fehler: {}", status),
            ipv6: format!("❗ Clientseitiger Fehler: {}", status),
        }
    } else if status.is_server_error() {
        ConnectionStatus {
            session: format!("❗ Serverseitiger Fehler: {}", status),
            ipv4: format!("❗ Serverseitiger Fehler: {}", status),
            ipv6: format!("❗ Serverseitiger Fehler: {}", status),
        }
    } else {
        ConnectionStatus {
            session: format!("❗ Unerwarteter Statuscode: {}", status),
            ipv4: format!("❗ Unerwarteter Statuscode: {}", status),
            ipv6: format!("❗ Unerwarteter Statuscode: {}", status),
        }
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
            change_wan_credentials,
            kill,
            connection_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
