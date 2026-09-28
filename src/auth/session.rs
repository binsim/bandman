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

#[cfg(feature = "ssr")]
pub(super) async fn member_from_session(
    pool: &sqlx::PgPool,
    cookies: &axum_extra::extract::CookieJar,
) -> Result<Option<crate::models::Member>, ServerFnError> {
    let Some(cookie) = cookies.get(cookie_name()) else {
        return Ok(None);
    };
    let Some(member_id) = verify_session_value(cookie.value()) else {
        return Ok(None);
    };

    crate::models::Member::find_active_by_id(pool, member_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
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
pub async fn require_admin() -> Result<crate::models::Member, ServerFnError> {
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;

    let cookies: CookieJar = extract().await?;
    let app_state = expect_context::<crate::state::AppState>();
    let pool = &app_state.pool;
    let member = member_from_session(pool, &cookies)
        .await?
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    if !member.role.is_admin() {
        return Err(ServerFnError::new("Administrator access required"));
    }

    Ok(member)
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
        let (signed_id, signature) = signed.split_once('.').unwrap();
        let mut tampered_signature = signature.to_string();
        let last = tampered_signature.pop().unwrap();
        tampered_signature.push(if last == '0' { '1' } else { '0' });
        let tampered = format!("{signed_id}.{tampered_signature}");
        assert_eq!(verify_session_value(&tampered), None);
    }

    #[test]
    fn rejects_garbage() {
        std::env::set_var("SESSION_SECRET", "unit-test-secret");
        assert_eq!(verify_session_value("not-a-session"), None);
    }
}
