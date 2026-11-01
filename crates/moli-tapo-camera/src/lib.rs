//! Tapo RTSP cameras through an isolated go2rtc relay. No recording or writes to cameras.
use anyhow::{bail, ensure};
use http_body_util::Full;
use moli_core::Device;
use moli_runtime::{
    BoxFuture, Driver, DriverCtx,
    media::{VideoFile, VideoSource},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

pub const ACCOUNT: &str = "camera_account";
static CHECK: Mutex<()> = Mutex::const_new(());

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub host: Ipv4Addr,
    pub model: String,
    #[serde(default = "rtsp_port")]
    pub rtsp_port: u16,
    pub relay_host: String,
    #[serde(default = "relay_port")]
    pub relay_port: u16,
}
fn rtsp_port() -> u16 {
    554
}
fn relay_port() -> u16 {
    1984
}

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub username: String,
    pub password: String,
}
impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CameraAccount(<redacted>)")
    }
}
impl Account {
    pub fn valid(&self) -> bool {
        !self.username.is_empty()
            && self.username.len() <= 128
            && !self.password.is_empty()
            && self.password.len() <= 256
            && !self
                .username
                .chars()
                .chain(self.password.chars())
                .any(char::is_control)
    }
}
impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            (self.host.is_private() || self.host.is_loopback())
                && self.rtsp_port > 0
                && matches!(self.model.as_str(), "C200" | "C320WS"),
            "invalid camera configuration"
        );
        ensure!(
            !self.relay_host.is_empty()
                && self.relay_host.len() <= 253
                && self
                    .relay_host
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
                && self.relay_port > 0,
            "invalid relay configuration"
        );
        Ok(())
    }
    async fn request(&self, method: http::Method, path: String) -> anyhow::Result<Vec<u8>> {
        let req = http::Request::builder()
            .method(method)
            .uri(path)
            .body(Full::new(moli_net::Body::new()))?;
        let (status, body) = moli_net::web(
            &self.relay_host,
            self.relay_port,
            false,
            req,
            Duration::from_secs(12),
            4 * 1024 * 1024,
        )
        .await
        .map_err(|_| anyhow::anyhow!("camera relay unavailable"))?;
        ensure!(status.is_success(), "camera request failed");
        Ok(body.to_vec())
    }
    async fn register(&self, name: &str, account: &Account) -> anyhow::Result<()> {
        ensure!(account.valid(), "invalid camera account");
        let source = format!(
            "rtsp://{}:{}@{}:{}/stream2#backchannel=0",
            encode(&account.username),
            encode(&account.password),
            self.host,
            self.rtsp_port
        );
        self.request(
            http::Method::PATCH,
            format!("/api/streams?name={}&src={}", encode(name), encode(&source)),
        )
        .await?;
        Ok(())
    }
    /// Probes only the declared camera. Never returns upstream bodies or credentials.
    pub async fn verify(&self, instance: &str, account: &Account) -> anyhow::Result<()> {
        self.validate()?;
        // One reusable probe name bounds relay memory even if a caller disconnects.
        let _checking = CHECK.lock().await;
        let name = format!("{}_verify", stream_name(instance));
        self.register(&name, account).await?;
        let response = self
            .request(
                http::Method::GET,
                format!("/api/streams?src={}&video=h264", encode(&name)),
            )
            .await;
        let bytes = response?;
        let report: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("invalid camera response"))?;
        ensure!(
            report
                .get("producers")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|p| !p.is_empty()),
            "camera did not supply video"
        );
        Ok(())
    }
}
fn encode(value: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}
fn stream_name(instance: &str) -> String {
    use std::fmt::Write as _;
    let mut s = String::from("moli_");
    for b in instance.bytes() {
        let _ = write!(s, "{b:02x}");
    }
    s
}
fn valid_session(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

#[derive(Default)]
struct State {
    account: Option<Account>,
    sessions: HashMap<String, Instant>,
}
struct Video {
    config: Config,
    name: String,
    state: Mutex<State>,
}
impl std::fmt::Debug for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TapoVideo(<redacted>)")
    }
}
impl VideoSource for Video {
    fn file<'a>(
        &'a self,
        resource: &'a str,
        session: Option<&'a str>,
    ) -> BoxFuture<'a, anyhow::Result<VideoFile>> {
        Box::pin(async move {
            let mut state = self.state.lock().await;
            state
                .sessions
                .retain(|_, at| at.elapsed() < Duration::from_secs(30));
            let (path, content_type) = match resource {
                "stream.m3u8" if session.is_none() => {
                    ensure!(state.sessions.len() < 8, "too many camera viewers");
                    let account = state
                        .account
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("camera account missing"))?;
                    self.config.register(&self.name, account).await?;
                    (
                        format!("/api/stream.m3u8?src={}&video=h264", encode(&self.name)),
                        "application/vnd.apple.mpegurl",
                    )
                }
                "hls/playlist.m3u8" | "hls/init.mp4" | "hls/segment.m4s" => {
                    let id = session
                        .filter(|id| valid_session(id))
                        .ok_or_else(|| anyhow::anyhow!("invalid video session"))?;
                    let at = state
                        .sessions
                        .get_mut(id)
                        .ok_or_else(|| anyhow::anyhow!("expired video session"))?;
                    *at = Instant::now();
                    let mime = if resource.ends_with("m3u8") {
                        "application/vnd.apple.mpegurl"
                    } else {
                        "video/mp4"
                    };
                    (format!("/api/{resource}?id={id}"), mime)
                }
                _ => bail!("unknown video resource"),
            };
            let result = self.config.request(http::Method::GET, path).await;
            let bytes = match result {
                Ok(b) => b,
                Err(e) => {
                    return Err(e);
                }
            };
            if resource.ends_with("m3u8") {
                let text = std::str::from_utf8(&bytes)?;
                ensure!(
                    bytes.len() <= 64 * 1024 && text.starts_with("#EXTM3U"),
                    "invalid video playlist"
                );
                if resource == "stream.m3u8" {
                    let id = text
                        .lines()
                        .find_map(|l| l.strip_prefix("hls/playlist.m3u8?id="))
                        .filter(|id| valid_session(id))
                        .ok_or_else(|| anyhow::anyhow!("invalid video playlist"))?;
                    state.sessions.insert(id.to_owned(), Instant::now());
                }
                validate_playlist(text, session)?;
            }
            Ok(VideoFile {
                content_type,
                bytes,
            })
        })
    }
}
fn validate_playlist(text: &str, session: Option<&str>) -> anyhow::Result<()> {
    for line in text.lines() {
        let uri = if let Some(map) = line.strip_prefix("#EXT-X-MAP:URI=\"") {
            Some(map.trim_end_matches('"'))
        } else if line.is_empty() || line.starts_with('#') {
            ensure!(!line.contains("URI="), "unknown video URI");
            None
        } else {
            Some(line)
        };
        if let Some(uri) = uri {
            let (file, query) = uri
                .split_once("?id=")
                .ok_or_else(|| anyhow::anyhow!("invalid video playlist"))?;
            ensure!(
                matches!(file, "hls/playlist.m3u8" | "init.mp4" | "segment.m4s"),
                "invalid video resource"
            );
            let id = query.split('&').next().unwrap_or("");
            ensure!(
                valid_session(id) && session.is_none_or(|s| s == id),
                "invalid video session"
            );
            ensure!(
                !query.contains(['\"', '\\', ':', '/', '%']),
                "invalid video query"
            );
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Camera {
    config: Config,
}
impl Camera {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}
impl Driver for Camera {
    fn kind(&self) -> &'static str {
        "tapo-camera"
    }
    fn run<'a>(&'a self, ctx: &'a mut DriverCtx) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let id = ctx.device_id("camera");
            let video = Arc::new(Video {
                config: self.config.clone(),
                name: stream_name(ctx.instance().as_str()),
                state: Mutex::new(State::default()),
            });
            ctx.provide_video(&id, video.clone());
            ctx.upsert_device(Device {
                id: id.clone(),
                instance: ctx.instance().clone(),
                native_name: format!("Tapo {}", self.config.model).into(),
                manufacturer: Some("TP-Link".into()),
                model: Some(self.config.model.clone().into()),
                description: None,
                native_room: None,
                members: Vec::new(),
                points: Vec::new(),
            });
            let mut last = None;
            loop {
                let secret = ctx.secret(ACCOUNT);
                if secret != last {
                    let account = secret
                        .as_deref()
                        .and_then(|s| serde_json::from_str::<Account>(s).ok())
                        .filter(Account::valid);
                    let present = account.is_some();
                    let mut state = video.state.lock().await;
                    state.account = account;
                    state.sessions.clear();
                    drop(state);
                    ctx.set_availability(&id, present);
                    if present {
                        ctx.ready();
                    } else {
                        ctx.wait_for(moli_i18n::tr!("camera.account.wait"));
                    }
                    last = secret;
                } else if last.is_none() {
                    ctx.set_availability(&id, false);
                    ctx.wait_for(moli_i18n::tr!("camera.account.wait"));
                }
                tokio::select! { () = ctx.cancelled() => return Ok(()), () = tokio::time::sleep(Duration::from_secs(3)) => {} }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_are_encoded_and_never_debugged() {
        assert_eq!(encode("a@b:#/ %"), "a%40b%3A%23%2F%20%25");
        assert!(
            !Account {
                username: "a".into(),
                password: "\n".into()
            }
            .valid()
        );
        assert_ne!(stream_name("a_b"), stream_name("a-b"));
    }
    #[test]
    fn playlists_cannot_escape_the_camera_or_relay() {
        assert!(validate_playlist("#EXTM3U\nhls/playlist.m3u8?id=Ab12", None).is_ok());
        assert!(
            validate_playlist(
                "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4?id=Ab12\"\nsegment.m4s?id=Ab12&n=1",
                Some("Ab12")
            )
            .is_ok()
        );
        for bad in [
            "https://example.com/leak",
            "../secret?id=Ab12",
            "segment.m4s?id=Other",
            "segment.m4s?id=Ab12&x=http://example.com",
        ] {
            assert!(validate_playlist(&format!("#EXTM3U\n{bad}"), Some("Ab12")).is_err());
        }
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    #[tokio::test]
    async fn a_viewer_cannot_use_another_cameras_session_or_an_expired_one() {
        let video = Video {
            config: Config {
                host: "127.0.0.1".parse().unwrap(),
                model: "C200".into(),
                rtsp_port: 8554,
                relay_host: "127.0.0.1".into(),
                relay_port: 9,
            },
            name: "test".into(),
            state: Mutex::new(State::default()),
        };
        video
            .state
            .lock()
            .await
            .sessions
            .insert("OwnSession".into(), Instant::now());
        assert!(
            video
                .file("hls/init.mp4", Some("OtherSession"))
                .await
                .unwrap_err()
                .to_string()
                .contains("expired")
        );
        assert!(video.file("../api/streams", None).await.is_err());
        video.state.lock().await.sessions.insert(
            "OldSession".into(),
            Instant::now().checked_sub(Duration::from_secs(31)).unwrap(),
        );
        assert!(
            video
                .file("hls/init.mp4", Some("OldSession"))
                .await
                .unwrap_err()
                .to_string()
                .contains("expired")
        );
    }
    #[test]
    fn secrets_are_redacted_and_public_targets_are_refused() {
        let account = Account {
            username: "fixture-user".into(),
            password: "fixture-password".into(),
        };
        let text = format!("{account:?}");
        assert!(!text.contains(&account.username) && !text.contains(&account.password));
        let config = Config {
            host: "203.0.113.1".parse().unwrap(),
            model: "C200".into(),
            rtsp_port: 554,
            relay_host: "127.0.0.1".into(),
            relay_port: 1984,
        };
        assert!(config.validate().is_err());
        assert!(
            validate_playlist("#EXTM3U\n#EXT-X-KEY:URI=\"https://example.com\"", None).is_err()
        );
    }
}
