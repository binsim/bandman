use crate::models::{Member, MemberRole};
use leptos::prelude::*;
use uuid::Uuid;

/// Lists all band members for an administrator.
#[server(AdminListMembers, "/api")]
pub(super) async fn admin_list_members() -> Result<Vec<Member>, ServerFnError> {
    let (pool, _) = require_admin().await?;
    crate::auth::members_repo::list_members(&pool)
        .await
        .map_err(ServerFnError::new)
}

/// Creates a new member.
#[server(AdminCreateMember, "/api")]
pub(super) async fn admin_create_member(
    name: String,
    role: MemberRole,
) -> Result<Member, ServerFnError> {
    let name = validate_member_name(name).map_err(ServerFnError::new)?;
    let (pool, admin_id) = require_admin().await?;
    crate::auth::members_repo::create_member(&pool, admin_id, &name, role)
        .await
        .map_err(ServerFnError::new)
}

/// Updates a member's name and role atomically.
#[server(AdminUpdateMemberDetails, "/api")]
pub(super) async fn admin_update_member_details(
    member_id: Uuid,
    name: String,
    role: MemberRole,
) -> Result<Member, ServerFnError> {
    let name = validate_member_name(name).map_err(ServerFnError::new)?;
    let (pool, admin_id) = require_admin().await?;
    crate::auth::members_repo::update_member_details(&pool, admin_id, member_id, &name, role)
        .await
        .map_err(ServerFnError::new)
}

/// Activates or deactivates a member.
#[server(AdminSetMemberActive, "/api")]
pub(super) async fn admin_set_member_active(
    member_id: Uuid,
    active: bool,
) -> Result<Member, ServerFnError> {
    let (pool, admin_id) = require_admin().await?;
    crate::auth::members_repo::set_member_active(&pool, admin_id, member_id, active)
        .await
        .map_err(ServerFnError::new)
}

/// Permanently deletes a member.
#[server(AdminDeleteMember, "/api")]
pub(super) async fn admin_delete_member(member_id: Uuid) -> Result<(), ServerFnError> {
    let (pool, admin_id) = require_admin().await?;
    crate::auth::members_repo::delete_member(&pool, admin_id, member_id)
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

#[cfg(feature = "ssr")]
async fn require_admin() -> Result<(sqlx::PgPool, Uuid), ServerFnError> {
    use axum_extra::extract::CookieJar;
    use leptos_axum::extract;
    use sqlx::PgPool;

    let cookies: CookieJar = extract().await?;
    let cookie = cookies
        .get(crate::auth::session::cookie_name())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;
    let admin_id = crate::auth::session::verify_session_value(cookie.value())
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    let pool = expect_context::<PgPool>();
    let member = crate::auth::members_repo::find_active_by_id(&pool, admin_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Administrator access required"))?;

    if !member.role.is_admin() {
        return Err(ServerFnError::new("Administrator access required"));
    }

    Ok((pool, admin_id))
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
