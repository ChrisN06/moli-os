//! One persistent « Perso » look per set of lamps. Capture is owner-only;
//! recall never bypasses the command guard or a guest's room boundaries.
use crate::{caller::Caller, people::People, session::Sessions};
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use moli_core::{DeviceId, PointId, Semantic, Value};
use moli_runtime::{CommandError, DeviceView, Hub};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone)]
struct Store {
    path: Option<PathBuf>,
    writing: Arc<Mutex<()>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Targets {
    lights: Vec<DeviceId>,
}
#[derive(Deserialize)]
struct Scope {
    lights: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Lamp {
    device: DeviceId,
    power: (String, Value),
    settings: Vec<(String, Value)>,
}
type Presets = BTreeMap<String, Vec<Lamp>>;
fn fail(status: StatusCode, key: &str) -> Response {
    (status, Json(json!({"error":moli_i18n::tr(key)}))).into_response()
}
fn scope(ids: &[DeviceId]) -> Option<String> {
    if ids.is_empty() || ids.len() > 32 {
        return None;
    }
    let mut ids = ids.to_vec();
    ids.sort();
    ids.dedup();
    serde_json::to_string(&ids).ok()
}
fn capture(view: &DeviceView) -> Option<Lamp> {
    moli_ambiance::Lamp::of(view)?;
    if view.online == Some(false) || !view.device.members.is_empty() {
        return None;
    }
    let power = view
        .device
        .points
        .iter()
        .find(|p| p.semantic == Semantic::OnOff && p.access.write)?;
    let value = &view.state.get(&power.key)?.value;
    if *value == Value::Null {
        return None;
    }
    let mut settings = Vec::new();
    let white = view
        .state
        .get("color_temp")
        .is_some_and(|s| s.value != Value::Null);
    for key in [
        if white { "color_temp" } else { "color" },
        "saturation",
        "brightness",
    ] {
        if key == "saturation" && white {
            continue;
        }
        if view.device.point(key).is_some_and(|p| p.access.write)
            && let Some(sample) = view.state.get(key)
            && sample.value != Value::Null
        {
            settings.push((key.to_owned(), sample.value.clone()));
        }
    }
    Some(Lamp {
        device: view.device.id.clone(),
        power: (power.key.to_string(), value.clone()),
        settings,
    })
}
impl Lamp {
    fn orders(&self) -> Vec<(PointId, Value)> {
        let mut out = vec![(
            PointId::new(&self.device, &self.power.0),
            self.power.1.clone(),
        )];
        let off = matches!(&self.power.1, Value::Bool(false))
            || matches!(&self.power.1,Value::Text(s) if s.eq_ignore_ascii_case("off"));
        // Parameter writes can switch radio lamps on: never flash a saved-off lamp.
        if !off {
            out.extend(
                self.settings
                    .iter()
                    .map(|(k, v)| (PointId::new(&self.device, k), v.clone())),
            );
        }
        out
    }
}
#[allow(clippy::result_large_err)]
async fn read(store: &Store) -> Result<Presets, Response> {
    let Some(path) = &store.path else {
        return Err(fail(StatusCode::SERVICE_UNAVAILABLE, "preset.storage"));
    };
    match tokio::fs::read(path).await {
        Ok(b) => serde_json::from_slice(&b)
            .map_err(|_| fail(StatusCode::INTERNAL_SERVER_ERROR, "preset.storage")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(_) => Err(fail(StatusCode::INTERNAL_SERVER_ERROR, "preset.storage")),
    }
}
pub(crate) fn routes(home: Option<&std::path::Path>) -> Router<Hub> {
    Router::new()
        .route("/api/light-preset", get(status).put(save).post(recall))
        .layer(Extension(Store {
            path: home
                .and_then(std::path::Path::parent)
                .map(|p| p.join("light-presets.json")),
            writing: Arc::new(Mutex::new(())),
        }))
}
async fn status(
    State(hub): State<Hub>,
    Extension(store): Extension<Store>,
    caller: Caller,
    Query(query): Query<Scope>,
) -> Response {
    let Ok(ids) = serde_json::from_str::<Vec<DeviceId>>(&query.lights) else {
        return fail(StatusCode::BAD_REQUEST, "preset.invalid");
    };
    let Some(key) = scope(&ids) else {
        return fail(StatusCode::BAD_REQUEST, "preset.invalid");
    };
    for id in &ids {
        if let Err(no) = crate::rest::guest_may(&hub, &caller, id) {
            return no;
        }
    }
    match read(&store).await {
        Ok(p) => Json(json!({"saved":p.contains_key(&key)})).into_response(),
        Err(e) => e,
    }
}
async fn save(
    State(hub): State<Hub>,
    Extension(store): Extension<Store>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Json(body): Json<Targets>,
) -> Response {
    if !crate::people_api::is_owner(&caller, &auth.0.0, &auth.1.0, &headers) {
        return fail(StatusCode::FORBIDDEN, "preset.owner");
    }
    let Some(key) = scope(&body.lights) else {
        return fail(StatusCode::BAD_REQUEST, "preset.invalid");
    };
    let mut ids = body.lights;
    ids.sort();
    ids.dedup();
    let lamps: Option<Vec<_>> = ids
        .iter()
        .map(|id| hub.device(id).and_then(|v| capture(&v)))
        .collect();
    let Some(lamps) = lamps else {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "preset.unknown");
    };
    let _writing = store.writing.lock().await;
    let mut presets = match read(&store).await {
        Ok(p) => p,
        Err(e) => return e,
    };
    if presets.len() >= 64 && !presets.contains_key(&key) {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "preset.invalid");
    }
    presets.insert(key, lamps);
    let Some(path) = &store.path else {
        return fail(StatusCode::SERVICE_UNAVAILABLE, "preset.storage");
    };
    let Ok(bytes) = serde_json::to_vec(&presets) else {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "preset.storage");
    };
    let tmp = path.with_extension("tmp");
    let written = async {
        tokio::fs::write(&tmp, bytes).await?;
        tokio::fs::rename(tmp, path).await
    }
    .await;
    match written {
        Ok(()) => Json(json!({"saved":true})).into_response(),
        Err(_) => fail(StatusCode::INTERNAL_SERVER_ERROR, "preset.storage"),
    }
}
async fn recall(
    State(hub): State<Hub>,
    Extension(store): Extension<Store>,
    Extension(humans): Extension<Arc<Sessions>>,
    caller: Caller,
    headers: HeaderMap,
    Json(body): Json<Targets>,
) -> Response {
    let Some(key) = scope(&body.lights) else {
        return fail(StatusCode::BAD_REQUEST, "preset.invalid");
    };
    let presets = match read(&store).await {
        Ok(p) => p,
        Err(e) => return e,
    };
    let Some(lamps) = presets.get(&key) else {
        return fail(StatusCode::NOT_FOUND, "preset.missing");
    };
    if scope(&lamps.iter().map(|l| l.device.clone()).collect::<Vec<_>>()).as_ref() != Some(&key) {
        return fail(StatusCode::INTERNAL_SERVER_ERROR, "preset.storage");
    }
    for lamp in lamps {
        if let Err(no) = crate::rest::guest_may(&hub, &caller, &lamp.device) {
            return no;
        }
    }
    let origin = crate::rest::origin(&hub, &headers, &humans, &caller);
    let mut report = moli_ambiance::Report::default();
    for lamp in lamps {
        let mut done = true;
        for (point, value) in lamp.orders() {
            if let Err(e) = hub
                .command(&point, value, origin, Some(caller.describe()))
                .await
            {
                done = false;
                match e {
                    CommandError::NeedsApproval { id, reason } => {
                        report.held.push(moli_ambiance::Held {
                            device: lamp.device.clone(),
                            point,
                            approval: id,
                            reason,
                        });
                    }
                    e => report.failed.push(moli_ambiance::Failed {
                        device: lamp.device.clone(),
                        error: e.to_string(),
                    }),
                }
                break;
            }
        }
        if done {
            report.done.push(lamp.device.clone());
        }
    }
    Json(report).into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scopes_ignore_order_and_saved_off_does_not_flash() {
        let a = DeviceId::from("test:a");
        let b = DeviceId::from("test:b");
        assert_eq!(scope(&[a.clone(), b.clone()]), scope(&[b, a.clone()]));
        assert!(scope(&[]).is_none());
        let mut lamp = Lamp {
            device: a,
            power: ("on".into(), Value::Bool(false)),
            settings: vec![("brightness".into(), Value::Int(40))],
        };
        assert_eq!(lamp.orders().len(), 1);
        lamp.power.1 = Value::Bool(true);
        assert_eq!(lamp.orders().len(), 2);
    }
}
