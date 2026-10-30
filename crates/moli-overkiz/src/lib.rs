//! Overkiz cloud, using the legacy session login (not the local API).
//! Protocol reference: python-overkiz-api and Somfy's enduser API schemas.
//! No response body, cookie, login or native URL is ever included in errors.
mod model;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use http::{Method, Request, StatusCode};
use moli_core::DeviceId;
use moli_net::{empty, json_body, web_with_headers};
use moli_runtime::{BoxFuture, CommandRequest, Driver, DriverCtx};
use serde::Deserialize;
use serde_json::{Value as Json, json};

const HOST: &str = "ha101-1.overkiz.com";
const BASE: &str = "/enduser-mobile-web/enduserAPI/";
const MAX_BODY: usize = 4 * 1024 * 1024;
const LIMIT: Duration = Duration::from_secs(4);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Enable physical writes only after comparing the discovered capabilities.
    #[serde(default = "yes")]
    pub read_only: bool,
    #[serde(default = "poll_default")]
    pub poll_seconds: u64,
}
const fn yes() -> bool {
    true
}
const fn poll_default() -> u64 {
    10
}

#[derive(Debug)]
pub struct Overkiz {
    config: Config,
}
impl Overkiz {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        anyhow::ensure!(
            (5..=300).contains(&config.poll_seconds),
            "poll_seconds must be 5..300"
        );
        Ok(Self { config })
    }
}
impl Driver for Overkiz {
    fn kind(&self) -> &'static str {
        "overkiz"
    }
    fn run<'a>(&'a self, ctx: &'a mut DriverCtx) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let (Some(username), Some(password)) = (ctx.secret("username"), ctx.secret("password"))
            else {
                ctx.wait_for(moli_i18n::tr!("pilotes.overkiz.identifiants"));
                ctx.cancelled().await;
                return Ok(());
            };
            let result = run(&self.config, ctx, &username, &password).await;
            if let Err(error) = &result {
                for id in ctx.devices() {
                    ctx.set_availability(&id, false);
                }
                ctx.wait_for(error.to_string());
                // Cloud failures must not turn the supervisor's backoff into a login storm.
                let until = tokio::time::sleep(Duration::from_secs(300));
                tokio::pin!(until);
                loop {
                    tokio::select! {
                        () = &mut until => break,
                        command = ctx.next_command() => match command {
                            Some(command) => command.reply(Err(moli_i18n::tr!("pilotes.overkiz.indisponible"))),
                            None => return Ok(()),
                        }
                    }
                }
            }
            result
        })
    }
}

// No Debug: a session contains a credential even if it has no password.
#[derive(Clone, Copy)]
struct Endpoint {
    host: &'static str,
    port: u16,
    tls: bool,
}
const CLOUD: Endpoint = Endpoint {
    host: HOST,
    port: 443,
    tls: true,
};
struct Session {
    cookie: String,
    endpoint: Endpoint,
}
impl Session {
    async fn login(username: &str, password: &str) -> anyhow::Result<Self> {
        Self::login_at(CLOUD, username, password).await
    }
    async fn login_at(endpoint: Endpoint, username: &str, password: &str) -> anyhow::Result<Self> {
        let body = format!(
            "userId={}&userPassword={}",
            model::encode(username),
            model::encode(password)
        );
        let builder = Request::builder()
            .method(Method::POST)
            .uri(format!("{BASE}login"))
            .header("content-type", "application/x-www-form-urlencoded")
            .header("user-agent", "moli-os");
        let mut request = empty(builder)?;
        *request.body_mut() = moli_net::Body::from(body).into();
        let (status, headers, bytes) = web_with_headers(
            endpoint.host,
            endpoint.port,
            endpoint.tls,
            request,
            LIMIT,
            MAX_BODY,
        )
        .await
        .map_err(|_| anyhow::anyhow!(moli_i18n::tr!("pilotes.overkiz.indisponible")))?;
        check_status(status)?;
        if status != StatusCode::NO_CONTENT {
            let value: Json = serde_json::from_slice(&bytes)
                .context(moli_i18n::tr!("pilotes.overkiz.reponse"))?;
            anyhow::ensure!(
                value["success"] == true,
                "{}",
                moli_i18n::tr!("pilotes.overkiz.authentification")
            );
        }
        let cookie = headers
            .get_all(http::header::SET_COOKIE)
            .iter()
            .filter_map(|h| h.to_str().ok())
            .find_map(|h| {
                h.split(';')
                    .next()
                    .filter(|s| s.starts_with("JSESSIONID=") && s.len() > 11)
            })
            .context(moli_i18n::tr!("pilotes.overkiz.authentification"))?
            .to_owned();
        Ok(Self { cookie, endpoint })
    }
    async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<&Json>,
        limit: Duration,
    ) -> anyhow::Result<Json> {
        let builder = Request::builder()
            .method(method)
            .uri(format!("{BASE}{path}"))
            .header("cookie", &self.cookie)
            .header("user-agent", "moli-os");
        let request = match body {
            Some(body) => json_body(builder, body)?,
            None => empty(builder)?,
        };
        let (status, _, bytes) = web_with_headers(
            self.endpoint.host,
            self.endpoint.port,
            self.endpoint.tls,
            request,
            limit,
            MAX_BODY,
        )
        .await
        .map_err(|_| anyhow::anyhow!(moli_i18n::tr!("pilotes.overkiz.indisponible")))?;
        check_status(status)?;
        let value: Json =
            serde_json::from_slice(&bytes).context(moli_i18n::tr!("pilotes.overkiz.reponse"))?;
        anyhow::ensure!(
            value.get("error").is_none(),
            "{}",
            moli_i18n::tr!("pilotes.overkiz.refus")
        );
        Ok(value)
    }
}
fn check_status(status: StatusCode) -> anyhow::Result<()> {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            bail!(moli_i18n::tr!("pilotes.overkiz.authentification"))
        }
        StatusCode::TOO_MANY_REQUESTS => bail!(moli_i18n::tr!("pilotes.overkiz.limite")),
        status if !status.is_success() => bail!(moli_i18n::tr!("pilotes.overkiz.refus")),
        _ => Ok(()),
    }
}
async fn run(
    config: &Config,
    ctx: &mut DriverCtx,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let session = Session::login(username, password).await?;
    let registered = session
        .call(Method::POST, "events/register", Some(&json!({})), LIMIT)
        .await?;
    let listener = registered["id"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256)
        .context(moli_i18n::tr!("pilotes.overkiz.reponse"))?;
    let result = connected(config, ctx, &session, listener).await;
    // Best effort, including failed startup: do not leave a listener polling nowhere.
    let _ = session
        .call(
            Method::POST,
            &format!("events/{}/unregister", model::encode(listener)),
            Some(&json!({})),
            LIMIT,
        )
        .await;
    result
}
async fn connected(
    config: &Config,
    ctx: &mut DriverCtx,
    session: &Session,
    listener: &str,
) -> anyhow::Result<()> {
    let setup = session
        .call(Method::GET, "setup/devices", None, LIMIT)
        .await?;
    let devices = setup
        .as_array()
        .context(moli_i18n::tr!("pilotes.overkiz.reponse"))?;
    anyhow::ensure!(
        devices.len() <= 1024,
        "{}",
        moli_i18n::tr!("pilotes.overkiz.reponse")
    );
    let mut known = HashMap::new();
    for raw in devices {
        let Some(device) = model::Equipment::parse(raw) else {
            continue;
        };
        let id = ctx.device_id(&model::encode(&device.url));
        ctx.upsert_device(device.device(&id, ctx.instance(), config.read_only));
        device.publish(ctx, &id);
        known.insert(id, device);
    }
    for id in ctx.devices() {
        if !known.contains_key(&id) {
            ctx.remove_device(&id);
        }
    }
    ctx.ready();
    let mut tick = tokio::time::interval(Duration::from_secs(config.poll_seconds));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            command = ctx.next_command() => {
                let Some(command) = command else { return Ok(()) };
                send_command(config, &known, session, command).await;
            }
            _ = tick.tick() => {
                let events = session.call(Method::POST, &format!("events/{}/fetch", model::encode(listener)), Some(&json!({})), LIMIT).await?;
                let events = events.as_array().context(moli_i18n::tr!("pilotes.overkiz.reponse"))?;
                for event in events {
                    if matches!(event["name"].as_str(), Some("DeviceCreatedEvent" | "DeviceRemovedEvent")) {
                        bail!(moli_i18n::tr!("pilotes.overkiz.inventaire"));
                    }
                    let Some(url) = event["deviceURL"].as_str() else { continue };
                    let id = ctx.device_id(&model::encode(url));
                    if let Some(device) = known.get_mut(&id) { device.event(ctx, &id, event); }
                }
            }
        }
    }
}
async fn send_command(
    config: &Config,
    known: &HashMap<DeviceId, model::Equipment>,
    session: &Session,
    command: CommandRequest,
) {
    if command.abandoned() {
        return;
    }
    let payload = known
        .get(&command.device.id)
        .and_then(|device| device.order(config.read_only, &command.key, &command.value));
    let Some(payload) = payload else {
        command.reply(Err(moli_i18n::tr!("pilotes.overkiz.lecture_seule")));
        return;
    };
    let limit = command
        .deadline
        .saturating_duration_since(Instant::now())
        .min(LIMIT);
    if limit.is_zero() {
        return;
    }
    let result = session
        .call(Method::POST, "exec/apply", Some(&payload), limit)
        .await
        .and_then(|answer| {
            anyhow::ensure!(
                answer["execId"].as_str().is_some(),
                "{}",
                moli_i18n::tr!("pilotes.overkiz.refus")
            );
            Ok(())
        })
        .map_err(|error| error.to_string());
    // Acceptance is not state confirmation. Only the event listener changes state.
    command.reply(result);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_and_http_failures_are_bounded() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert!(config.read_only);
        assert!(Overkiz::new(config).is_ok());
        for seconds in [0, 1, 301] {
            assert!(
                Overkiz::new(Config {
                    read_only: true,
                    poll_seconds: seconds
                })
                .is_err()
            );
        }
        for status in [
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::BAD_GATEWAY,
        ] {
            assert!(check_status(status).is_err());
        }
        assert!(check_status(StatusCode::OK).is_ok());
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    async fn mock(
        replies: Vec<(u16, &'static str, &'static str)>,
    ) -> (Endpoint, tokio::task::JoinHandle<Vec<String>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            let mut received = Vec::new();
            for (status, headers, body) in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                loop {
                    let mut buffer = [0_u8; 4096];
                    let count = socket.read(&mut buffer).await.unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    let text = String::from_utf8_lossy(&bytes);
                    if let Some(end) = text.find("\r\n\r\n") {
                        let length = text[..end]
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                received.push(String::from_utf8(bytes).unwrap());
                socket.write_all(format!("HTTP/1.1 {status} Response\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}",body.len()).as_bytes()).await.unwrap();
            }
            received
        });
        (
            Endpoint {
                host: "127.0.0.1",
                port,
                tls: false,
            },
            task,
        )
    }

    #[tokio::test]
    async fn legacy_login_cookie_and_expiry_use_the_same_session_without_replay() {
        let (endpoint, requests) = mock(vec![
            (
                200,
                "Set-Cookie: JSESSIONID=synthetic; Path=/; HttpOnly\r\n",
                r#"{"success":true}"#,
            ),
            (200, "", r#"{"id":"listener-test"}"#),
            (401, "", r#"{"error":"never disclose this body"}"#),
        ])
        .await;
        let session = Session::login_at(endpoint, "user@example.com", "fictitious+&password")
            .await
            .unwrap();
        assert_eq!(
            session
                .call(Method::POST, "events/register", Some(&json!({})), LIMIT)
                .await
                .unwrap()["id"],
            "listener-test"
        );
        let error = session
            .call(
                Method::POST,
                "events/listener-test/fetch",
                Some(&json!({})),
                LIMIT,
            )
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("never disclose"));
        let received = requests.await.unwrap();
        assert!(
            received[0].contains("userId=user%40example.com&userPassword=fictitious%2B%26password")
        );
        assert!(received[1].contains("JSESSIONID=synthetic"));
        assert_eq!(received.len(), 3);
    }

    #[tokio::test]
    async fn a_cloud_execution_refusal_is_not_replayed_or_reported_as_success() {
        let (endpoint, requests) = mock(vec![(
            200,
            "",
            r#"{"errorCode":"EXEC_QUEUE_FULL","error":"synthetic rejected order"}"#,
        )])
        .await;
        let session = Session {
            cookie: "JSESSIONID=synthetic".into(),
            endpoint,
        };
        let error = session
            .call(
                Method::POST,
                "exec/apply",
                Some(&json!({"actions":[]})),
                LIMIT,
            )
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("synthetic rejected order"));
        assert_eq!(requests.await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn successful_login_without_session_cookie_is_refused() {
        let (endpoint, requests) = mock(vec![(200, "", r#"{"success":true}"#)]).await;
        assert!(
            Session::login_at(endpoint, "user@example.com", "synthetic")
                .await
                .is_err()
        );
        assert_eq!(requests.await.unwrap().len(), 1);
    }
}
