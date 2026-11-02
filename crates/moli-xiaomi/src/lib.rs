//! Xiaomi integrations: local miIO for the vacuum; cloud-authorized, local
//! video through go2rtc. No firmware modification, recording or transcoding.
mod camera;
pub mod cloud;
pub mod map;
pub mod miio;
mod vacuum;
use anyhow::ensure;
pub use camera::Camera;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, time::Duration};
use tokio::sync::Mutex;
pub use vacuum::{Vacuum, Zone};
pub const ACCOUNT: &str = "xiaomi_account";
pub const ZONE: &str = "kitchen_zone";
pub static RELAY_SETUP: Mutex<()> = Mutex::const_new(());
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Vacuum,
    Camera,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub host: Ipv4Addr,
    #[serde(default = "port")]
    pub port: u16,
    #[serde(default = "poll")]
    pub poll_s: u64,
    #[serde(default)]
    pub room: Option<String>,
    pub relay_host: String,
    #[serde(default = "relay_port")]
    pub relay_port: u16,
}
fn port() -> u16 {
    54321
}
fn poll() -> u64 {
    30
}
fn relay_port() -> u16 {
    1984
}
impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            (self.host.is_private() || self.host.is_loopback())
                && self.port > 0
                && (10..=300).contains(&self.poll_s),
            "Xiaomi needs a local address and a bounded poll interval"
        );
        self.relay().validate()
    }
    pub fn relay(&self) -> moli_tapo_camera::Relay {
        moli_tapo_camera::Relay {
            host: self.relay_host.clone(),
            port: self.relay_port,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Configured {
    pub instance: String,
    pub kind: Kind,
    pub config: Config,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub user: String,
    pub region: String,
    pub device: String,
    pub model: String,
    pub relay_config: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub commands: bool,
}
impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("XiaomiAccount(<redacted>)")
    }
}
impl Account {
    pub fn valid(&self, kind: Kind) -> bool {
        !self.user.is_empty()
            && self.user.len() <= 32
            && self.user.bytes().all(|b| b.is_ascii_digit())
            && !self.device.is_empty()
            && self.device.len() <= 32
            && self.device.bytes().all(|b| b.is_ascii_digit())
            && self.region == "de"
            && self.relay_config.len() <= 65536
            && cloud::pass_token(&self.relay_config, &self.user).is_some()
            && match kind {
                Kind::Vacuum => {
                    self.model == "rockrobo.vacuum.v1"
                        && self
                            .token
                            .as_deref()
                            .is_some_and(|t| miio::token(t).is_ok())
                }
                Kind::Camera => self.model == "isa.camera.hlc7",
            }
    }
    pub async fn cloud(&self) -> anyhow::Result<cloud::Cloud> {
        let pass = cloud::pass_token(&self.relay_config, &self.user)
            .ok_or_else(|| anyhow::anyhow!("Xiaomi account token unavailable"))?;
        cloud::Cloud::login(&self.user, &pass, &self.region).await
    }
}
/// Restore only a RAM-backed relay file. No credentials on a persistent YAML.
pub async fn restore_relay(config: &Config, account: &Account) -> anyhow::Result<()> {
    let _lock = RELAY_SETUP.lock().await;
    let expected = cloud::pass_token(&account.relay_config, &account.user)
        .ok_or_else(|| anyhow::anyhow!("Xiaomi account token unavailable"))?;
    let relay = config.relay();
    let (status, body) = relay
        .exchange(http::Method::GET, "/api/config".into(), Vec::new())
        .await?;
    if status.is_success()
        && std::str::from_utf8(&body)
            .ok()
            .and_then(|s| cloud::pass_token(s, &account.user))
            == Some(expected)
    {
        return Ok(());
    }
    let (status, _) = relay
        .exchange(
            http::Method::POST,
            "/api/config".into(),
            account.relay_config.as_bytes().to_vec(),
        )
        .await?;
    ensure!(
        status.is_success(),
        "Xiaomi relay configuration unavailable"
    );
    let _ = relay
        .exchange(http::Method::POST, "/api/restart".into(), Vec::new())
        .await;
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if let Ok(bytes) = relay.request(http::Method::GET, "/api/xiaomi".into()).await
            && serde_json::from_slice::<Vec<String>>(&bytes)
                .is_ok_and(|users| users.contains(&account.user))
        {
            return Ok(());
        }
    }
    anyhow::bail!("Xiaomi relay restart failed")
}
