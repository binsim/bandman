#[cfg(feature = "ssr")]
use crate::models::{SongCategory, SongMetadataOptions, WishlistItem};
use crate::models::{
    SongMetadataAssignment, SongMetadataSelection, WishlistCategory, WishlistData, WishlistFeedback,
};
use leptos::prelude::*;

#[server(WishlistList, "/api")]
pub(super) async fn wishlist_list() -> Result<WishlistData, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_member().await?;
    let items = WishlistItem::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let categories = SongCategory::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let labels = crate::models::SongLabel::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(WishlistData {
        items,
        metadata: SongMetadataOptions { categories, labels },
    })
}

#[server(WishlistSetFeedback, "/api")]
pub(super) async fn wishlist_set_feedback(
    item_id: uuid::Uuid,
    category: Option<WishlistCategory>,
) -> Result<Option<WishlistFeedback>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::set_feedback(&pool, item_id, member.id, category)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistCreateMedley, "/api")]
pub(super) async fn wishlist_create_medley(
    item_ids: Vec<uuid::Uuid>,
    name: String,
) -> Result<uuid::Uuid, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::create_medley(&pool, &item_ids, &name, member.id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistUpdateMedley, "/api")]
pub(super) async fn wishlist_update_medley(
    medley_id: uuid::Uuid,
    item_ids: Vec<uuid::Uuid>,
    name: String,
    metadata: Option<SongMetadataSelection>,
) -> Result<SongMetadataAssignment, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::update_medley_with_metadata(
        &pool,
        medley_id,
        &item_ids,
        &name,
        member.id,
        member.role == crate::models::MemberRole::Admin,
        metadata.unwrap_or_default(),
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistRemoveMedleyItem, "/api")]
pub(super) async fn wishlist_remove_medley_item(
    medley_id: uuid::Uuid,
    item_id: uuid::Uuid,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::remove_medley_item(
        &pool,
        medley_id,
        item_id,
        member.id,
        member.role == crate::models::MemberRole::Admin,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistMoveMedleyItem, "/api")]
pub(super) async fn wishlist_move_medley_item(
    medley_id: uuid::Uuid,
    item_id: uuid::Uuid,
    position_delta: i32,
) -> Result<Vec<uuid::Uuid>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::move_medley_item(
        &pool,
        medley_id,
        item_id,
        position_delta,
        member.id,
        member.role == crate::models::MemberRole::Admin,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistDisbandMedley, "/api")]
pub(super) async fn wishlist_disband_medley(medley_id: uuid::Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::disband_medley(
        &pool,
        medley_id,
        member.id,
        member.role == crate::models::MemberRole::Admin,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}
