//! Authentication: name-based login and session cookies.

#[cfg(feature = "ssr")]
pub(crate) mod session;

use crate::models::Member;
use leptos::prelude::*;

/// Lists active members for the login picker.
#[server(ListMembers, "/api")]
pub async fn list_members() -> Result<Vec<Member>, ServerFnError> {
    use sqlx::PgPool;

    let pool = expect_context::<PgPool>();
    crate::models::Member::list_active(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Logs in as the member with the given display name.
#[server(LoginAsMember, "/api")]
pub async fn login_as_member(name: String) -> Result<Member, ServerFnError> {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;
    use sqlx::PgPool;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let pool = expect_context::<PgPool>();
    let member = crate::models::Member::find_active_by_name(&pool, &name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Member not found"))?;

    let member = member
        .update_last_login(&pool)
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

    Ok(member)
}

/// Returns the currently logged-in member, if any.
#[server(CurrentMember, "/api")]
pub async fn current_member() -> Result<Option<Member>, ServerFnError> {
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
    let member = crate::models::Member::find_active_by_id(&pool, member_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(member)
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
