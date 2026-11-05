//! Daily water imports. The encrypted core store also includes these readings in backups.
use crate::{caller::Caller, people::People, session::Sessions};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use moli_runtime::Hub;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

const SECRET: &str = "water_history";
const MAX_DAYS: usize = 3660;
#[derive(Default, Serialize, Deserialize)]
struct History {
    imported_at: u64,
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
fn date(text: &str) -> bool {
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
pub(crate) fn routes() -> Router<Hub> {
    Router::new()
        .route("/api/water", get(history).put(import))
        .layer(Extension(Arc::new(Mutex::new(()))))
}
async fn history(State(hub): State<Hub>, caller: Caller) -> Response {
    if caller
        .person
        .as_ref()
        .is_some_and(crate::people::PersonRef::is_guest)
    {
        return fail(StatusCode::FORBIDDEN, "water.owner");
    }
    match read(&hub) {
        Ok(history) => Json(history).into_response(),
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
    Json(history).into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
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
