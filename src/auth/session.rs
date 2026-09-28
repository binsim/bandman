//! Signed session cookie helpers (SSR).

use hmac::{Hmac, Mac};
use leptos::prelude::*;
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const COOKIE_NAME: &str = "bandman_session";

pub fn cookie_name() -> &'static str {
    COOKIE_NAME
}

fn session_secret() -> String {
    std::env::var("SESSION_SECRET").unwrap_or_else(|_| {
        tracing::warn!("SESSION_SECRET not set; using insecure default for development");
        "dev-only-insecure-session-secret-change-me".to_string()
    })
}

pub fn sign_member_id(member_id: Uuid) -> String {
    let id = member_id.to_string();
    let mut mac = HmacSha256::new_from_slice(session_secret().as_bytes()).expect("HMAC key length");
    mac.update(id.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("{id}.{sig}")
}

pub fn verify_session_value(value: &str) -> Option<Uuid> {
    let (id_str, sig) = value.split_once('.')?;
    let member_id = Uuid::parse_str(id_str).ok()?;

    let mut mac = HmacSha256::new_from_slice(session_secret().as_bytes()).expect("HMAC key length");
    mac.update(id_str.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if constant_time_eq(sig.as_bytes(), expected.as_bytes()) {
        Some(member_id)
    } else {
        None
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(feature = "ssr")]
pub async fn require_admin() -> Result<(sqlx::PgPool, Uuid), ServerFnError> {
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;

    let pool = expect_context::<crate::state::AppState>().pool;
    let cookies: CookieJar = extract().await?;
    let cookie = cookies
        .get(crate::auth::session::cookie_name())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;
    let admin_id = crate::auth::session::verify_session_value(cookie.value())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    let member = crate::models::Member::find_active_by_id(&pool, admin_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    if !member.role.is_admin() {
        return Err(ServerFnError::new("Administrator access required"));
    }

    Ok((pool, admin_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        std::env::set_var("SESSION_SECRET", "unit-test-secret");
        let id = Uuid::new_v4();
        let signed = sign_member_id(id);
        assert_eq!(verify_session_value(&signed), Some(id));
    }

    #[test]
    fn rejects_tampered_signature() {
        std::env::set_var("SESSION_SECRET", "unit-test-secret");
        let id = Uuid::new_v4();
        let signed = sign_member_id(id);
        let tampered = format!("{signed}x");
        assert_eq!(verify_session_value(&tampered), None);
    }

    #[test]
    fn rejects_garbage() {
        std::env::set_var("SESSION_SECRET", "unit-test-secret");
        assert_eq!(verify_session_value("not-a-session"), None);
    }
}
