//! Camera accounts are owner-only writes; media keeps snapshot access rules.
use crate::{caller::Caller, people::People, session::Sessions};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use moli_core::{DeviceId, InstanceId};
use moli_runtime::Hub;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Configured(pub Vec<(String, moli_tapo_camera::Config)>);
fn error(status: StatusCode, key: &str) -> Response {
    (status, Json(json!({"error":moli_i18n::tr(key)}))).into_response()
}

pub(crate) async fn accounts(
    State(hub): State<Hub>,
    Extension(Configured(configs)): Extension<Configured>,
    caller: Caller,
) -> Response {
    if caller
        .person
        .as_ref()
        .is_some_and(crate::people::PersonRef::is_guest)
    {
        return error(StatusCode::FORBIDDEN, "serveur.personnes.invite_limite");
    }
    Json(json!({"cameras":configs.iter().map(|(instance, c)| json!({"instance":instance,"model":c.model,"host":c.host.to_string(),"configured":hub.secret(&InstanceId::from(instance.as_str()),moli_tapo_camera::ACCOUNT).is_some()})).collect::<Vec<_>>()})).into_response()
}

pub(crate) async fn account(
    State(hub): State<Hub>,
    Extension(Configured(configs)): Extension<Configured>,
    auth: (Extension<Arc<Sessions>>, Extension<Arc<People>>),
    caller: Caller,
    headers: HeaderMap,
    Path(instance): Path<String>,
    Json(account): Json<moli_tapo_camera::Account>,
) -> Response {
    let (Extension(humans), Extension(persons)) = auth;
    if !crate::people_api::is_owner(&caller, &humans, &persons, &headers) {
        return error(StatusCode::FORBIDDEN, "camera.account.owner");
    }
    let Some((_, config)) = configs.iter().find(|(i, _)| i == &instance) else {
        return error(StatusCode::NOT_FOUND, "camera.account.unknown");
    };
    if !account.valid() {
        return error(StatusCode::UNPROCESSABLE_ENTITY, "camera.account.invalid");
    }
    if config.verify(&instance, &account).await.is_err() {
        return error(StatusCode::BAD_GATEWAY, "camera.account.failed");
    }
    let Ok(secret) = serde_json::to_string(&account) else {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "camera.account.failed");
    };
    if hub
        .store_secret(
            &instance.as_str().into(),
            moli_tapo_camera::ACCOUNT,
            &secret,
        )
        .await
        .is_err()
    {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "camera.account.storage");
    }
    Json(json!({"stored":true})).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VideoQuery {
    id: Option<String>,
    n: Option<u64>,
}

pub(crate) async fn video(
    State(hub): State<Hub>,
    Extension(humans): Extension<Arc<Sessions>>,
    caller: Caller,
    headers: HeaderMap,
    Path((id, resource)): Path<(String, String)>,
    Query(q): Query<VideoQuery>,
) -> Response {
    let id = DeviceId::from(id);
    if caller
        .person
        .as_ref()
        .is_some_and(crate::people::PersonRef::is_guest)
    {
        return error(StatusCode::FORBIDDEN, "serveur.personnes.invite_limite");
    }
    if hub.protected_room(&id).is_some() && !humans.is_human(&headers, &caller.key) {
        return error(StatusCode::FORBIDDEN, "camera.video.protected");
    }
    let _ = q.n;
    match hub.camera_video(&id, &resource, q.id.as_deref()).await {
        Ok(file) => (
            [
                ("content-type", file.content_type),
                ("cache-control", "no-store"),
                ("x-content-type-options", "nosniff"),
            ],
            file.bytes,
        )
            .into_response(),
        Err(_) => error(StatusCode::BAD_GATEWAY, "camera.video.failed"),
    }
}

pub(crate) fn routes(configs: Vec<(String, moli_tapo_camera::Config)>) -> axum::Router<Hub> {
    use axum::{
        extract::DefaultBodyLimit,
        routing::{get, put},
    };
    axum::Router::new()
        .route("/api/cameras/accounts", get(accounts))
        .route(
            "/api/cameras/{instance}/account",
            put(account).layer(DefaultBodyLimit::max(4096)),
        )
        .route("/api/devices/{id}/video/{*resource}", get(video))
        .layer(Extension(Configured(configs)))
}
