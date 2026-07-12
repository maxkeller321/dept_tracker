use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::routes::auth::session_from_headers;
use crate::AppState;

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if path == "/health"
        || path == "/auth/login"
        || path == "/auth/register"
        || path == "/auth/status"
    {
        return next.run(request).await;
    }

    let enabled = match db::auth::auth_enabled(&state.pool).await {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal error" })),
            )
                .into_response();
        }
    };

    if !enabled {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "create an account first" })),
        )
            .into_response();
    }

    let token = session_from_headers(request.headers());
    let ok = match token {
        Some(t) => db::auth::session_valid(&state.pool, &t).await.unwrap_or(false),
        None => false,
    };

    if ok {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "authentication required" })),
        )
            .into_response()
    }
}
