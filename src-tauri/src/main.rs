// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use tauri::State;

use chrono::{DateTime, Local};
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

#[derive(Debug, Serialize)]
struct Dhcpv6Status {
    timestamp: String,
    srvaddr: String,
    srvid: String,
    t1: String,
    t2: String,
    prefix: String,
    wanaddr: String,
    preflft: String,
    validlft: String,
    dns1: String,
    dns2: String,
    aftr: String,
}

impl Dhcpv6Status {
    fn no_lease() -> Self {
        Self::with_all(String::from(
            "✖ Keine Lease vorhanden (erster Systemstart oder Stromausfall?)",
        ))
    }

    fn with_all(message: String) -> Self {
        Self {
            timestamp: message.clone(),
            srvaddr: message.clone(),
            srvid: message.clone(),
            t1: message.clone(),
            t2: message.clone(),
            prefix: message.clone(),
            wanaddr: message.clone(),
            preflft: message.clone(),
            validlft: message.clone(),
            dns1: message.clone(),
            dns2: message.clone(),
            aftr: message,
        }
    }
}

impl From<Dhcpv6Lease> for Dhcpv6Status {
    fn from(lease: Dhcpv6Lease) -> Self {
        let validity = if lease.is_valid() { "✅" } else { "❌" };

        Self {
            timestamp: format!(
                "{} {}",
                validity,
                DateTime::<Local>::from(lease.timestamp).format("%d.%m.%Y %H:%M:%S UTC%Z")
            ),
            srvaddr: if lease.server
                == SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0))
            {
                String::from("ff02::1:2 (Alle DHCPv6-Server, da der Server keine spezifische Adresse angegeben hat)")
            } else {
                lease.server.to_string()
            },
            srvid: hex::encode(lease.server_id),
            t1: if lease.t1 == 0 {
                String::from("Sofort")
            } else if lease.t1 == u32::MAX {
                String::from("Nie")
            } else {
                let remaining_secs = std::cmp::max(
                    (Duration::from_secs(lease.t1.into())
                        - lease.timestamp.elapsed().unwrap_or(Duration::ZERO))
                    .as_secs(),
                    0,
                );
                format!(
                    "Alle {} Sekunden ({} Sekunden verbleibend)",
                    lease.t1, remaining_secs
                )
            },
            t2: if lease.t2 == 0 {
                String::from("Sofort")
            } else if lease.t2 == u32::MAX {
                String::from("Nie")
            } else {
                let remaining_secs = std::cmp::max(
                    (Duration::from_secs(lease.t2.into())
                        - lease.timestamp.elapsed().unwrap_or(Duration::ZERO))
                    .as_secs(),
                    0,
                );
                format!(
                    "Alle {} Sekunden ({} Sekunden verbleibend)",
                    lease.t2, remaining_secs
                )
            },
            prefix: format!("{}/{}", lease.prefix, lease.len),
            wanaddr: format!("{}1/64", lease.prefix),
            preflft: if lease.preflft == 0 {
                String::from("⚠ Niemals für neue Verbindungen verwenden")
            } else if lease.preflft == u32::MAX {
                String::from("Unendlich")
            } else {
                let remaining_secs = std::cmp::max(
                    (Duration::from_secs(lease.preflft.into())
                        - lease.timestamp.elapsed().unwrap_or(Duration::ZERO))
                    .as_secs(),
                    0,
                );
                format!(
                    "{} Sekunden ({} Sekunden verbleibend)",
                    lease.preflft, remaining_secs
                )
            },
            validlft: if lease.validlft == 0 {
                String::from("⚠ Internetanbieter verlangte manuell sofortigen Verfall")
            } else if lease.validlft == u32::MAX {
                String::from("Unendlich")
            } else {
                let remaining_secs = std::cmp::max(
                    (Duration::from_secs(lease.validlft.into())
                        - lease.timestamp.elapsed().unwrap_or(Duration::ZERO))
                    .as_secs(),
                    0,
                );
                format!(
                    "{} Sekunden ({} Sekunden verbleibend)",
                    lease.validlft, remaining_secs
                )
            },
            dns1: lease.dns1.to_string(),
            dns2: lease.dns2.to_string(),
            aftr: match lease.aftr {
                Some(aftr) => format!("🟢 Aktiviert | Tunnel-Endpunkt (AFTR): {}", aftr),
                None => String::from("⚪ Deaktiviert"),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct Dhcpv6Lease {
    timestamp: std::time::SystemTime,
    server: SocketAddr,
    server_id: Vec<u8>,
    t1: u32,
    t2: u32,
    prefix: Ipv6Addr,
    len: u8,
    preflft: u32,
    validlft: u32,
    dns1: Ipv6Addr,
    dns2: Ipv6Addr,
    aftr: Option<String>,
}

#[derive(Debug, Serialize)]
struct Duid {
    duid: String,
    status_text: String,
}

impl Dhcpv6Lease {
    fn is_valid(&self) -> bool {
        let expiry = self.timestamp + Duration::from_secs(self.validlft.into());
        SystemTime::now() < expiry
    }
}

#[derive(Debug, Deserialize)]
struct Dhcpv4Lease {
    address: Ipv4Addr,
    expires: SystemTime,
    client_id: Vec<u8>,
    hostname: Option<String>,
}

#[derive(Debug, Serialize)]
struct LeaseRow {
    addr: String,
    client_id: String,
    hostname: String,
    expires: String,
}

#[derive(Debug, Serialize)]
struct Leases {
    clients: Vec<LeaseRow>,
    status_text: String,
}

#[derive(Debug, Serialize)]
struct Domain {
    domain: String,
    status_text: String,
}

impl FromIterator<Dhcpv4Lease> for Leases {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Dhcpv4Lease>,
    {
        Self {
            clients: iter
                .into_iter()
                .map(|lease| {
                    let remaining_secs = std::cmp::max(
                        (lease
                            .expires
                            .duration_since(SystemTime::now())
                            .unwrap_or(Duration::ZERO))
                        .as_secs(),
                        0,
                    );

                    LeaseRow {
                        addr: lease.address.to_string(),
                        client_id: hex::encode(lease.client_id),
                        hostname: lease.hostname.unwrap_or_default(),
                        expires: format!(
                            "{} ({} Sekunden verbleibend)",
                            DateTime::<Local>::from(lease.expires)
                                .format("%d.%m.%Y %H:%M:%S UTC%Z"),
                            remaining_secs
                        ),
                    }
                })
                .collect(),
            status_text: String::new(),
        }
    }
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

#[tauri::command]
async fn dhcpv6_status(state: State<'_, Mutex<Session>>) -> Result<Dhcpv6Status, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(Dhcpv6Status::with_all(String::from(
                "❗ Keine Instanz ausgewählt, bitte melden Sie sich neu an!",
            )))
        }
    };

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", "/data/dhcp6.lease")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_dhcpv6_status_response(response).await,
        Err(e) => Dhcpv6Status::with_all(format!("❗ Abfrage fehlgeschlagen: {}", e)),
    })
}

async fn handle_dhcpv6_status_response(response: Response) -> Dhcpv6Status {
    let status = response.status();
    if status.is_success() {
        match response.json::<Dhcpv6Lease>().await {
            Ok(lease) => Dhcpv6Status::from(lease),
            Err(e) => Dhcpv6Status::with_all(format!("❗ Fehlerhafte Leasedatei. Fehler: {}", e)),
        }
    } else if status == StatusCode::UNAUTHORIZED {
        Dhcpv6Status::with_all(String::from(
            "❗ Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!",
        ))
    } else if status == StatusCode::NOT_FOUND {
        Dhcpv6Status::no_lease()
    } else if status.is_client_error() {
        Dhcpv6Status::with_all(format!("❗ Clientseitiger Fehler: {}", status))
    } else if status.is_server_error() {
        Dhcpv6Status::with_all(format!("❗ Serverseitiger Fehler: {}", status))
    } else {
        Dhcpv6Status::with_all(format!("❗ Unerwarteter Statuscode: {}", status))
    }
}

#[tauri::command]
async fn load_duid(state: State<'_, Mutex<Session>>) -> Result<Duid, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(Duid {
                duid: String::new(),
                status_text: String::from(
                    "Keine Instanz ausgewählt, bitte melden Sie sich neu an!",
                ),
            })
        }
    };

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", "/data/dhcp6.duid")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_load_duid_response(response).await,
        Err(e) => Duid {
            duid: String::new(),
            status_text: format!("Abruf des aktuellen Client-DUID fehlgeschlagen: {}", e),
        },
    })
}

async fn handle_load_duid_response(response: Response) -> Duid {
    let status = response.status();
    if status.is_success() {
        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                return Duid {
                    duid: String::new(),
                    status_text: format!(
                    "Keine Rohdaten vom Server erhalten, bitte Neustart durchführen. Fehler: {}",
                    e
                ),
                }
            }
        };

        Duid {
            duid: hex::encode(bytes),
            status_text: String::new(),
        }
    } else if status == StatusCode::UNAUTHORIZED {
        Duid {
            duid: String::new(),
            status_text: String::from(
                "Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!",
            ),
        }
    } else if status == StatusCode::NOT_FOUND {
        Duid{
    duid:String::new(),
    status_text:String::from("Kein Client-DUID gespeichert (erster Systemstart oder Stromausfall?), wird bei Bedarf zufällig generiert und gespeichert"),
    }
    } else if status.is_client_error() {
        Duid {
            duid: String::new(),
            status_text: format!("Clientseitiger Fehler: {}", status),
        }
    } else if status.is_server_error() {
        Duid {
            duid: String::new(),
            status_text: format!("Serverseitiger Fehler: {}", status),
        }
    } else {
        Duid {
            duid: String::new(),
            status_text: format!("Unerwarteter Statuscode: {}", status),
        }
    }
}

#[tauri::command]
async fn change_duid(duid: String, state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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

    let bytes = match hex::decode(&duid) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Ok(format!(
                "Eingabe ist keine gültige Hexadezimalsequenz: {}",
                e
            ))
        }
    };

    let response = client
        .post(instance.url.join("/data/write").unwrap())
        .query(&[("path", "/data/dhcp6.duid")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .body(bytes)
        .send();

    Ok(match response.await {
        Ok(response) => handle_change_duid_response(response),
        Err(e) => format!("Änderung fehlgeschlagen: {}", e),
    })
}

fn handle_change_duid_response(response: Response) -> String {
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
async fn leases(subnet: String, state: State<'_, Mutex<Session>>) -> Result<Leases, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(Leases {
                clients: Vec::new(),
                status_text: String::from("Keine Instanz ausgewählt, bitte melden Sie sich neu an"),
            })
        }
    };

    let leases_path = format!(
        "/data/dhcp4d.leases_eth0{}",
        match subnet.as_str() {
            "management" => "",
            "trusted" => ".10",
            "untrusted" => ".20",
            "isolated" => ".30",
            "exposed" => ".40",
            subnet =>
                return Ok(Leases {
                    clients: Vec::new(),
                    status_text: format!(
                        r#"Anwendungsinterner Fehler: Ungültiges Subnetz ("management", "trusted", "untrusted", "isolated" oder "exposed" erwartet, "{}" erhalten)"#,
                        subnet
                    )
                }),
        }
    );

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", leases_path)])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_leases_response(response).await,
        Err(e) => Leases {
            clients: Vec::new(),
            status_text: format!("Abfrage fehlgeschlagen: {}", e),
        },
    })
}

async fn handle_leases_response(response: Response) -> Leases {
    let status = response.status();
    if status.is_success() {
        match response.json::<Vec<Dhcpv4Lease>>().await {
            Ok(leases) => Leases::from_iter(leases),
            Err(e) => Leases {
                clients: Vec::new(),
                status_text: format!("Fehlerhafte Leasedatei. Fehler: {}", e),
            },
        }
    } else if status == StatusCode::UNAUTHORIZED {
        Leases {
            clients: Vec::new(),
            status_text: String::from(
                "Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!",
            ),
        }
    } else if status == StatusCode::NOT_FOUND {
        Leases {
            clients: Vec::new(),
            status_text: String::from(
                "Noch keine Leasedatei vorhanden (erster Systemstart oder Stromausfall?)",
            ),
        }
    } else if status.is_client_error() {
        Leases {
            clients: Vec::new(),
            status_text: format!("Clientseitiger Fehler: {}", status),
        }
    } else if status.is_server_error() {
        Leases {
            clients: Vec::new(),
            status_text: format!("Serverseitiger Fehler: {}", status),
        }
    } else {
        Leases {
            clients: Vec::new(),
            status_text: format!("Unerwarteter Statuscode: {}", status),
        }
    }
}

#[tauri::command]
async fn load_domain(state: State<'_, Mutex<Session>>) -> Result<Domain, ()> {
    let (client, instance) = {
        let state = state.lock().unwrap();
        (state.client.clone(), state.instance.clone())
    };
    let instance = match instance {
        Some(instance) => instance,
        None => {
            return Ok(Domain {
                domain: String::new(),
                status_text: String::from(
                    "Keine Instanz ausgewählt, bitte melden Sie sich neu an!",
                ),
            })
        }
    };

    let response = client
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", "/data/dnsd.domain")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_load_domain_response(response).await,
        Err(e) => Domain {
            domain: String::new(),
            status_text: format!("Abfrage fehlgeschlagen: {}", e),
        },
    })
}

async fn handle_load_domain_response(response: Response) -> Domain {
    let status = response.status();
    if status.is_success() {
        match response.text().await {
            Ok(domain) => Domain {
                domain,
                status_text: String::new(),
            },
            Err(e) => Domain {
                domain: String::new(),
                status_text: format!("Keinen Text vom Server erhalten. Fehler: {}", e),
            },
        }
    } else if status == StatusCode::UNAUTHORIZED {
        Domain {
            domain: String::new(),
            status_text: format!("Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!"),
        }
    } else if status == StatusCode::NOT_FOUND {
        Domain {
            domain: String::new(),
            status_text: format!("Keine lokale Domain eingestellt"),
        }
    } else if status.is_client_error() {
        Domain {
            domain: String::new(),
            status_text: format!("Clientseitiger Fehler: {}", status),
        }
    } else if status.is_server_error() {
        Domain {
            domain: String::new(),
            status_text: format!("Serverseitiger Fehler: {}", status),
        }
    } else {
        Domain {
            domain: String::new(),
            status_text: format!("Unerwarteter Statuscode: {}", status),
        }
    }
}

#[tauri::command]
async fn change_domain(domain: String, state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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
        .query(&[("path", "/data/dnsd.domain")])
        .basic_auth("rustkrazy", Some(&instance.password))
        .body(domain)
        .send();

    Ok(match response.await {
        Ok(response) => handle_change_domain_response(response),
        Err(e) => format!("Änderung fehlgeschlagen: {}", e),
    })
}

fn handle_change_domain_response(response: Response) -> String {
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
async fn delete(file_path: String, state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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
        .post(instance.url.join("/data/remove").unwrap())
        .query(&[("path", file_path)])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_delete_response(response),
        Err(e) => format!("Löschung fehlgeschlagen: {}", e),
    })
}

fn handle_delete_response(response: Response) -> String {
    let status = response.status();
    if status.is_success() {
        String::from("Löschung erfolgreich")
    } else if status == StatusCode::UNAUTHORIZED {
        String::from("Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!")
    } else if status == StatusCode::NOT_FOUND {
        String::from("Bereits gelöscht (keine unnötige Änderung vorgenommen)")
    } else if status.is_client_error() {
        format!("Clientseitiger Fehler: {}", status)
    } else if status.is_server_error() {
        format!("Serverseitiger Fehler: {}", status)
    } else {
        format!("Unerwarteter Statuscode: {}", status)
    }
}

#[tauri::command]
async fn change_sys_password(
    old: String,
    to: String,
    repeat: String,
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

    if to != repeat {
        return Ok(String::from(
            "Das neue Passwort und seine Wiederholung stimmen nicht überein",
        ));
    }

    let response = client
        .post(instance.url.join("/data/write").unwrap())
        .query(&[("path", "/data/admind.passwd")])
        .basic_auth("rustkrazy", Some(&old))
        .body(to)
        .send();

    Ok(match response.await {
        Ok(response) => handle_change_sys_password_response(response),
        Err(e) => format!("Änderung fehlgeschlagen: {}", e),
    })
}

fn handle_change_sys_password_response(response: Response) -> String {
    let status = response.status();
    if status.is_success() {
        String::from("Änderung erfolgreich")
    } else if status == StatusCode::UNAUTHORIZED {
        String::from("Das alte Passwort ist ungültig")
    } else if status.is_client_error() {
        format!("Clientseitiger Fehler: {}", status)
    } else if status.is_server_error() {
        format!("Serverseitiger Fehler: {}", status)
    } else {
        format!("Unerwarteter Statuscode: {}", status)
    }
}

#[tauri::command]
async fn reboot(state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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
        .post(instance.url.join("/reboot").unwrap())
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_reboot_response(response),
        Err(e) => format!("Befehl fehlgeschlagen: {}", e),
    })
}

fn handle_reboot_response(response: Response) -> String {
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
async fn shutdown(state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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
        .post(instance.url.join("/shutdown").unwrap())
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_shutdown_response(response),
        Err(e) => format!("Befehl fehlgeschlagen: {}", e),
    })
}

fn handle_shutdown_response(response: Response) -> String {
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
async fn log_read(logfile: String, state: State<'_, Mutex<Session>>) -> Result<String, ()> {
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
        .get(instance.url.join("/data/read").unwrap())
        .query(&[("path", format!("/tmp/{}", logfile))])
        .basic_auth("rustkrazy", Some(&instance.password))
        .send();

    Ok(match response.await {
        Ok(response) => handle_log_read_response(response).await,
        Err(e) => format!("Abfrage fehlgeschlagen: {}", e),
    })
}

async fn handle_log_read_response(response: Response) -> String {
    let status = response.status();
    if status.is_success() {
        match response.text().await {
            Ok(logs) => logs,
            Err(e) => format!("Keinen Text vom Server erhalten. Fehler: {}", e),
        }
    } else if status == StatusCode::UNAUTHORIZED {
        String::from("Ungültiges Verwaltungspasswort, bitte melden Sie sich neu an!")
    } else if status == StatusCode::NOT_FOUND {
        String::from(
            "Protokolldatei existiert nicht, möglicherweise ist der Dienst noch nicht gestartet",
        )
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
                .resolve(
                    "rsdsl",
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 128, 10, 254)), 8443),
                )
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
            connection_status,
            dhcpv6_status,
            load_duid,
            change_duid,
            leases,
            load_domain,
            change_domain,
            delete,
            change_sys_password,
            reboot,
            shutdown,
            log_read
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
