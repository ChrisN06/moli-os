//! Owner-only Xiaomi setup; neither tokens nor upstream errors reach the browser.
use crate::{caller::Caller, people::People, session::Sessions};
use anyhow::Context as _;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use moli_runtime::Hub;
use moli_xiaomi::{ACCOUNT, Account, Configured, Kind, Vacuum, ZONE, Zone};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
#[derive(Clone)]
struct Setup {
    devices: Arc<Vec<Configured>>,
    login: Arc<Mutex<Option<Attempt>>>,
}
struct Attempt {
    owner: String,
    until: Instant,
    phase: &'static str,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    captcha: String,
    #[serde(default)]
    verify: String,
}
fn fail(status: StatusCode, key: &str) -> Response {
    (status, Json(json!({"error":moli_i18n::tr(key)}))).into_response()
}
fn owner(
    caller: &Caller,
    auth: &(Extension<Arc<Sessions>>, Extension<Arc<People>>),
    headers: &HeaderMap,
) -> bool {
    crate::people_api::is_owner(caller, &auth.0.0, &auth.1.0, headers)
}
pub(crate) fn routes(devices: Vec<Configured>) -> Router<Hub> {
    Router::new()
        .route("/api/xiaomi", get(status))
        .route("/api/xiaomi/login", post(login))
        .route("/api/xiaomi/discover", post(discover))
        .route("/api/xiaomi/{instance}/map", get(map))
        .route("/api/xiaomi/{instance}/zone", get(zone).put(save_zone))
        .layer(axum::extract::DefaultBodyLimit::max(8192))
        .layer(Extension(Setup {
            devices: Arc::new(devices),
            login: Arc::new(Mutex::new(None)),
        }))
}
async fn status(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    caller: Caller,
) -> Response {
    if caller
        .person
        .as_ref()
        .is_some_and(crate::people::PersonRef::is_guest)
    {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    Json(json!({"devices":setup.devices.iter().map(|d|{
 let account=hub.secret(&d.instance.as_str().into(),ACCOUNT).and_then(|s|serde_json::from_str::<Account>(&s).ok()).filter(|a|a.valid(d.kind));
 json!({"instance":d.instance,"kind":d.kind,"host":d.config.host.to_string(),"configured":account.is_some(),"commands":d.kind == Kind::Vacuum && account.is_some()})
 }).collect::<Vec<_>>()})).into_response()
}
fn check_step(step: &Login) -> bool {
    step.username.len() <= 256
        && step.password.len() <= 256
        && step.captcha.len() <= 32
        && step.verify.len() <= 32
        && !step
            .username
            .chars()
            .chain(step.password.chars())
            .chain(step.captcha.chars())
            .chain(step.verify.chars())
            .any(char::is_control)
        && ((!step.username.is_empty()
            && !step.password.is_empty()
            && step.captcha.is_empty()
            && step.verify.is_empty())
            || (step.username.is_empty()
                && step.password.is_empty()
                && (step.captcha.is_empty() != step.verify.is_empty())))
}
fn continuation(attempt: Option<&Attempt>, owner: &str, step: &Login) -> bool {
    attempt.is_some_and(|a| {
        a.owner == owner
            && a.until > Instant::now()
            && a.phase
                == if step.captcha.is_empty() {
                    "verify"
                } else {
                    "captcha"
                }
    })
}
async fn login(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Json(step): Json<Login>,
) -> Response {
    if !owner(&caller, &auth, &headers) {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    if !check_step(&step) {
        return fail(StatusCode::BAD_REQUEST, "xiaomi.account.invalid");
    }
    let Some(device) = setup.devices.first() else {
        return fail(
            StatusCode::SERVICE_UNAVAILABLE,
            "xiaomi.account.unavailable",
        );
    };
    let mut attempt = setup.login.lock().await;
    if attempt
        .as_ref()
        .is_some_and(|a| a.owner != caller.key && a.until > Instant::now())
    {
        return fail(StatusCode::CONFLICT, "xiaomi.account.busy");
    }
    if step.username.is_empty() && !continuation(attempt.as_ref(), &caller.key, &step) {
        return fail(StatusCode::BAD_REQUEST, "xiaomi.account.expired");
    }
    let _relay_setup = moli_xiaomi::RELAY_SETUP.lock().await;
    let form = BTreeMap::from([
        ("username", step.username),
        ("password", step.password),
        ("captcha", step.captcha),
        ("verify", step.verify),
    ]);
    let reply = device
        .config
        .relay()
        .exchange(
            Method::POST,
            "/api/xiaomi".into(),
            moli_xiaomi::cloud::form(&form).into_bytes(),
        )
        .await;
    let Ok((status, bytes)) = reply else {
        *attempt = None;
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    *attempt = None;
    if status == StatusCode::UNAUTHORIZED {
        let Ok(v) = serde_json::from_slice::<Value>(&bytes) else {
            return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
        };
        let mut challenge = json!({});
        for key in ["captcha", "verify_phone", "verify_email"] {
            if let Some(s) = v[key].as_str().filter(|s| s.len() <= 512 * 1024) {
                challenge[key] = json!(s);
            }
        }
        let phase = if challenge.get("captcha").is_some() {
            "captcha"
        } else if challenge.get("verify_email").is_some() || challenge.get("verify_phone").is_some()
        {
            "verify"
        } else {
            return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
        };
        *attempt = Some(Attempt {
            owner: caller.key.clone(),
            until: Instant::now() + Duration::from_secs(600),
            phase,
        });
        return Json(json!({"challenge":challenge})).into_response();
    }
    if !status.is_success() {
        *attempt = None;
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    }
    *attempt = None;
    finish_login(&hub, &setup, &device.config.relay()).await
}
async fn finish_login(hub: &Hub, setup: &Setup, relay: &moli_tapo_camera::Relay) -> Response {
    let (Ok(users), Ok(config)) = (
        relay.request(Method::GET, "/api/xiaomi".into()).await,
        relay.request(Method::GET, "/api/config".into()).await,
    ) else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    let Ok(users) = serde_json::from_slice::<Vec<String>>(&users) else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    let Ok(config) = String::from_utf8(config) else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    if users.len() != 1 || config.len() > 65536 {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "xiaomi.account.invalid");
    }
    let result = register(hub, setup, &users[0], &config).await;
    if result.status().is_success() {
        // Clear go2rtc's cached cloud session after reauthentication.
        let _ = relay
            .exchange(Method::POST, "/api/restart".into(), Vec::new())
            .await;
    }
    result
}

async fn register(hub: &Hub, setup: &Setup, user: &str, config: &str) -> Response {
    let Some(pass) = moli_xiaomi::cloud::pass_token(config, user) else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    let Ok(cloud) = moli_xiaomi::cloud::Cloud::login(user, &pass, "de").await else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    let Ok(devices) = cloud.devices().await else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.account.failed");
    };
    let mut results = Vec::new();
    for configured in setup.devices.iter() {
        let expected = match configured.kind {
            Kind::Camera => "isa.camera.hlc7",
            Kind::Vacuum => "rockrobo.vacuum.v1",
        };
        let found: Vec<_> = devices
            .iter()
            .filter(|d| {
                d["localip"].as_str() == Some(&configured.config.host.to_string())
                    && d["model"].as_str() == Some(expected)
            })
            .take(2)
            .collect();
        let [device] = found.as_slice() else {
            results.push(json!({"instance":configured.instance,"configured":false,"error":moli_i18n::tr!("xiaomi.account.not_found")}));
            continue;
        };
        let previous = hub
            .secret(&configured.instance.as_str().into(), ACCOUNT)
            .and_then(|s| serde_json::from_str::<Account>(&s).ok());
        let account = Account {
            user: user.to_owned(),
            region: "de".into(),
            device: device["did"].as_str().unwrap_or_default().to_owned(),
            model: expected.into(),
            relay_config: format!("xiaomi:\n  \"{user}\": \"{pass}\"\n"),
            token: if configured.kind == Kind::Vacuum {
                device["token"].as_str().map(str::to_owned)
            } else {
                None
            },
        };
        if !account.valid(configured.kind) {
            results.push(json!({"instance":configured.instance,"configured":false,"error":moli_i18n::tr!("xiaomi.account.invalid")}));
            continue;
        }
        let mut info = json!({});
        if configured.kind == Kind::Vacuum {
            if let Ok(v) = Vacuum::verify(&configured.config, &account).await {
                info = v;
            } else {
                results.push(json!({"instance":configured.instance,"configured":false,"error":moli_i18n::tr!("xiaomi.vacuum.unavailable")}));
                continue;
            }
        }
        if configured.kind == Kind::Vacuum
            && previous
                .as_ref()
                .is_some_and(|a| a.user != account.user || a.device != account.device)
            && hub
                .store_secret(&configured.instance.as_str().into(), ZONE, "")
                .await
                .is_err()
        {
            return fail(StatusCode::INTERNAL_SERVER_ERROR, "xiaomi.account.storage");
        }
        let Ok(secret) = serde_json::to_string(&account) else {
            return fail(StatusCode::INTERNAL_SERVER_ERROR, "xiaomi.account.storage");
        };
        if hub
            .store_secret(&configured.instance.as_str().into(), ACCOUNT, &secret)
            .await
            .is_err()
        {
            return fail(StatusCode::INTERNAL_SERVER_ERROR, "xiaomi.account.storage");
        }
        results.push(json!({"instance":configured.instance,"configured":true,"info":info}));
    }
    Json(json!({"devices":results})).into_response()
}
async fn discover(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
) -> Response {
    if !owner(&caller, &auth, &headers) {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    for d in setup.devices.iter() {
        if let Some(a) = hub
            .secret(&d.instance.as_str().into(), ACCOUNT)
            .and_then(|s| serde_json::from_str::<Account>(&s).ok())
        {
            return register(&hub, &setup, &a.user, &a.relay_config).await;
        }
    }
    fail(StatusCode::CONFLICT, "xiaomi.account.wait")
}
async fn map(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Path(instance): Path<String>,
) -> Response {
    if !owner(&caller, &auth, &headers) {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    let Some(d) = setup
        .devices
        .iter()
        .find(|d| d.instance == instance && d.kind == Kind::Vacuum)
    else {
        return fail(StatusCode::NOT_FOUND, "xiaomi.account.not_found");
    };
    match fetch_map(&hub, d).await {
        Ok(m) => Json(m).into_response(),
        Err(error) => fail(
            StatusCode::BAD_GATEWAY,
            moli_xiaomi::map::failure_key(&error),
        ),
    }
}
async fn fetch_map(hub: &Hub, d: &Configured) -> anyhow::Result<moli_xiaomi::map::Map> {
    let secret = hub
        .secret(&d.instance.as_str().into(), ACCOUNT)
        .ok_or_else(|| anyhow::anyhow!("missing vacuum account"))?;
    let account: Account = serde_json::from_str(&secret)?;
    anyhow::ensure!(account.valid(Kind::Vacuum), "invalid vacuum account");
    let mut client = moli_xiaomi::miio::Miio::connect(
        d.config.host,
        d.config.port,
        account.token.as_deref().unwrap_or_default(),
    )
    .await
    .context("map_connect")?;
    moli_xiaomi::map::current(&d.config, &account, &mut client).await
}
async fn zone(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Path(instance): Path<String>,
) -> Response {
    if !owner(&caller, &auth, &headers) {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    if !setup
        .devices
        .iter()
        .any(|d| d.instance == instance && d.kind == Kind::Vacuum)
    {
        return fail(StatusCode::NOT_FOUND, "xiaomi.account.not_found");
    }
    let zone = hub
        .secret(&instance.as_str().into(), ZONE)
        .and_then(|s| serde_json::from_str::<Zone>(&s).ok());
    Json(json!({"zone":zone})).into_response()
}
async fn save_zone(
    State(hub): State<Hub>,
    Extension(setup): Extension<Setup>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Path(instance): Path<String>,
    Json(mut zone): Json<Zone>,
) -> Response {
    if !owner(&caller, &auth, &headers) {
        return fail(StatusCode::FORBIDDEN, "xiaomi.account.owner");
    }
    let Some(d) = setup
        .devices
        .iter()
        .find(|d| d.instance == instance && d.kind == Kind::Vacuum)
    else {
        return fail(StatusCode::NOT_FOUND, "xiaomi.account.not_found");
    };
    if !zone.valid() {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "xiaomi.zone.invalid");
    }
    let Ok(current) = fetch_map(&hub, d).await else {
        return fail(StatusCode::BAD_GATEWAY, "xiaomi.zone.map_failed");
    };
    if !current.matches(&zone) {
        return fail(StatusCode::CONFLICT, "xiaomi.zone.changed");
    }
    zone.confirmed_at = moli_core::now_ms();
    let Ok(secret) = serde_json::to_string(&zone) else {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "xiaomi.account.storage");
    };
    if hub
        .store_secret(&instance.as_str().into(), ZONE, &secret)
        .await
        .is_err()
    {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "xiaomi.account.storage");
    }
    Json(json!({"saved":true})).into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn challenge_is_bound_to_owner_phase_and_expiry() {
        let mut attempt = Attempt {
            owner: "owner-fixture".into(),
            until: Instant::now() + Duration::from_secs(60),
            phase: "captcha",
        };
        let captcha = Login {
            captcha: "code".into(),
            ..Login::default()
        };
        let verify = Login {
            verify: "123456".into(),
            ..Login::default()
        };
        assert!(continuation(Some(&attempt), "owner-fixture", &captcha));
        assert!(!continuation(Some(&attempt), "another-owner", &captcha));
        assert!(!continuation(Some(&attempt), "owner-fixture", &verify));
        attempt.phase = "verify";
        assert!(continuation(Some(&attempt), "owner-fixture", &verify));
        attempt.until = Instant::now();
        assert!(!continuation(Some(&attempt), "owner-fixture", &verify));
        assert!(!continuation(None, "owner-fixture", &captcha));
    }
    #[test]
    fn login_steps_do_not_mix_passwords_and_challenges() {
        assert!(check_step(&Login {
            username: "user@example.com".into(),
            password: "fixture".into(),
            ..Login::default()
        }));
        assert!(!check_step(&Login {
            password: "fixture".into(),
            ..Login::default()
        }));
        assert!(!check_step(&Login {
            captcha: "123".into(),
            verify: "456".into(),
            ..Login::default()
        }));
        assert!(!check_step(&Login {
            verify: "x".repeat(10000),
            ..Login::default()
        }));
    }
}
