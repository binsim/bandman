//! Authentication: name-based login and session cookies.

#[cfg(feature = "ssr")]
mod members_repo;
#[cfg(feature = "ssr")]
mod session;

use crate::models::{Member, MemberRole, MemberSummary};
use leptos::prelude::*;
use uuid::Uuid;

/// Lists active members for the login picker.
#[server(ListMembers, "/api")]
pub async fn list_members() -> Result<Vec<MemberSummary>, ServerFnError> {
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    members_repo::list_active_members(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Logs in as the member with the given display name.
#[server(LoginAsMember, "/api")]
pub async fn login_as_member(name: String) -> Result<MemberSummary, ServerFnError> {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;
    use sqlx::PgPool;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let pool = expect_context::<PgPool>();
    let member = members_repo::find_active_by_name(&pool, &name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Member not found"))?;

    members_repo::touch_last_login(&pool, member.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let signed = session::sign_member_id(member.id);
    let mut cookie = Cookie::new(session::cookie_name(), signed);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(std::env::var("COOKIE_SECURE").ok().as_deref() == Some("true"));

    let response = expect_context::<ResponseOptions>();
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string())
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );

    Ok(MemberSummary::from(member))
}

/// Returns the currently logged-in member, if any.
#[server(CurrentMember, "/api")]
pub async fn current_member() -> Result<Option<MemberSummary>, ServerFnError> {
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let jar: CookieJar = extract().await?;
    let Some(cookie) = jar.get(session::cookie_name()) else {
        return Ok(None);
    };
    let Some(member_id) = session::verify_session_value(cookie.value()) else {
        return Ok(None);
    };

    let pool = expect_context::<PgPool>();
    let member = members_repo::find_active_by_id(&pool, member_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(member.map(MemberSummary::from))
}

/// Clears the session cookie.
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;

    let mut cookie = Cookie::new(session::cookie_name(), "");
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Lax);
    cookie.make_removal();

    let response = expect_context::<ResponseOptions>();
    response.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie.to_string())
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    Ok(())
}

/// Lists all band members for an administrator.
#[server(AdminListMembers, "/api")]
pub async fn admin_list_members() -> Result<Vec<Member>, ServerFnError> {
    let (pool, _) = require_admin().await?;
    members_repo::list_members(&pool)
        .await
        .map_err(ServerFnError::new)
}

/// Creates a new member.
#[server(AdminCreateMember, "/api")]
pub async fn admin_create_member(name: String, role: MemberRole) -> Result<Member, ServerFnError> {
    let name = validate_member_name(name).map_err(ServerFnError::new)?;
    let (pool, admin_id) = require_admin().await?;
    members_repo::create_member(&pool, admin_id, &name, role)
        .await
        .map_err(ServerFnError::new)
}

/// Renames a member while keeping the member ID and session history unchanged.
#[server(AdminSetMemberName, "/api")]
pub async fn admin_set_member_name(member_id: Uuid, name: String) -> Result<(), ServerFnError> {
    let name = validate_member_name(name).map_err(ServerFnError::new)?;
    let (pool, admin_id) = require_admin().await?;
    members_repo::set_member_name(&pool, admin_id, member_id, &name)
        .await
        .map_err(ServerFnError::new)
}

/// Changes a member's role.
#[server(AdminSetMemberRole, "/api")]
pub async fn admin_set_member_role(member_id: Uuid, role: MemberRole) -> Result<(), ServerFnError> {
    let (pool, admin_id) = require_admin().await?;
    members_repo::set_member_role(&pool, admin_id, member_id, role)
        .await
        .map_err(ServerFnError::new)
}

/// Activates or deactivates a member.
#[server(AdminSetMemberActive, "/api")]
pub async fn admin_set_member_active(member_id: Uuid, active: bool) -> Result<(), ServerFnError> {
    let (pool, admin_id) = require_admin().await?;
    members_repo::set_member_active(&pool, admin_id, member_id, active)
        .await
        .map_err(ServerFnError::new)
}

#[cfg(feature = "ssr")]
fn validate_member_name(name: String) -> Result<String, &'static str> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err("Name must be 1-80 characters and contain no control characters");
    }
    Ok(name.to_string())
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::validate_member_name;

    #[test]
    fn member_name_is_trimmed() {
        assert_eq!(
            validate_member_name("  Band Member  ".to_string()).as_deref(),
            Ok("Band Member")
        );
    }

    #[test]
    fn member_name_rejects_empty_control_and_overlong_values() {
        assert!(validate_member_name("  ".to_string()).is_err());
        assert!(validate_member_name("Band\nMember".to_string()).is_err());
        assert!(validate_member_name("a".repeat(81)).is_err());
    }

    #[test]
    fn member_name_accepts_up_to_eighty_characters() {
        let expected = "é".repeat(80);
        assert_eq!(
            validate_member_name(expected.clone()).as_deref(),
            Ok(expected.as_str())
        );
    }
}

#[cfg(feature = "ssr")]
async fn require_admin() -> Result<(sqlx::PgPool, Uuid), ServerFnError> {
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let cookies: CookieJar = extract().await?;
    let cookie = cookies
        .get(session::cookie_name())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;
    let admin_id = session::verify_session_value(cookie.value())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    let pool = expect_context::<PgPool>();
    let member = members_repo::find_active_by_id(&pool, admin_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    if !member.role.is_admin() {
        return Err(ServerFnError::new("Administrator access required"));
    }

    Ok((pool, admin_id))
}
