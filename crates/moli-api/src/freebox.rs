//! Local Freebox diagnostics. Pairing tokens remain in the encrypted hub store.
use crate::{caller::Caller, people::People, session::Sessions};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use moli_runtime::Hub;
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fmt::Write as _,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
const HOST: &str = "mafreebox.freebox.fr";
const APP: &str = "org.moli.network";
const MAX_BODY: usize = 2 * 1024 * 1024;
#[derive(Deserialize, Serialize)]
struct Account {
    token: String,
    version: u16,
}
struct Attempt {
    token: String,
    track: u64,
    version: u16,
    owner: String,
    until: Instant,
}
#[derive(Default)]
struct Data {
    attempt: Option<Attempt>,
    session: Option<String>,
    cached: Option<(Instant, Value)>,
}
#[derive(Clone, Default)]
struct Service(Arc<Mutex<Data>>);
fn fail(status: StatusCode) -> Response {
    (
        status,
        Json(json!({"error": moli_i18n::tr("freebox.failed")})),
    )
        .into_response()
}
async fn call(
    version: Option<u16>,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> anyhow::Result<Value> {
    let path = version.map_or_else(|| format!("/{path}"), |v| format!("/api/v{v}/{path}"));
    let mut builder = http::Request::builder()
        .method(if body.is_some() {
            Method::POST
        } else {
            Method::GET
        })
        .uri(path);
    if let Some(token) = token {
        builder = builder.header("X-Fbx-App-Auth", token);
    }
    let request = if let Some(body) = body {
        moli_net::json_body(builder, &body)?
    } else {
        moli_net::empty(builder)?
    };
    let (status, bytes) =
        moli_net::web(HOST, 80, false, request, Duration::from_secs(3), MAX_BODY).await?;
    anyhow::ensure!(status.is_success(), "Freebox request failed");
    let reply: Value = serde_json::from_slice(&bytes)?;
    if version.is_none() {
        return Ok(reply);
    }
    anyhow::ensure!(reply["success"] == true, "Freebox rejected request");
    Ok(reply["result"].clone())
}
fn configured(hub: &Hub) -> Option<Account> {
    hub.secret(&"freebox".into(), "account")
        .and_then(|s| serde_json::from_str(&s).ok())
}
pub(crate) fn routes() -> Router<Hub> {
    Router::new()
        .route("/api/freebox", get(status))
        .route("/api/freebox/traffic", get(traffic))
        .route("/api/freebox/pair", post(pair))
        .route("/api/freebox/confirm", post(confirm))
        .layer(Extension(Service::default()))
}
async fn status(State(hub): State<Hub>) -> Json<Value> {
    Json(json!({"configured": configured(&hub).is_some()}))
}
async fn pair(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN);
    }
    let mut data = service.0.lock().await;
    if let Some(attempt) = &data.attempt
        && attempt.until > Instant::now()
    {
        return fail(StatusCode::CONFLICT);
    }
    let Ok(version) = call(None, "api_version", None, None).await else {
        return fail(StatusCode::BAD_GATEWAY);
    };
    let Some(version) = version["api_version"]
        .as_str()
        .and_then(|s| s.split('.').next()?.parse::<u16>().ok())
        .filter(|v| (4..=99).contains(v))
    else {
        return fail(StatusCode::BAD_GATEWAY);
    };
    let Ok(reply) = call(
        Some(version),
        "login/authorize/",
        None,
        Some(
            json!({"app_id": APP,"app_name":"Moli OS","app_version":"1.0","device_name":"Moli OS"}),
        ),
    )
    .await
    else {
        return fail(StatusCode::BAD_GATEWAY);
    };
    let (Some(token), Some(track)) = (reply["app_token"].as_str(), reply["track_id"].as_u64())
    else {
        return fail(StatusCode::BAD_GATEWAY);
    };
    if token.is_empty() || token.len() > 256 {
        return fail(StatusCode::BAD_GATEWAY);
    }
    data.attempt = Some(Attempt {
        token: token.into(),
        track,
        version,
        owner: caller.key,
        until: Instant::now() + Duration::from_secs(180),
    });
    // Existing credentials remain usable until the replacement pairing is granted.
    let _ = hub;
    Json(json!({"pending":true})).into_response()
}
async fn confirm(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN);
    }
    let mut data = service.0.lock().await;
    let Some(attempt) = &data.attempt else {
        return fail(StatusCode::CONFLICT);
    };
    if attempt.owner != caller.key || attempt.until <= Instant::now() {
        return fail(StatusCode::CONFLICT);
    }
    let Ok(reply) = call(
        Some(attempt.version),
        &format!("login/authorize/{}", attempt.track),
        None,
        None,
    )
    .await
    else {
        return fail(StatusCode::BAD_GATEWAY);
    };
    if matches!(reply["status"].as_str(), Some("pending" | "unknown")) {
        return Json(json!({"pending":true})).into_response();
    }
    if reply["status"] != "granted" {
        data.attempt = None;
        return fail(StatusCode::CONFLICT);
    }
    let Ok(secret) = serde_json::to_string(&Account {
        token: attempt.token.clone(),
        version: attempt.version,
    }) else {
        return fail(StatusCode::INTERNAL_SERVER_ERROR);
    };
    if hub
        .store_secret(&"freebox".into(), "account", &secret)
        .await
        .is_err()
    {
        return fail(StatusCode::SERVICE_UNAVAILABLE);
    }
    data.attempt = None;
    data.session = None;
    data.cached = None;
    Json(json!({"configured":true})).into_response()
}
fn password(token: &str, challenge: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, token.as_bytes());
    hmac::sign(&key, challenge.as_bytes()).as_ref().iter().fold(
        String::with_capacity(40),
        |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        },
    )
}
async fn collect(account: &Account, session: &mut Option<String>) -> anyhow::Result<Value> {
    let version = Some(account.version);
    let connection = if let Some(token) = session.as_deref() {
        call(version, "connection/", Some(token), None).await.ok()
    } else {
        None
    };
    let connection = if let Some(connection) = connection {
        connection
    } else {
        let login = call(version, "login/", None, None).await?;
        let challenge = login["challenge"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing challenge"))?;
        anyhow::ensure!(challenge.len() <= 256, "Invalid challenge");
        let reply = call(
            version,
            "login/session/",
            None,
            Some(json!({"app_id":APP,"password":password(&account.token,challenge)})),
        )
        .await?;
        let token = reply["session_token"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() <= 512)
            .ok_or_else(|| anyhow::anyhow!("Missing session"))?;
        *session = Some(token.into());
        call(version, "connection/", session.as_deref(), None).await?
    };
    let token = session.as_deref();
    let (ports, hosts, wifi0, wifi1, stat1, stat2, stat3) = tokio::try_join!(
        call(version, "switch/status/", token, None),
        call(version, "lan/browser/pub/", token, None),
        call(version, "wifi/ap/0/stations/", token, None),
        call(version, "wifi/ap/1/stations/", token, None),
        call(version, "switch/port/1/stats/", token, None),
        call(version, "switch/port/2/stats/", token, None),
        call(version, "switch/port/3/stats/", token, None)
    )?;
    anyhow::ensure!(
        ports.as_array().is_some_and(|v| v.len() <= 16)
            && hosts.as_array().is_some_and(|v| v.len() <= 2048),
        "Invalid topology"
    );
    Ok(
        json!({"connection":connection,"ports":ports,"hosts":hosts,"radios":[wifi0,wifi1],"stats":{"1":stat1,"2":stat2,"3":stat3},"timestamp":moli_core::now_ms()}),
    )
}
async fn traffic(State(hub): State<Hub>, Extension(service): Extension<Service>) -> Response {
    let Some(account) = configured(&hub) else {
        return (StatusCode::NOT_FOUND, Json(json!({"configured":false}))).into_response();
    };
    let mut data = service.0.lock().await;
    if let Some((at, value)) = &data.cached
        && at.elapsed() < Duration::from_millis(4500)
    {
        return Json(value.clone()).into_response();
    }
    let result = Box::pin(tokio::time::timeout(
        Duration::from_secs(8),
        collect(&account, &mut data.session),
    ))
    .await;
    if let Ok(Ok(value)) = result {
        data.cached = Some((Instant::now(), value.clone()));
        Json(value).into_response()
    } else {
        data.cached = None;
        fail(StatusCode::BAD_GATEWAY)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn freebox_hmac_sha1() {
        assert_eq!(
            super::password("key", "The quick brown fox jumps over the lazy dog"),
            "de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9"
        );
    }
}
