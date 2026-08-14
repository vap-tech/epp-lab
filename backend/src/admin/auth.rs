use std::future::Future;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{
        HeaderMap, StatusCode,
        header::{COOKIE, SET_COOKIE},
        request::Parts,
    },
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::{app::AppState, storage::admin};

const SESSION_COOKIE: &str = "__Host-epp_lab_session";

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub(crate) struct SessionResponse {
    pub authenticated: bool,
    pub user: Option<AdminUserResponse>,
    pub csrf_token: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AdminUserResponse {
    pub id: Uuid,
    pub username: String,
}

pub(crate) async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<
    (
        StatusCode,
        [(axum::http::header::HeaderName, String); 1],
        Json<SessionResponse>,
    ),
    StatusCode,
> {
    let user = admin::find_active_user(&state.db, &request.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let valid = user.as_ref().is_some_and(|user| {
        PasswordHash::new(&user.password_hash)
            .ok()
            .and_then(|hash| {
                Argon2::default()
                    .verify_password(request.password.as_bytes(), &hash)
                    .ok()
            })
            .is_some()
    });
    let Some(user) = user.filter(|_| valid) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let token = random_token();
    let csrf = random_token();
    let now = Utc::now();
    admin::create_session(
        &state.db,
        user.id,
        &hash(&token),
        &hash(&csrf),
        now,
        now + Duration::hours(12),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cookie = format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Secure");
    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie)],
        Json(SessionResponse {
            authenticated: true,
            user: Some(AdminUserResponse {
                id: user.id,
                username: user.username,
            }),
            csrf_token: Some(csrf),
        }),
    ))
}

pub(crate) async fn session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, StatusCode> {
    let Some(token) = cookie_value(&headers, SESSION_COOKIE) else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            user: None,
            csrf_token: None,
        }));
    };
    let Some(session) = admin::find_session(&state.db, &hash(token), Utc::now())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            user: None,
            csrf_token: None,
        }));
    };
    let csrf = random_token();
    admin::rotate_csrf(&state.db, session.id, &hash(&csrf), Utc::now())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(SessionResponse {
        authenticated: true,
        user: Some(AdminUserResponse {
            id: session.admin_user_id,
            username: session.username,
        }),
        csrf_token: Some(csrf),
    }))
}

pub(crate) async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<([(axum::http::header::HeaderName, String); 1], StatusCode), StatusCode> {
    if let Some(token) = cookie_value(&headers, SESSION_COOKIE) {
        admin::revoke(&state.db, &hash(token), Utc::now())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok((
        [(
            SET_COOKIE,
            format!("{SESSION_COOKIE}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict; Secure"),
        )],
        StatusCode::NO_CONTENT,
    ))
}

pub(crate) struct AdminSession {
    _user_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AdminSession {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let token = cookie_value(&parts.headers, SESSION_COOKIE).map(str::to_owned);
        let state = Arc::clone(state);
        async move {
            let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
            let session = admin::find_session(&state.db, &hash(&token), Utc::now())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::UNAUTHORIZED)?;
            Ok(Self {
                _user_id: session.admin_user_id,
            })
        }
    }
}

fn random_token() -> String {
    hex::encode(Uuid::new_v4().as_bytes()) + &hex::encode(Uuid::new_v4().as_bytes())
}
fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}
fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then_some(value)
        })
}
