//! Daily water imports. The encrypted core store also includes these readings in backups.
use crate::water_suez::{self, Account};
use crate::{caller::Caller, people::People, session::Sessions};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use moli_runtime::Hub;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, Instant};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const SECRET: &str = "water_history";
const MAX_DAYS: usize = 3660;
#[derive(Default, Serialize, Deserialize)]
struct History {
    imported_at: u64,
    #[serde(default)]
    synchronized_at: u64,
    #[serde(default)]
    meter: Option<String>,
    #[serde(default)]
    backfilled: bool,
    days: BTreeMap<String, f64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import {
    csv: String,
    unit: String,
}
fn fail(status: StatusCode, key: &str) -> Response {
    (status, Json(json!({"error":moli_i18n::tr(key)}))).into_response()
}
fn read(hub: &Hub) -> anyhow::Result<History> {
    hub.core_secret(SECRET).map_or_else(
        || Ok(History::default()),
        |text| Ok(serde_json::from_str(&text)?),
    )
}
pub(super) fn date(text: &str) -> bool {
    text.len() == 10 && text.parse::<jiff::civil::Date>().is_ok()
}
fn fields(line: &str) -> Vec<&str> {
    line.split(';')
        .map(|s| s.trim().trim_matches('"'))
        .collect()
}
fn parse(input: &Import) -> anyhow::Result<BTreeMap<String, f64>> {
    anyhow::ensure!(input.csv.len() <= 256 * 1024, "water import size");
    let divisor = match input.unit.as_str() {
        "m3" => 1.0,
        "L" => 1000.0,
        _ => anyhow::bail!("water unit"),
    };
    let mut lines = input.csv.trim_start_matches('\u{feff}').lines();
    anyhow::ensure!(
        lines.next().map(fields) == Some(vec!["Date", "Consommation", "Index"]),
        "water daily header"
    );
    let mut days = BTreeMap::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let columns = fields(line);
        anyhow::ensure!(columns.len() == 3 && date(columns[0]), "water daily row");
        let volume: f64 = columns[1].replace(',', ".").parse()?;
        let index: f64 = columns[2].replace(',', ".").parse()?;
        anyhow::ensure!(
            volume.is_finite()
                && index.is_finite()
                && volume >= 0.0
                && volume / divisor <= 1_000_000.0
                && index >= 0.0,
            "water invalid number"
        );
        // The provider exports unavailable dates as volume=0,index=0, not a measured zero.
        if index == 0.0 {
            anyhow::ensure!(volume == 0.0, "water missing index");
            continue;
        }
        anyhow::ensure!(
            days.insert(columns[0].to_owned(), volume / divisor)
                .is_none(),
            "water duplicate date"
        );
        anyhow::ensure!(days.len() <= MAX_DAYS, "water too many days");
    }
    anyhow::ensure!(!days.is_empty(), "water no readings");
    Ok(days)
}
pub(crate) fn routes(hub: Hub, shutdown: CancellationToken) -> Router<Hub> {
    let writing = Arc::new(Mutex::new(()));
    let service = Service {
        state: Arc::new(Mutex::new(SyncState::default())),
        writing: writing.clone(),
    };
    let worker = service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(12 * 60 * 60));
        loop {
            tokio::select! {
                () = shutdown.cancelled() => break,
                _ = interval.tick() => {
                    retry_poll(&shutdown, || async {
                        let Some(account) = account(&hub) else { return Ok(()); };
                        synchronize(&hub, &worker, account, false).await
                    }).await;
                }
            }
        }
    });
    Router::new()
        .route("/api/water", get(history).put(import))
        .route("/api/water/account", get(account_status).post(connect))
        .route("/api/water/sync", post(refresh))
        .layer(Extension(writing))
        .layer(Extension(service))
}
// One transient login failure must not postpone the next reading for twelve hours.
async fn retry_poll<F, Fut>(shutdown: &CancellationToken, mut fetch: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = water_suez::Result<()>>,
{
    for attempt in 0..2 {
        let result = tokio::select! {
            () = shutdown.cancelled() => return,
            result = fetch() => result,
        };
        if matches!(result, Ok(()) | Err("water.human_required" | "water.wait")) {
            return;
        }
        if attempt == 0 {
            tokio::select! {
                () = shutdown.cancelled() => return,
                () = tokio::time::sleep(Duration::from_secs(60)) => {}
            }
        }
    }
}
async fn history(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    caller: Caller,
) -> Response {
    if caller
        .person
        .as_ref()
        .is_some_and(crate::people::PersonRef::is_guest)
    {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    match read(&hub) {
        Ok(history) => Json(public_history(&history, &hub, &service).await).into_response(),
        Err(_) => fail(StatusCode::INTERNAL_SERVER_ERROR, "water.failed"),
    }
}
async fn import(
    State(hub): State<Hub>,
    Extension(writing): Extension<Arc<Mutex<()>>>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Json(input): Json<Import>,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    let Ok(days) = parse(&input) else {
        return fail(StatusCode::BAD_REQUEST, "water.invalid");
    };
    let _lock = writing.lock().await;
    let Ok(mut history) = read(&hub) else {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "water.failed");
    };
    history.days.extend(days);
    if history.days.len() > MAX_DAYS {
        return fail(StatusCode::BAD_REQUEST, "water.invalid");
    }
    history.imported_at = moli_core::now_ms();
    let Ok(text) = serde_json::to_string(&history) else {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "water.failed");
    };
    if hub.store_core_secret(SECRET, &text).await.is_err() {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "water.failed");
    }
    Json(json!({"days":history.days,"imported_at":history.imported_at,"synchronized_at":history.synchronized_at})).into_response()
}
const ACCOUNT: &str = "water_suez_account";
#[derive(Default)]
struct SyncState {
    attempted: Option<Instant>,
    error: Option<&'static str>,
}
#[derive(Clone)]
struct Service {
    state: Arc<Mutex<SyncState>>,
    writing: Arc<Mutex<()>>,
}
fn account(hub: &Hub) -> Option<Account> {
    hub.core_secret(ACCOUNT)
        .and_then(|s| serde_json::from_str(&s).ok())
}
async fn public_history(history: &History, hub: &Hub, service: &Service) -> serde_json::Value {
    let state = service.state.lock().await;
    json!({"days":history.days,"imported_at":history.imported_at,"synchronized_at":history.synchronized_at,
        "sync":{"configured":account(hub).is_some(),"error":state.error.map(moli_i18n::tr)}})
}
async fn account_status(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    match read(&hub) {
        Ok(history) => {
            let state = service.state.lock().await;
            Json(json!({"configured":account(&hub).is_some(),"synchronized_at":history.synchronized_at,"error":state.error.map(moli_i18n::tr)})).into_response()
        }
        Err(_) => fail(StatusCode::INTERNAL_SERVER_ERROR, "water.failed"),
    }
}
async fn synchronize(
    hub: &Hub,
    service: &Service,
    mut account: Account,
    save: bool,
) -> water_suez::Result<()> {
    let mut state = service.state.lock().await;
    if state
        .attempted
        .is_some_and(|at| at.elapsed() < Duration::from_secs(60))
    {
        return Err("water.wait");
    }
    state.attempted = Some(Instant::now());
    let result = async {
        let backfill = !read(hub).map_err(|_| "water.failed")?.backfilled;
        let (meter, days) = water_suez::fetch(&account, backfill).await?;
        let _writing = service.writing.lock().await;
        let mut history = read(hub).map_err(|_| "water.failed")?;
        if history.meter.as_ref().is_some_and(|old| old != &meter) {
            return Err("water.meter_changed");
        }
        history.days.extend(days);
        if history.days.len() > MAX_DAYS {
            return Err("water.invalid");
        }
        history.meter = Some(meter.clone());
        history.backfilled = true;
        history.synchronized_at = moli_core::now_ms();
        let text = serde_json::to_string(&history).map_err(|_| "water.failed")?;
        hub.store_core_secret(SECRET, &text)
            .await
            .map_err(|_| "water.failed")?;
        if save {
            account.meter = meter;
            let text = serde_json::to_string(&account).map_err(|_| "water.failed")?;
            hub.store_core_secret(ACCOUNT, &text)
                .await
                .map_err(|_| "water.failed")?;
        }
        Ok(())
    }
    .await;
    state.error = result.as_ref().err().copied();
    result
}
async fn connect(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Json(input): Json<Account>,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    if !input.valid() {
        return fail(StatusCode::BAD_REQUEST, "water.account_invalid");
    }
    match synchronize(&hub, &service, input, true).await {
        Ok(()) => Json(json!({"configured":true})).into_response(),
        Err(key) => fail(
            if key == "water.wait" {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_GATEWAY
            },
            key,
        ),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Refresh {}
async fn refresh(
    State(hub): State<Hub>,
    Extension(service): Extension<Service>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Json(_): Json<Refresh>,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    let Some(account) = account(&hub) else {
        return fail(StatusCode::BAD_REQUEST, "water.account_invalid");
    };
    match synchronize(&hub, &service, account, false).await {
        Ok(()) => Json(json!({"configured":true})).into_response(),
        Err(key) => fail(
            if key == "water.wait" {
                StatusCode::TOO_MANY_REQUESTS
            } else {
                StatusCode::BAD_GATEWAY
            },
            key,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn polling_retries_once_after_a_minute_but_never_retries_a_human_challenge() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = AtomicUsize::new(0);
        let started = tokio::time::Instant::now();
        retry_poll(&CancellationToken::new(), || async {
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                Err("water.login_failed")
            } else {
                Ok(())
            }
        })
        .await;
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(started.elapsed(), Duration::from_secs(60));
        calls.store(0, Ordering::SeqCst);
        retry_poll(&CancellationToken::new(), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err("water.human_required")
        })
        .await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        calls.store(0, Ordering::SeqCst);
        retry_poll(&CancellationToken::new(), || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err("water.connection_failed")
        })
        .await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "a persistent failure is bounded too"
        );
    }
    fn input(csv: &str) -> Import {
        Import {
            csv: csv.into(),
            unit: "m3".into(),
        }
    }
    #[test]
    fn daily_export_distinguishes_missing_from_zero_and_preserves_units() {
        let mut data = input(
            "\u{feff}\"Date\";\"Consommation\";\"Index\"\r\n\"2026-11-01\";\"0.125\";\"100.125\"\r\n2026-11-02;0;100.125\n2026-11-03;0;0\n",
        );
        let parsed = parse(&data).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed["2026-11-01"], 0.125);
        assert_eq!(parsed["2026-11-02"], 0.0);
        data.unit = "L".into();
        assert_eq!(parse(&data).unwrap()["2026-11-01"], 0.000_125);
    }
    #[test]
    fn malformed_monthly_duplicate_and_nonfinite_imports_fail_atomically() {
        for row in [
            "2026-02-30;1;10",
            "2026-11;1;10",
            "2026-11-01;NaN;10",
            "2026-11-01;-1;10",
            "2026-11-01;1;0",
            "2026-11-01;1;10\n2026-11-01;2;12",
        ] {
            assert!(parse(&input(&format!("Date;Consommation;Index\n{row}"))).is_err());
        }
        assert!(parse(&input("Date;Consommation;Index\n2026-11-01;0;0")).is_err());
    }
}
