use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
}

fn session_cookie(token: &str) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        db::auth::SESSION_COOKIE,
        token,
        30 * 24 * 3600
    )
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Response, ApiError> {
    let token = db::auth::register(&state.pool, &body.username, &body.password)
        .await
        .map_err(|msg| ApiError::bad_request(msg))?;
    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, session_cookie(&token))],
        Json(serde_json::json!({ "ok": true })),
    )
        .into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Response, ApiError> {
    let enabled = db::auth::auth_enabled(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !enabled {
        return Err(ApiError::bad_request(
            "no account yet — create one on the setup screen",
        ));
    }
    let token = db::auth::login(&state.pool, &body.username, &body.password)
        .await
        .map_err(|_| ApiError::unauthorized("invalid credentials"))?;
    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, session_cookie(&token))],
        Json(serde_json::json!({ "ok": true })),
    )
        .into_response())
}

pub async fn logout(State(state): State<AppState>) -> Result<Response, ApiError> {
    db::auth::logout(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let cookie = format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0", db::auth::SESSION_COOKIE);
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, cookie)],
    )
        .into_response())
}

pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let enabled = db::auth::auth_enabled(&state.pool)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let needs_setup = !enabled;
    let authenticated = if needs_setup {
        false
    } else if let Some(token) = session_from_headers(&headers) {
        db::auth::session_valid(&state.pool, &token)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?
    } else {
        false
    };
    Ok(Json(serde_json::json!({
        "auth_enabled": enabled,
        "needs_setup": needs_setup,
        "authenticated": authenticated,
    })))
}

pub fn session_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|part| {
            part.strip_prefix(&format!("{}=", db::auth::SESSION_COOKIE))
                .map(str::to_string)
        })
}
