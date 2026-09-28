use crate::models::{Member, MemberRole};
use leptos::prelude::*;
use uuid::Uuid;

/// Lists all band members for an administrator.
#[server(AdminListMembers, "/api")]
pub(super) async fn admin_list_members() -> Result<Vec<Member>, ServerFnError> {
    let (pool, _) = crate::auth::session::require_admin().await?;
    Member::list_all(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

/// Updates a member's name.
#[server(AdminUpdateMemberName, "/api")]
pub(super) async fn admin_update_member_name(
    member_id: Uuid,
    name: String,
) -> Result<Member, ServerFnError> {
    let (pool, _) = crate::auth::session::require_admin().await?;
    let mut member = Member::find_by_id(&pool, member_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Member not found"))?;
    member
        .update_name(&pool, &name)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(member)
}

/// Updates a member's role.
#[server(AdminUpdateMemberRole, "/api")]
pub(super) async fn admin_update_member_role(
    member_id: Uuid,
    role: MemberRole,
) -> Result<Member, ServerFnError> {
    let (pool, _) = crate::auth::session::require_admin().await?;
    let mut member = Member::find_by_id(&pool, member_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Member not found"))?;
    member
        .update_role(&pool, role)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(member)
}

/// Activates or deactivates a member.
#[server(AdminSetMemberActive, "/api")]
pub(super) async fn admin_set_member_active(
    member_id: Uuid,
    active: bool,
) -> Result<Member, ServerFnError> {
    let (pool, _) = crate::auth::session::require_admin().await?;
    let mut member = Member::find_by_id(&pool, member_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?
        .ok_or_else(|| ServerFnError::new("Member not found"))?;
    member
        .set_active(&pool, active)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(member)
}

/// Permanently deletes a member.
#[server(AdminDeleteMember, "/api")]
pub(super) async fn admin_delete_member(member_id: Uuid) -> Result<(), ServerFnError> {
    let (pool, admin_id) = crate::auth::session::require_admin().await?;
    if admin_id == member_id {
        return Err(ServerFnError::new("You cannot delete your own account"));
    }
    Member::delete(&pool, member_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}
