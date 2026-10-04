mod create_song;

use crate::models::{
    Member, WishlistCategory, WishlistFeedback, WishlistItem, WishlistMedleyFeedback,
};
use leptos::prelude::*;
use leptos_fluent::tr;

use create_song::CreateSong;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct WishlistPageData {
    items: Vec<WishlistItem>,
    medley_feedback: Vec<WishlistMedleyFeedback>,
    show_needs_feedback: bool,
}

#[server(WishlistList, "/api")]
async fn wishlist_list() -> Result<WishlistPageData, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    let items = WishlistItem::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let show_needs_feedback = Member::wishlist_show_needs_feedback(&pool, member.id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let medley_feedback = WishlistItem::list_medley_feedback(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(WishlistPageData {
        items,
        medley_feedback,
        show_needs_feedback,
    })
}

#[server(WishlistSetFeedbackFilter, "/api")]
async fn wishlist_set_feedback_filter(show_needs_feedback: bool) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    Member::set_wishlist_show_needs_feedback(&pool, member.id, show_needs_feedback)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistCreate, "/api")]
async fn wishlist_create(
    title: String,
    artist: String,
    link: String,
) -> Result<WishlistItem, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    let link_title = resolve_link_title(&link).await;
    WishlistItem::create(
        &pool,
        &title,
        Some(&artist),
        Some(&link),
        link_title.as_deref(),
        &[],
        WishlistCategory::Song,
        member.id,
        &member.name,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistSetFeedback, "/api")]
async fn wishlist_set_feedback(
    item_id: uuid::Uuid,
    category: Option<WishlistCategory>,
) -> Result<Option<WishlistFeedback>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::set_feedback(&pool, item_id, member.id, category)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistSetMedleyFeedback, "/api")]
async fn wishlist_set_medley_feedback(
    medley_id: uuid::Uuid,
    category: Option<WishlistCategory>,
) -> Result<Option<WishlistMedleyFeedback>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::set_medley_feedback(&pool, medley_id, member.id, category)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistDelete, "/api")]
async fn wishlist_delete(item_id: uuid::Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::delete(
        &pool,
        item_id,
        member.id,
        member.role == crate::models::MemberRole::Admin,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistUpdate, "/api")]
async fn wishlist_update(
    item_id: uuid::Uuid,
    title: String,
    artist: String,
    link: String,
) -> Result<WishlistItem, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    let link_title = resolve_link_title(&link).await;
    WishlistItem::update_details(
        &pool,
        item_id,
        member.id,
        member.role == crate::models::MemberRole::Admin,
        &title,
        Some(&artist),
        Some(&link),
        link_title.as_deref(),
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistCreateMedley, "/api")]
async fn wishlist_create_medley(
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
async fn wishlist_update_medley(
    medley_id: uuid::Uuid,
    item_ids: Vec<uuid::Uuid>,
    name: String,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    WishlistItem::update_medley(
        &pool,
        medley_id,
        &item_ids,
        &name,
        member.id,
        member.role == crate::models::MemberRole::Admin,
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(WishlistRemoveMedleyItem, "/api")]
async fn wishlist_remove_medley_item(
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
async fn wishlist_move_medley_item(
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
async fn wishlist_disband_medley(medley_id: uuid::Uuid) -> Result<(), ServerFnError> {
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

#[derive(serde::Deserialize)]
#[cfg(feature = "ssr")]
struct YoutubeOembed {
    title: String,
}

#[cfg(feature = "ssr")]
async fn resolve_link_title(link: &str) -> Option<String> {
    let url = reqwest::Url::parse(link).ok()?;
    if !matches!(
        url.host_str(),
        Some(
            "youtube.com" | "www.youtube.com" | "m.youtube.com" | "music.youtube.com" | "youtu.be"
        )
    ) {
        return None;
    }

    let response = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(client) => {
            client
                .get("https://www.youtube.com/oembed")
                .query(&[("url", link), ("format", "json")])
                .send()
                .await
        }
        Err(error) => {
            tracing::warn!(%error, "Could not build YouTube metadata client");
            return None;
        }
    };

    match response {
        Ok(response) if response.status().is_success() => {
            match response.json::<YoutubeOembed>().await {
                Ok(metadata) if !metadata.title.trim().is_empty() => {
                    Some(metadata.title.trim().chars().take(200).collect())
                }
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(%error, "Could not decode YouTube link metadata");
                    None
                }
            }
        }
        Ok(response) => {
            tracing::warn!(status = %response.status(), "Could not fetch YouTube link metadata");
            None
        }
        Err(error) => {
            tracing::warn!(%error, "Could not fetch YouTube link metadata");
            None
        }
    }
}

#[component]
pub fn WishlistPage() -> impl IntoView {
    let member = expect_context::<Resource<Option<crate::models::Member>>>();
    view! {
        <Suspense fallback=|| view! {
            <section class="wishlist-page">
                <h1>{move || tr!("nav-wishlist")}</h1>
                <p class="muted">{move || tr!("wishlist-loading")}</p>
            </section>
        }>
            {move || match member.get() {
                Some(Some(member)) => view! { <WishlistContent current_member=member /> }.into_any(),
                Some(None) => view! {
                    <section class="wishlist-page">
                        <header class="wishlist-header">
                            <h1>{move || tr!("nav-wishlist")}</h1>
                            <p class="lead">{move || tr!("wishlist-sign-in")}</p>
                            <a class="btn btn-primary" href="/login">{move || tr!("nav-login")}</a>
                        </header>
                    </section>
                }
                .into_any(),
                None => ().into_any(),
            }}
        </Suspense>
    }
}

#[component]
fn WishlistContent(current_member: Member) -> impl IntoView {
    let wishlist = Resource::new(|| (), |_| async move { wishlist_list().await });
    view! {
        <Suspense fallback=|| view! {
            <section class="wishlist-page">
                <h1>{move || tr!("nav-wishlist")}</h1>
                <p class="muted">{move || tr!("wishlist-loading")}</p>
            </section>
        }>
            {move || match wishlist.get() {
                Some(Ok(data)) => view! {
                    <WishlistLoaded
                        initial_items=data.items
                        initial_medley_feedback=data.medley_feedback
                        initial_show_needs_feedback=data.show_needs_feedback
                        member_id=current_member.id
                        can_delete_all=current_member.role == crate::models::MemberRole::Admin
                    />
                }.into_any(),
                Some(Err(error)) => view! {
                    <section class="wishlist-page">
                        <h1>{move || tr!("nav-wishlist")}</h1>
                        <p class="form-error" role="alert">{error.to_string()}</p>
                    </section>
                }
                .into_any(),
                None => ().into_any(),
            }}
        </Suspense>
    }
}

#[component]
fn WishlistLoaded(
    initial_items: Vec<WishlistItem>,
    initial_medley_feedback: Vec<WishlistMedleyFeedback>,
    initial_show_needs_feedback: bool,
    member_id: uuid::Uuid,
    can_delete_all: bool,
) -> AnyView {
    let items = RwSignal::new(initial_items);
    let medley_feedback = RwSignal::new(initial_medley_feedback);
    let medley_management_active = RwSignal::new(false);
    provide_context(medley_management_active);
    let editing_medley_id = RwSignal::new(Option::<uuid::Uuid>::None);
    provide_context(editing_medley_id);
    let editing_medley_song_ids = RwSignal::new(Vec::<uuid::Uuid>::new());
    provide_context(editing_medley_song_ids);
    let show_needs_feedback = RwSignal::new(initial_show_needs_feedback);
    let filter_pending = RwSignal::new(false);
    let filter_error = RwSignal::new(Option::<String>::None);
    let on_filter_changed = Callback::new(move |show: bool| {
        if filter_pending.get() || show_needs_feedback.get() == show {
            return;
        }
        let previous = show_needs_feedback.get();
        show_needs_feedback.set(show);
        filter_pending.set(true);
        filter_error.set(None);
        leptos::task::spawn_local(async move {
            if let Err(message) = wishlist_set_feedback_filter(show).await {
                show_needs_feedback.set(previous);
                filter_error.set(Some(message.to_string()));
            }
            filter_pending.set(false);
        });
    });
    let selecting_medley = RwSignal::new(false);
    let new_medley_name = RwSignal::new(String::new());
    let selected_songs = RwSignal::new(Vec::<uuid::Uuid>::new());
    let medley_pending = RwSignal::new(false);
    let medley_error = RwSignal::new(Option::<String>::None);
    let visible_items = Memo::new(move |_| {
        let items = items.get();
        if editing_medley_id.get().is_some() {
            items
        } else if show_needs_feedback.get() {
            let medleys_with_pending_feedback: std::collections::HashSet<_> = items
                .iter()
                .filter(|item| {
                    !item
                        .feedback
                        .iter()
                        .any(|feedback| feedback.member_id == member_id)
                })
                .filter_map(|item| item.medley_id)
                .collect();
            items
                .into_iter()
                .filter(|item| {
                    item.medley_id
                        .map(|medley_id| medleys_with_pending_feedback.contains(&medley_id))
                        .unwrap_or_else(|| {
                            !item
                                .feedback
                                .iter()
                                .any(|feedback| feedback.member_id == member_id)
                        })
                })
                .collect::<Vec<_>>()
        } else {
            items
        }
    });
    let on_song_created = Callback::new(move |item: WishlistItem| {
        items.update(|items| {
            items.push(item);
            sort_wishlist_items(items);
        });
    });
    let create_medley = Callback::new(move |_: ()| {
        let selected = selected_songs.get();
        let name = new_medley_name.get();
        medley_pending.set(true);
        medley_error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_create_medley(selected.clone(), name.clone()).await {
                Ok(medley_id) => {
                    items.update(|items| {
                        for (index, id) in selected.iter().enumerate() {
                            if let Some(item) = items.iter_mut().find(|item| item.id == *id) {
                                item.medley_id = Some(medley_id);
                                item.medley_position = Some(index as i32);
                                item.medley_size = Some(selected.len() as i64);
                                item.medley_created_by_id = Some(member_id);
                                item.medley_name = Some(name.clone());
                            }
                        }
                        sort_wishlist_items(items);
                    });
                    on_filter_changed.run(false);
                    selected_songs.set(Vec::new());
                    selecting_medley.set(false);
                    medley_management_active.set(false);
                    new_medley_name.set(String::new());
                }
                Err(message) => medley_error.set(Some(message.to_string())),
            }
            medley_pending.set(false);
        });
    });
    let start_creating_medley = Callback::new(move |_: ()| {
        selected_songs.set(Vec::new());
        medley_error.set(None);
        selecting_medley.set(true);
        medley_management_active.set(true);
    });
    let on_medley_renamed = Callback::new(move |(medley_id, name): (uuid::Uuid, String)| {
        items.update(|items| {
            for item in items
                .iter_mut()
                .filter(|item| item.medley_id == Some(medley_id))
            {
                item.medley_name = Some(name.clone());
            }
        });
    });
    let remove_medley_item =
        Callback::new(move |(medley_id, item_id): (uuid::Uuid, uuid::Uuid)| {
            if medley_pending.get() {
                return;
            }
            medley_pending.set(true);
            medley_error.set(None);
            leptos::task::spawn_local(async move {
                match wishlist_remove_medley_item(medley_id, item_id).await {
                    Ok(()) => {
                        items.update(|items| {
                            let mut members: Vec<_> = items
                                .iter()
                                .filter(|item| {
                                    item.medley_id == Some(medley_id) && item.id != item_id
                                })
                                .map(|item| (item.medley_position.unwrap_or_default(), item.id))
                                .collect();
                            members.sort_by_key(|(position, _)| *position);
                            if let Some(item) = items.iter_mut().find(|item| item.id == item_id) {
                                item.medley_id = None;
                                item.medley_position = None;
                                item.medley_size = None;
                                item.medley_created_by_id = None;
                                item.medley_name = None;
                            }
                            if members.len() < 2 {
                                for item in items
                                    .iter_mut()
                                    .filter(|item| item.medley_id == Some(medley_id))
                                {
                                    item.medley_id = None;
                                    item.medley_position = None;
                                    item.medley_size = None;
                                    item.medley_created_by_id = None;
                                    item.medley_name = None;
                                }
                            } else {
                                for (position, (_, id)) in members.iter().enumerate() {
                                    if let Some(item) = items.iter_mut().find(|item| item.id == *id)
                                    {
                                        item.medley_position = Some(position as i32);
                                        item.medley_size = Some(members.len() as i64);
                                    }
                                }
                            }
                            sort_wishlist_items(items);
                        });
                    }
                    Err(message) => medley_error.set(Some(message.to_string())),
                }
                medley_pending.set(false);
            });
        });
    let move_medley_item = Callback::new(
        move |(medley_id, item_id, position_delta): (uuid::Uuid, uuid::Uuid, i32)| {
            if medley_pending.get() {
                return;
            }
            medley_pending.set(true);
            medley_error.set(None);
            leptos::task::spawn_local(async move {
                match wishlist_move_medley_item(medley_id, item_id, position_delta).await {
                    Ok(ordered_ids) => items.update(|items| {
                        for (position, id) in ordered_ids.iter().enumerate() {
                            if let Some(item) = items.iter_mut().find(|item| item.id == *id) {
                                item.medley_position = Some(position as i32);
                            }
                        }
                        sort_wishlist_items(items);
                    }),
                    Err(message) => medley_error.set(Some(message.to_string())),
                }
                medley_pending.set(false);
            });
        },
    );
    let disband_medley = Callback::new(move |medley_id: uuid::Uuid| {
        if medley_pending.get() {
            return;
        }
        medley_pending.set(true);
        medley_error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_disband_medley(medley_id).await {
                Ok(()) => {
                    items.update(|items| {
                        for item in items
                            .iter_mut()
                            .filter(|item| item.medley_id == Some(medley_id))
                        {
                            item.medley_id = None;
                            item.medley_position = None;
                            item.medley_size = None;
                            item.medley_created_by_id = None;
                            item.medley_name = None;
                        }
                        sort_wishlist_items(items);
                    });
                }
                Err(message) => medley_error.set(Some(message.to_string())),
            }
            medley_pending.set(false);
        });
    });

    view! {
        <section class="wishlist-page">
            <header class="wishlist-header">
                <p class="eyebrow">{move || tr!("wishlist-eyebrow")}</p>
                <h1>{move || tr!("nav-wishlist")}</h1>
                <p class="lead">{move || tr!("wishlist-lead")}</p>
            </header>
            <CreateSong items on_created=on_song_created />
            <WishlistListing
                items
                medley_feedback
                visible_items
                member_id
                can_delete_all
                show_needs_feedback
                filter_pending
                filter_error
                on_filter_changed
                selecting_medley
                new_medley_name
                selected_songs
                medley_pending
                medley_error
                on_create_medley=create_medley
                on_start_creating_medley=start_creating_medley
                on_disband_medley=disband_medley
                on_medley_renamed
                on_remove_medley_item=remove_medley_item
                on_move_medley_item=move_medley_item
            />
        </section>
    }
    .into_any()
}

#[derive(Clone, PartialEq, Eq)]
enum WishlistDisplayEntry {
    Song(WishlistItem),
    Medley { id: uuid::Uuid },
}

impl WishlistDisplayEntry {
    fn key(&self) -> String {
        match self {
            Self::Song(item) => format!("song-{}", item.id),
            Self::Medley { id } => format!("medley-{id}"),
        }
    }
}

fn group_wishlist_items(items: Vec<WishlistItem>) -> Vec<WishlistDisplayEntry> {
    let mut entries = Vec::new();
    let mut medley_indices = std::collections::HashMap::new();
    for item in items {
        let Some(medley_id) = item.medley_id else {
            entries.push(WishlistDisplayEntry::Song(item));
            continue;
        };
        if !medley_indices.contains_key(&medley_id) {
            let index = entries.len();
            medley_indices.insert(medley_id, index);
            entries.push(WishlistDisplayEntry::Medley { id: medley_id });
        }
    }
    entries
}

#[component]
fn WishlistListing(
    items: RwSignal<Vec<WishlistItem>>,
    medley_feedback: RwSignal<Vec<WishlistMedleyFeedback>>,
    visible_items: Memo<Vec<WishlistItem>>,
    member_id: uuid::Uuid,
    can_delete_all: bool,
    show_needs_feedback: RwSignal<bool>,
    filter_pending: RwSignal<bool>,
    filter_error: RwSignal<Option<String>>,
    on_filter_changed: Callback<bool>,
    selecting_medley: RwSignal<bool>,
    new_medley_name: RwSignal<String>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    medley_error: RwSignal<Option<String>>,
    on_create_medley: Callback<()>,
    on_start_creating_medley: Callback<()>,
    on_disband_medley: Callback<uuid::Uuid>,
    on_medley_renamed: Callback<(uuid::Uuid, String)>,
    on_remove_medley_item: Callback<(uuid::Uuid, uuid::Uuid)>,
    on_move_medley_item: Callback<(uuid::Uuid, uuid::Uuid, i32)>,
) -> AnyView {
    let entries = Memo::new(move |_| group_wishlist_items(visible_items.get()));
    view! {
        <section class="wishlist-list" aria-labelledby="wishlist-list-title">
            <div class="wishlist-list-header">
                <h2 id="wishlist-list-title">{move || tr!("wishlist-list-title")}</h2>
                <WishlistListHeader
                    items
                    show_needs_feedback
                    filter_pending
                    on_filter_changed
                    selecting_medley
                    on_start_creating_medley
                />
            </div>
            <WishlistMedleyControls
                selecting_medley
                new_medley_name
                selected_songs
                medley_pending
                medley_error
                on_create_medley
            />
            <Show when=move || medley_error.get().is_some()>
                <p class="form-error" role="alert">{move || medley_error.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || filter_error.get().is_some()>
                <p class="form-error" role="alert">{move || filter_error.get().unwrap_or_default()}</p>
            </Show>
            <Show
                when=move || !items.get().is_empty()
                fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-empty")}</p> }
            >
                <Show
                    when=move || !visible_items.get().is_empty()
                    fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-no-feedback-pending")}</p> }
                >
                    <div class="wishlist-items">
                        <WishlistItemList
                            items
                            medley_feedback
                            entries
                            member_id
                            can_delete_all
                            selecting_medley
                            selected_songs
                            medley_pending
                            on_disband_medley
                            on_medley_renamed
                            on_remove_medley_item
                            on_move_medley_item
                        />
                    </div>
                </Show>
            </Show>
        </section>
    }
    .into_any()
}

#[component]
fn WishlistListHeader(
    items: RwSignal<Vec<WishlistItem>>,
    show_needs_feedback: RwSignal<bool>,
    filter_pending: RwSignal<bool>,
    on_filter_changed: Callback<bool>,
    selecting_medley: RwSignal<bool>,
    on_start_creating_medley: Callback<()>,
) -> AnyView {
    view! {
        <div class="wishlist-list-header-actions">
            <span class="muted">{move || tr!("wishlist-count", {"count" => items.get().len().to_string()})}</span>
            <div class="wishlist-filter" role="group" aria-label=move || tr!("wishlist-filter-label")>
                <button
                    class="wishlist-filter-button"
                    class:is-active=move || show_needs_feedback.get()
                    type="button"
                    aria-pressed=move || show_needs_feedback.get().to_string()
                    on:click=move |_| on_filter_changed.run(true)
                    disabled=move || filter_pending.get()
                >
                    {move || tr!("wishlist-filter-needs-feedback")}
                </button>
                <button
                    class="wishlist-filter-button"
                    class:is-active=move || !show_needs_feedback.get()
                    type="button"
                    aria-pressed=move || (!show_needs_feedback.get()).to_string()
                    on:click=move |_| on_filter_changed.run(false)
                    disabled=move || filter_pending.get()
                >
                    {move || tr!("wishlist-filter-all")}
                </button>
            </div>
            <Show when=move || !selecting_medley.get()>
                <button
                    class="btn btn-primary btn-sm wishlist-create-medley-button"
                    type="button"
                    on:click=move |_| on_start_creating_medley.run(())
                >
                    {move || tr!("wishlist-medley-start")}
                </button>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistMedleyControls(
    selecting_medley: RwSignal<bool>,
    new_medley_name: RwSignal<String>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    medley_error: RwSignal<Option<String>>,
    on_create_medley: Callback<()>,
) -> AnyView {
    let medley_management_active = expect_context::<RwSignal<bool>>();
    view! {
        <div
            class="wishlist-medley-controls"
            class:is-selecting=move || selecting_medley.get()
        >
            <Show when=move || selecting_medley.get()>
                <button
                    class="btn btn-ghost btn-sm"
                    type="button"
                    on:click=move |_| {
                        selecting_medley.set(false);
                        medley_management_active.set(false);
                        selected_songs.set(Vec::new());
                        new_medley_name.set(String::new());
                        medley_error.set(None);
                    }
                    disabled=move || medley_pending.get()
                >
                    {move || tr!("admin-cancel")}
                </button>
            </Show>
            <Show when=move || selecting_medley.get()>
                <span class="wishlist-medley-selection-hint">
                    {move || tr!("wishlist-medley-create-hint")}
                </span>
                <div class="wishlist-medley-selection-actions">
                    <span class="muted wishlist-medley-selected-count">
                        {move || tr!("wishlist-medley-selected", {"count" => selected_songs.get().len().to_string()})}
                    </span>
                    <input
                        class="input wishlist-medley-name-input"
                        maxlength="120"
                        aria-label=move || tr!("wishlist-medley-name")
                        placeholder=move || tr!("wishlist-medley-name")
                        prop:value=move || new_medley_name.get()
                        on:input=move |event| new_medley_name.set(event_target_value(&event))
                        disabled=move || medley_pending.get()
                    />
                    <button
                        class="btn btn-primary btn-sm"
                        type="button"
                        on:click=move |_| on_create_medley.run(())
                        disabled=move || medley_pending.get() || selected_songs.get().len() < 2 || new_medley_name.get().trim().is_empty()
                    >
                        {move || if medley_pending.get() { tr!("wishlist-medley-saving") } else { tr!("wishlist-medley-create") }}
                    </button>
                </div>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistItemList(
    items: RwSignal<Vec<WishlistItem>>,
    medley_feedback: RwSignal<Vec<WishlistMedleyFeedback>>,
    entries: Memo<Vec<WishlistDisplayEntry>>,
    member_id: uuid::Uuid,
    can_delete_all: bool,
    selecting_medley: RwSignal<bool>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    on_disband_medley: Callback<uuid::Uuid>,
    on_medley_renamed: Callback<(uuid::Uuid, String)>,
    on_remove_medley_item: Callback<(uuid::Uuid, uuid::Uuid)>,
    on_move_medley_item: Callback<(uuid::Uuid, uuid::Uuid, i32)>,
) -> AnyView {
    let on_feedback_changed = Callback::new(
        move |(item_id, feedback): (uuid::Uuid, Option<WishlistFeedback>)| {
            items.update(|items| {
                if let Some(item) = items.iter_mut().find(|item| item.id == item_id) {
                    item.feedback
                        .retain(|existing| existing.member_id != member_id);
                    if let Some(feedback) = feedback {
                        item.feedback.push(feedback);
                    }
                }
            });
        },
    );
    let on_updated = Callback::new(move |updated: WishlistItem| {
        items.update(|items| {
            if let Some(item) = items.iter_mut().find(|item| item.id == updated.id) {
                let mut updated = updated.clone();
                updated.feedback = item.feedback.clone();
                updated.medley_id = item.medley_id;
                updated.medley_position = item.medley_position;
                updated.medley_size = item.medley_size;
                updated.medley_created_by_id = item.medley_created_by_id;
                updated.medley_name = item.medley_name.clone();
                *item = updated;
            }
            sort_wishlist_items(items);
        });
    });
    let on_deleted = Callback::new(move |item_id| {
        items.update(|items| {
            let medley_id = items
                .iter()
                .find(|item| item.id == item_id)
                .and_then(|item| item.medley_id);
            items.retain(|item| item.id != item_id);
            if let Some(medley_id) = medley_id {
                let mut members: Vec<_> = items
                    .iter()
                    .filter(|item| item.medley_id == Some(medley_id))
                    .map(|item| (item.medley_position.unwrap_or_default(), item.id))
                    .collect();
                members.sort_by_key(|(position, _)| *position);
                if members.len() < 2 {
                    for item in items
                        .iter_mut()
                        .filter(|item| item.medley_id == Some(medley_id))
                    {
                        item.medley_id = None;
                        item.medley_position = None;
                        item.medley_size = None;
                        item.medley_created_by_id = None;
                        item.medley_name = None;
                    }
                } else {
                    for (position, (_, member_id)) in members.iter().enumerate() {
                        if let Some(item) = items.iter_mut().find(|item| item.id == *member_id) {
                            item.medley_position = Some(position as i32);
                            item.medley_size = Some(members.len() as i64);
                        }
                    }
                }
            }
            sort_wishlist_items(items);
        });
    });

    view! {
        <For
            each=move || entries.get()
            key=WishlistDisplayEntry::key
            children=move |entry| {
                view! {
                    <WishlistListEntry
                        entry
                        items
                        medley_feedback
                        member_id
                        can_delete_all
                        selecting_medley
                        selected_songs
                        medley_pending
                        on_disband_medley
                        on_medley_renamed
                        on_remove_medley_item
                        on_move_medley_item
                        on_feedback_changed
                        on_updated
                        on_deleted
                    />
                }.into_any()
            }
        />
    }
    .into_any()
}

#[component]
fn WishlistListEntry(
    entry: WishlistDisplayEntry,
    items: RwSignal<Vec<WishlistItem>>,
    medley_feedback: RwSignal<Vec<WishlistMedleyFeedback>>,
    member_id: uuid::Uuid,
    can_delete_all: bool,
    selecting_medley: RwSignal<bool>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    on_disband_medley: Callback<uuid::Uuid>,
    on_medley_renamed: Callback<(uuid::Uuid, String)>,
    on_remove_medley_item: Callback<(uuid::Uuid, uuid::Uuid)>,
    on_move_medley_item: Callback<(uuid::Uuid, uuid::Uuid, i32)>,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_updated: Callback<WishlistItem>,
    on_deleted: Callback<uuid::Uuid>,
) -> AnyView {
    match entry {
        WishlistDisplayEntry::Song(item) => view! {
            <WishlistListItem
                item
                member_id
                can_delete_all
                selecting_medley
                selected_songs
                medley_pending
                on_feedback_changed
                on_updated
                on_deleted
                on_remove_medley_item
                on_move_medley_item
                can_manage_medley=false
            />
        }
        .into_any(),
        WishlistDisplayEntry::Medley { id } => view! {
            <WishlistMedley
                id
                items
                medley_feedback
                member_id
                can_delete_all
                selecting_medley
                selected_songs
                medley_pending
                on_disband_medley
                on_medley_renamed
                on_remove_medley_item
                on_move_medley_item
                on_feedback_changed
                on_updated
                on_deleted
            />
        }
        .into_any(),
    }
}

#[component]
fn WishlistMedley(
    id: uuid::Uuid,
    items: RwSignal<Vec<WishlistItem>>,
    medley_feedback: RwSignal<Vec<WishlistMedleyFeedback>>,
    member_id: uuid::Uuid,
    can_delete_all: bool,
    selecting_medley: RwSignal<bool>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    on_disband_medley: Callback<uuid::Uuid>,
    on_medley_renamed: Callback<(uuid::Uuid, String)>,
    on_remove_medley_item: Callback<(uuid::Uuid, uuid::Uuid)>,
    on_move_medley_item: Callback<(uuid::Uuid, uuid::Uuid, i32)>,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_updated: Callback<WishlistItem>,
    on_deleted: Callback<uuid::Uuid>,
) -> AnyView {
    let medley_management_active = expect_context::<RwSignal<bool>>();
    let songs = Memo::new(move |_| {
        let mut songs: Vec<_> = items
            .get()
            .into_iter()
            .filter(|item| item.medley_id == Some(id))
            .collect();
        songs.sort_by_key(|item| item.medley_position);
        songs
    });
    let medley_name = Memo::new(move |_| {
        songs
            .get()
            .first()
            .and_then(|item| item.medley_name.clone())
            .unwrap_or_default()
    });
    let medley_creator = Memo::new(move |_| {
        songs
            .get()
            .first()
            .and_then(|item| item.medley_created_by_id)
    });
    let can_manage = move || can_delete_all || medley_creator.get() == Some(member_id);
    let song_count = Memo::new(move |_| songs.get().len());
    let expanded = RwSignal::new(true);
    let editing_name = RwSignal::new(false);
    let edit_name = RwSignal::new(medley_name.get_untracked());
    let edit_song_ids = expect_context::<RwSignal<Vec<uuid::Uuid>>>();
    let editing_medley_id = expect_context::<RwSignal<Option<uuid::Uuid>>>();
    let rename_pending = RwSignal::new(false);
    let rename_error = RwSignal::new(Option::<String>::None);
    let confirm_disband = RwSignal::new(false);
    let edit_songs = Memo::new(move |_| {
        let all_items = items.get();
        edit_song_ids
            .get()
            .into_iter()
            .filter_map(|song_id| all_items.iter().find(|item| item.id == song_id).cloned())
            .collect::<Vec<_>>()
    });
    let start_editing = Callback::new(move |_: ()| {
        edit_name.set(medley_name.get());
        edit_song_ids.set(songs.get().iter().map(|item| item.id).collect());
        rename_error.set(None);
        editing_medley_id.set(Some(id));
        editing_name.set(true);
        medley_management_active.set(true);
        expanded.set(true);
    });
    let cancel_editing = Callback::new(move |_: ()| {
        edit_name.set(medley_name.get());
        edit_song_ids.set(Vec::new());
        rename_error.set(None);
        editing_medley_id.set(None);
        editing_name.set(false);
        medley_management_active.set(false);
    });
    let confirm_ungroup = Callback::new(move |_: ()| {
        on_disband_medley.run(id);
        confirm_disband.set(false);
    });
    let save_medley = move |_| {
        let new_name = edit_name.get().trim().to_string();
        let ordered_ids = edit_song_ids.get();
        if new_name.is_empty() || ordered_ids.len() < 2 || rename_pending.get() {
            return;
        }
        rename_pending.set(true);
        medley_pending.set(true);
        rename_error.set(None);
        let creator_id = medley_creator.get();
        leptos::task::spawn_local(async move {
            match wishlist_update_medley(id, ordered_ids.clone(), new_name.clone()).await {
                Ok(()) => {
                    let song_count = ordered_ids.len() as i64;
                    items.update(|items| {
                        for item in items.iter_mut().filter(|item| item.medley_id == Some(id)) {
                            if !ordered_ids.contains(&item.id) {
                                item.medley_id = None;
                                item.medley_position = None;
                                item.medley_size = None;
                                item.medley_created_by_id = None;
                                item.medley_name = None;
                            }
                        }
                        for (position, song_id) in ordered_ids.iter().enumerate() {
                            if let Some(item) = items.iter_mut().find(|item| item.id == *song_id) {
                                item.medley_id = Some(id);
                                item.medley_position = Some(position as i32);
                                item.medley_size = Some(song_count);
                                item.medley_created_by_id = creator_id;
                                item.medley_name = Some(new_name.clone());
                            }
                        }
                        sort_wishlist_items(items);
                    });
                    on_medley_renamed.run((id, new_name.clone()));
                    edit_song_ids.set(Vec::new());
                    editing_medley_id.set(None);
                    editing_name.set(false);
                    medley_management_active.set(false);
                }
                Err(message) => rename_error.set(Some(message.to_string())),
            }
            rename_pending.set(false);
            medley_pending.set(false);
        });
    };
    view! {
        <section
            class="wishlist-medley"
            class:is-expanded=move || expanded.get()
            data-testid="wishlist-medley"
        >
            <header class="wishlist-medley-header">
                <div class="wishlist-medley-heading">
                    <button
                        class="wishlist-medley-toggle"
                        type="button"
                        aria-expanded=move || expanded.get().to_string()
                        aria-label=move || if expanded.get() {
                            tr!("wishlist-medley-collapse")
                        } else {
                            tr!("wishlist-medley-expand")
                        }
                        on:click=move |_| expanded.update(|expanded| *expanded = !*expanded)
                    >
                        <span aria-hidden="true">{move || if expanded.get() { "▾" } else { "▸" }}</span>
                    </button>
                    <h3>{move || medley_name.get()}</h3>
                    <span class="muted">
                        {move || tr!("wishlist-medley-song-count", {"count" => song_count.get().to_string()})}
                    </span>
                </div>
                <div class="wishlist-medley-header-actions">
                    <Show when=move || !medley_management_active.get()>
                        <WishlistMedleyFeedbackControl
                            medley_id=id
                            member_id
                            feedbacks=medley_feedback
                        />
                    </Show>
                    <Show when=move || !medley_management_active.get() && can_manage() && !selecting_medley.get()>
                        <WishlistMedleyActions
                            name=medley_name
                            pending=medley_pending
                            confirming=confirm_disband
                            on_edit=start_editing
                            on_confirm_disband=confirm_ungroup
                        />
                    </Show>
                </div>
            </header>
            <Show when=move || rename_error.get().is_some()>
                <p class="form-error" role="alert">{move || rename_error.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || editing_name.get()>
                <section
                    class="wishlist-medley-editor"
                    aria-label=move || tr!("wishlist-medley-edit")
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if event.key() == "Escape" && !rename_pending.get() {
                            cancel_editing.run(());
                        }
                    }
                >
                    <header class="wishlist-medley-editor-header">
                        <h2>{move || tr!("wishlist-medley-edit")}</h2>
                        <input
                            class="input"
                            maxlength="120"
                            aria-label=move || tr!("wishlist-medley-name")
                            prop:value=move || edit_name.get()
                            on:input=move |event| edit_name.set(event_target_value(&event))
                            disabled=move || rename_pending.get()
                        />
                    </header>
                    <p class="muted">{move || tr!("wishlist-medley-edit-hint")}</p>
                    <div class="wishlist-medley-edit-list">
                        <For
                            each=move || edit_songs.get()
                            key=|item| item.id
                            children=move |item| {
                                let song_id = item.id;
                                view! {
                                    <div class="wishlist-medley-edit-row">
                                        <label class="wishlist-medley-edit-checkbox">
                                            <input
                                                type="checkbox"
                                                aria-label=move || tr!("wishlist-medley-keep-song")
                                                prop:checked=true
                                                on:change=move |_| edit_song_ids.update(|ids| ids.retain(|id| *id != song_id))
                                                disabled=move || rename_pending.get()
                                            />
                                        </label>
                                        <div class="wishlist-medley-order-controls">
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                type="button"
                                                aria-label=move || tr!("wishlist-medley-move-up")
                                                on:click=move |_| edit_song_ids.update(|ids| move_song(ids, song_id, -1))
                                                disabled=move || rename_pending.get() || edit_song_ids.get().first() == Some(&song_id)
                                            >"↑"</button>
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                type="button"
                                                aria-label=move || tr!("wishlist-medley-move-down")
                                                on:click=move |_| edit_song_ids.update(|ids| move_song(ids, song_id, 1))
                                                disabled=move || rename_pending.get() || edit_song_ids.get().last() == Some(&song_id)
                                            >"↓"</button>
                                        </div>
                                        <WishlistRow
                                            item=item.clone()
                                            member_id
                                            can_delete=can_delete_all || item.proposed_by_id == Some(member_id)
                                            is_medley=true
                                            on_feedback_changed
                                            on_deleted
                                            on_updated
                                        />
                                    </div>
                                }
                            }
                        />
                    </div>
                    <div class="wishlist-medley-editor-actions">
                        <button
                            class="btn btn-primary btn-sm"
                            type="button"
                            on:click=save_medley
                            disabled=move || rename_pending.get() || edit_name.get().trim().is_empty() || edit_song_ids.get().len() < 2
                        >
                            {move || if rename_pending.get() { tr!("wishlist-medley-saving-changes") } else { tr!("admin-save-changes") }}
                        </button>
                        <button
                            class="btn btn-ghost btn-sm"
                            type="button"
                            on:click=move |_| cancel_editing.run(())
                            disabled=move || rename_pending.get()
                        >
                            {move || tr!("admin-cancel")}
                        </button>
                    </div>
                </section>
            </Show>
            <Show when=move || expanded.get()>
            <Show when=move || !editing_name.get()>
            <div class="wishlist-medley-songs">
                <For
                    each=move || songs.get()
                key=|item| (item.id, item.medley_position)
                    children=move |item| view! {
                        <WishlistListItem
                            item
                            member_id
                            can_delete_all
                            selecting_medley
                            selected_songs
                            medley_pending
                            on_feedback_changed
                            on_updated
                            on_deleted
                            on_remove_medley_item
                            on_move_medley_item
                            can_manage_medley=false
                        />
                    }
                />
            </div>
            </Show>
            </Show>
        </section>
    }
    .into_any()
}

#[component]
fn WishlistMedleyActions(
    name: Memo<String>,
    pending: RwSignal<bool>,
    confirming: RwSignal<bool>,
    on_edit: Callback<()>,
    on_confirm_disband: Callback<()>,
) -> AnyView {
    let menu_open = RwSignal::new(false);
    view! {
        <div class="wishlist-medley-menu">
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                aria-expanded=move || menu_open.get().to_string()
                on:click=move |_| menu_open.update(|open| *open = !*open)
            >
                {move || tr!("wishlist-medley-actions")}
                <span class="wishlist-medley-menu-arrow" aria-hidden="true">"▾"</span>
            </button>
            <Show when=move || menu_open.get()>
                <button
                    class="wishlist-medley-menu-backdrop"
                    type="button"
                    aria-label=move || tr!("admin-cancel")
                    on:click=move |_| menu_open.set(false)
                ></button>
            <div class="wishlist-medley-menu-items">
                <button
                    class="btn btn-ghost btn-sm"
                    type="button"
                    on:click=move |_| {
                        menu_open.set(false);
                        on_edit.run(());
                    }
                >
                    {move || tr!("wishlist-medley-edit")}
                </button>
                <button
                    class="btn btn-ghost btn-danger btn-sm"
                    type="button"
                    on:click=move |_| {
                        menu_open.set(false);
                        confirming.set(true);
                    }
                    disabled=move || pending.get()
                >
                    {move || tr!("wishlist-medley-disband")}
                </button>
            </div>
            </Show>
        </div>
        <Show when=move || confirming.get()>
            <div class="admin-delete-backdrop">
                <section
                    class="admin-delete-dialog"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || tr!("wishlist-medley-disband-title")
                >
                    <div class="admin-delete-dialog-content">
                        <h2>{move || tr!("wishlist-medley-disband-title")}</h2>
                        <p>{move || tr!("wishlist-medley-disband-confirm", {"name" => name.get()})}</p>
                        <div class="admin-delete-dialog-actions">
                            <button
                                class="btn btn-ghost btn-danger btn-sm"
                                type="button"
                                on:click=move |_| on_confirm_disband.run(())
                                disabled=move || pending.get()
                            >
                                {move || tr!("wishlist-medley-disband")}
                            </button>
                            <button
                                class="btn btn-ghost btn-sm"
                                type="button"
                                on:click=move |_| confirming.set(false)
                                disabled=move || pending.get()
                            >
                                {move || tr!("admin-cancel")}
                            </button>
                        </div>
                    </div>
                </section>
            </div>
        </Show>
    }
    .into_any()
}

#[component]
fn WishlistListItem(
    item: WishlistItem,
    member_id: uuid::Uuid,
    can_delete_all: bool,
    selecting_medley: RwSignal<bool>,
    selected_songs: RwSignal<Vec<uuid::Uuid>>,
    medley_pending: RwSignal<bool>,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_updated: Callback<WishlistItem>,
    on_deleted: Callback<uuid::Uuid>,
    on_remove_medley_item: Callback<(uuid::Uuid, uuid::Uuid)>,
    on_move_medley_item: Callback<(uuid::Uuid, uuid::Uuid, i32)>,
    can_manage_medley: bool,
) -> AnyView {
    let editing_medley_id = expect_context::<RwSignal<Option<uuid::Uuid>>>();
    let editing_medley_song_ids = expect_context::<RwSignal<Vec<uuid::Uuid>>>();
    let can_delete = can_delete_all || item.proposed_by_id == Some(member_id);
    let item_id = item.id;
    let medley_id = item.medley_id;
    let grouped = medley_id.is_some();
    let item_position = item.medley_position;
    let item_group_size = item.medley_size;
    let selected = Memo::new(move |_| selected_songs.get().contains(&item_id));
    let on_select = move |_| {
        selected_songs.update(|selected| {
            if selected.contains(&item_id) {
                selected.retain(|selected_id| *selected_id != item_id);
            } else {
                selected.push(item_id);
            }
        });
    };
    view! {
        <div
            class="wishlist-medley-song"
            class:is-grouped=grouped
            class:is-manageable=grouped && can_manage_medley
        >
            <Show when=move || selecting_medley.get() && !grouped>
                <label class="wishlist-medley-select">
                    <input
                        type="checkbox"
                        aria-label=move || tr!("wishlist-medley-select-song")
                        prop:checked=move || selected.get()
                        on:change=on_select
                        disabled=move || medley_pending.get()
                    />
                </label>
            </Show>
            <Show when=move || editing_medley_id.get().is_some() && !grouped && !selecting_medley.get()>
                <label class="wishlist-medley-select">
                    <input
                        type="checkbox"
                        aria-label=move || tr!("wishlist-medley-select-song")
                        prop:checked=move || editing_medley_song_ids.get().contains(&item_id)
                        on:change=move |_| editing_medley_song_ids.update(|selected| {
                            if selected.contains(&item_id) {
                                selected.retain(|selected_id| *selected_id != item_id);
                            } else {
                                selected.push(item_id);
                            }
                        })
                        disabled=move || medley_pending.get()
                    />
                </label>
            </Show>
            <WishlistRow
                item
                member_id
                can_delete
                on_feedback_changed
                on_deleted
                on_updated
                is_medley=grouped
            />
            <Show when=move || grouped && can_manage_medley>
                <div class="wishlist-medley-order-controls">
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        aria-label=move || tr!("wishlist-medley-move-up")
                        title=move || tr!("wishlist-medley-move-up")
                        on:click=move |_| {
                            if let Some(medley_id) = medley_id {
                                on_move_medley_item.run((medley_id, item_id, -1));
                            }
                        }
                        disabled=move || medley_pending.get() || item_position == Some(0)
                    >
                        "↑"
                    </button>
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        aria-label=move || tr!("wishlist-medley-move-down")
                        title=move || tr!("wishlist-medley-move-down")
                        on:click=move |_| {
                            if let Some(medley_id) = medley_id {
                                on_move_medley_item.run((medley_id, item_id, 1));
                            }
                        }
                        disabled=move || medley_pending.get() || item_position.map(|position| Some(position as i64 + 1) == item_group_size).unwrap_or(true)
                    >
                        "↓"
                    </button>
                </div>
                <button
                    class="btn btn-ghost btn-sm wishlist-medley-remove"
                    type="button"
                    aria-label=move || tr!("wishlist-medley-remove-song")
                    title=move || tr!("wishlist-medley-remove-song")
                    on:click=move |_| {
                        if let Some(medley_id) = medley_id {
                            on_remove_medley_item.run((medley_id, item_id));
                        }
                    }
                    disabled=move || medley_pending.get()
                >
                    "×"
                </button>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistRow(
    item: WishlistItem,
    member_id: uuid::Uuid,
    can_delete: bool,
    is_medley: bool,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_deleted: Callback<uuid::Uuid>,
    on_updated: Callback<WishlistItem>,
) -> AnyView {
    let medley_management_active = expect_context::<RwSignal<bool>>();
    let title = RwSignal::new(item.title.clone());
    let artist = RwSignal::new(item.artist.clone().unwrap_or_default());
    let link = RwSignal::new(item.link.clone().unwrap_or_default());
    let link_title = RwSignal::new(item.link_title.clone());
    let link_badge_label = Memo::new(move |_| {
        let current_link = link.get();
        if current_link.is_empty() {
            None
        } else {
            Some(
                link_title
                    .get()
                    .unwrap_or_else(|| link_label(&current_link)),
            )
        }
    });
    let edit_title = RwSignal::new(item.title.clone());
    let edit_artist = RwSignal::new(item.artist.clone().unwrap_or_default());
    let edit_link = RwSignal::new(item.link.clone().unwrap_or_default());
    let editing = RwSignal::new(false);
    let edit_pending = RwSignal::new(false);
    let edit_error = RwSignal::new(Option::<String>::None);
    let feedbacks = RwSignal::new(item.feedback.clone());
    let delete_pending = RwSignal::new(false);
    let delete_error = RwSignal::new(Option::<String>::None);
    let confirming_delete = RwSignal::new(false);
    let item_id = item.id;
    let confirm_title = RwSignal::new(item.title.clone());
    let update_song = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        edit_pending.set(true);
        edit_error.set(None);
        let new_title = edit_title.get();
        let new_artist = edit_artist.get();
        let new_link = edit_link.get();
        leptos::task::spawn_local(async move {
            match wishlist_update(item_id, new_title, new_artist, new_link).await {
                Ok(updated) => {
                    title.set(updated.title.clone());
                    artist.set(updated.artist.clone().unwrap_or_default());
                    link.set(updated.link.clone().unwrap_or_default());
                    link_title.set(updated.link_title.clone());
                    confirm_title.set(updated.title.clone());
                    on_updated.run(updated);
                    editing.set(false);
                }
                Err(message) => edit_error.set(Some(message.to_string())),
            }
            edit_pending.set(false);
        });
    };
    let cancel_edit = Callback::new(move |_: ()| {
        edit_title.set(title.get());
        edit_artist.set(artist.get());
        edit_link.set(link.get());
        edit_error.set(None);
        editing.set(false);
    });
    let delete_item = move |_| {
        delete_pending.set(true);
        delete_error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_delete(item_id).await {
                Ok(()) => on_deleted.run(item_id),
                Err(message) => delete_error.set(Some(message.to_string())),
            }
            confirming_delete.set(false);
            delete_pending.set(false);
        });
    };

    view! {
        <article
            class="wishlist-item"
            class:is-medley=is_medley
            class:is-editing=move || editing.get()
            class:needs-feedback=move || {
                !feedbacks
                    .get()
                    .iter()
                    .any(|feedback| feedback.member_id == member_id)
            }
            data-testid="wishlist-item"
        >
            <div class="wishlist-item-line">
                <div class="wishlist-song-summary">
                    <h3>{move || title.get()}</h3>
                    <Show when=move || !artist.get().is_empty()>
                        <span class="wishlist-artist">{move || artist.get()}</span>
                    </Show>
                    <Show when=move || link_badge_label.get().is_some()>
                        <a
                            class="wishlist-badge wishlist-link"
                            href=move || link.get()
                            target="_blank"
                            rel="noopener noreferrer"
                            title=move || link_badge_label.get().unwrap_or_default()
                        >
                            {move || link_badge_label.get().unwrap_or_default()}
                        </a>
                    </Show>
                </div>
                <Show when=move || !medley_management_active.get()>
                    <div class="wishlist-item-actions">
                        <WishlistFeedbackControl
                            item_id
                            member_id
                            is_medley
                            feedbacks
                            on_feedback_changed
                        />
                        {if can_delete {
                            view! {
                                <button
                                    class="btn btn-ghost btn-sm wishlist-edit-button"
                                    type="button"
                                    data-testid="wishlist-edit"
                                    on:click=move |_| {
                                        if editing.get() {
                                            edit_title.set(title.get());
                                            edit_artist.set(artist.get());
                                            edit_link.set(link.get());
                                            edit_error.set(None);
                                            editing.set(false);
                                        } else {
                                            edit_title.set(title.get());
                                            edit_artist.set(artist.get());
                                            edit_link.set(link.get());
                                            edit_error.set(None);
                                            editing.set(true);
                                        }
                                    }
                                    disabled=move || edit_pending.get()
                                >
                                    {move || if editing.get() { tr!("wishlist-close-edit") } else { tr!("wishlist-edit") }}
                                </button>
                            }
                            .into_any()
                        } else {
                            ().into_any()
                        }
                        }
                    </div>
                </Show>
            </div>
            <Show when=move || editing.get()>
                <button
                    class="wishlist-edit-backdrop"
                    type="button"
                    aria-label=move || tr!("wishlist-close-edit")
                    on:click=move |_| cancel_edit.run(())
                    disabled=move || edit_pending.get()
                ></button>
                <form
                    class="wishlist-edit-form"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || tr!("wishlist-edit")
                    on:submit=update_song
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if event.key() == "Escape" && !edit_pending.get() {
                            cancel_edit.run(());
                        }
                    }
                >
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-song-label")}</span>
                        <input
                            class="input"
                            data-testid="wishlist-edit-title"
                            maxlength="120"
                            required
                            prop:value=move || edit_title.get()
                            on:input=move |event| edit_title.set(event_target_value(&event))
                            disabled=move || edit_pending.get()
                        />
                    </label>
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-artist-label")}</span>
                        <input
                            class="input"
                            data-testid="wishlist-edit-artist"
                            maxlength="120"
                            prop:value=move || edit_artist.get()
                            on:input=move |event| edit_artist.set(event_target_value(&event))
                            disabled=move || edit_pending.get()
                        />
                    </label>
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-link-label")}</span>
                        <input
                            class="input"
                            data-testid="wishlist-edit-link"
                            type="url"
                            maxlength="2048"
                            prop:value=move || edit_link.get()
                            on:input=move |event| edit_link.set(event_target_value(&event))
                            disabled=move || edit_pending.get()
                        />
                    </label>
                    <div class="wishlist-edit-actions">
                        <button class="btn btn-primary btn-sm" type="submit" data-testid="wishlist-save-edit" disabled=move || edit_pending.get()>
                            {move || if edit_pending.get() { tr!("wishlist-edit-saving") } else { tr!("wishlist-save-edit") }}
                        </button>
                        <button class="btn btn-ghost btn-sm" type="button" data-testid="wishlist-cancel-edit" on:click=move |_| cancel_edit.run(()) disabled=move || edit_pending.get()>
                            {move || tr!("admin-cancel")}
                        </button>
                        {if can_delete {
                            view! {
                                <button
                                    class="btn btn-ghost btn-danger btn-sm wishlist-delete-button"
                                    type="button"
                                    data-testid="wishlist-delete"
                                    on:click=move |_| confirming_delete.set(true)
                                    disabled=move || edit_pending.get()
                                >
                                    {move || tr!("wishlist-delete")}
                                </button>
                            }
                            .into_any()
                        } else {
                            ().into_any()
                        }}
                    </div>
                    <Show when=move || edit_error.get().is_some()>
                        <p class="form-error" role="alert" data-testid="wishlist-edit-error">{move || edit_error.get().unwrap_or_default()}</p>
                    </Show>
                </form>
            </Show>
            <Show when=move || confirming_delete.get()>
                <div class="admin-delete-backdrop">
                    <section
                        class="admin-delete-dialog"
                        role="dialog"
                        aria-modal="true"
                        aria-label=move || tr!("wishlist-delete-title")
                        on:keydown=move |event: leptos::ev::KeyboardEvent| {
                            if event.key() == "Escape" && !delete_pending.get() {
                                confirming_delete.set(false);
                            }
                        }
                    >
                        <div class="admin-delete-dialog-content">
                            <h2>{move || tr!("wishlist-delete-title")}</h2>
                            <p>{move || tr!("wishlist-confirm-delete", {"name" => confirm_title.get()})}</p>
                            <Show when=move || delete_error.get().is_some()>
                                <p class="form-error" role="alert">{move || delete_error.get().unwrap_or_default()}</p>
                            </Show>
                            <div class="admin-delete-dialog-actions">
                                <button
                                    class="btn btn-ghost btn-danger btn-sm"
                                    type="button"
                                    data-testid="wishlist-confirm-delete"
                                    on:click=delete_item
                                    disabled=move || delete_pending.get()
                                >
                                    {move || if delete_pending.get() {
                                        tr!("wishlist-deleting")
                                    } else {
                                        tr!("wishlist-delete")
                                    }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    type="button"
                                    data-testid="wishlist-cancel-delete"
                                    on:click=move |_| confirming_delete.set(false)
                                    disabled=move || delete_pending.get()
                                >
                                    {move || tr!("admin-cancel")}
                                </button>
                            </div>
                        </div>
                    </section>
                </div>
            </Show>
        </article>
    }
    .into_any()
}

#[component]
fn WishlistFeedbackControl(
    item_id: uuid::Uuid,
    member_id: uuid::Uuid,
    is_medley: bool,
    feedbacks: RwSignal<Vec<WishlistFeedback>>,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
) -> AnyView {
    let selected = RwSignal::new(
        feedbacks
            .get_untracked()
            .iter()
            .find(|feedback| feedback.member_id == member_id)
            .map(|feedback| feedback.category),
    );
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let feedback_open = RwSignal::new(false);
    let save_feedback = Callback::new(move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let category = selected.get();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_set_feedback(item_id, category).await {
                Ok(feedback) => {
                    feedbacks.update(|feedbacks| {
                        feedbacks.retain(|existing| existing.member_id != member_id);
                        if let Some(feedback) = feedback.clone() {
                            feedbacks.push(feedback);
                            feedbacks.sort_by(|left, right| {
                                left.member_name
                                    .to_lowercase()
                                    .cmp(&right.member_name.to_lowercase())
                            });
                        }
                    });
                    on_feedback_changed.run((item_id, feedback));
                    feedback_open.set(false);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    });

    view! {
        <div class="wishlist-feedback-details" class:is-open=move || feedback_open.get()>
            <button
                class="wishlist-feedback-toggle"
                type="button"
                aria-expanded=move || feedback_open.get().to_string()
                on:click=move |_| feedback_open.update(|open| *open = !*open)
            >
                {move || tr!("wishlist-feedback-summary", {"count" => feedbacks.get().len().to_string()})}
            </button>
            <Show when=move || feedback_open.get()>
                <button
                    class="wishlist-feedback-backdrop"
                    type="button"
                    aria-label=move || tr!("wishlist-feedback-close")
                    on:click=move |_| feedback_open.set(false)
                >                </button>
                <div class="wishlist-feedback-content">
                    <WishlistFeedbackBadges feedbacks />
                    <WishlistFeedbackForm
                        feedbacks
                        member_id
                        is_medley
                        selected
                        pending
                        error
                        on_save=save_feedback
                    />
                </div>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistMedleyFeedbackControl(
    medley_id: uuid::Uuid,
    member_id: uuid::Uuid,
    feedbacks: RwSignal<Vec<WishlistMedleyFeedback>>,
) -> AnyView {
    let own_feedback = Memo::new(move |_| {
        feedbacks
            .get()
            .into_iter()
            .find(|feedback| feedback.medley_id == medley_id && feedback.member_id == member_id)
    });
    let selected = RwSignal::new(
        own_feedback
            .get_untracked()
            .map(|feedback| feedback.category),
    );
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let feedback_open = RwSignal::new(false);
    let save_feedback = Callback::new(move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let category = selected.get();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_set_medley_feedback(medley_id, category).await {
                Ok(feedback) => {
                    feedbacks.update(|feedbacks| {
                        feedbacks.retain(|existing| {
                            existing.medley_id != medley_id || existing.member_id != member_id
                        });
                        if let Some(feedback) = feedback {
                            feedbacks.push(feedback);
                            feedbacks.sort_by(|left, right| {
                                left.member_name
                                    .to_lowercase()
                                    .cmp(&right.member_name.to_lowercase())
                            });
                        }
                    });
                    feedback_open.set(false);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    });

    view! {
        <div class="wishlist-feedback-details" class:is-open=move || feedback_open.get()>
            <button
                class="wishlist-feedback-toggle"
                type="button"
                aria-expanded=move || feedback_open.get().to_string()
                on:click=move |_| feedback_open.update(|open| *open = !*open)
            >
                {move || tr!("wishlist-feedback-summary", {
                    "count" => feedbacks
                        .get()
                        .iter()
                        .filter(|feedback| feedback.medley_id == medley_id)
                        .count()
                        .to_string()
                })}
            </button>
            <Show when=move || feedback_open.get()>
                <button
                    class="wishlist-feedback-backdrop"
                    type="button"
                    aria-label=move || tr!("wishlist-feedback-close")
                    on:click=move |_| feedback_open.set(false)
                ></button>
                <WishlistMedleyFeedbackPanel
                    medley_id
                    member_id
                    feedbacks
                    selected
                    pending
                    error
                    on_save=save_feedback
                />
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistMedleyFeedbackPanel(
    medley_id: uuid::Uuid,
    member_id: uuid::Uuid,
    feedbacks: RwSignal<Vec<WishlistMedleyFeedback>>,
    selected: RwSignal<Option<WishlistCategory>>,
    pending: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_save: Callback<leptos::ev::SubmitEvent>,
) -> AnyView {
    view! {
        <div class="wishlist-feedback-content">
            <WishlistMedleyFeedbackBadges medley_id feedbacks />
            <WishlistMedleyFeedbackForm
                medley_id
                member_id
                feedbacks
                selected
                pending
                error
                on_save
            />
        </div>
    }
    .into_any()
}

#[component]
fn WishlistMedleyFeedbackBadges(
    medley_id: uuid::Uuid,
    feedbacks: RwSignal<Vec<WishlistMedleyFeedback>>,
) -> AnyView {
    view! {
        <div class="wishlist-feedback-badges">
            <For
                each=move || {
                    feedbacks
                        .get()
                        .into_iter()
                        .filter(|feedback| feedback.medley_id == medley_id)
                        .collect::<Vec<_>>()
                }
                key=|feedback| feedback.member_id
                children=move |feedback| {
                    let text = format!(
                        "{}: {}",
                        feedback.member_name,
                        category_label(feedback.category)
                    );
                    view! { <span class="wishlist-badge">{text}</span> }
                }
            />
            <Show when=move || {
                !feedbacks
                    .get()
                    .iter()
                    .any(|feedback| feedback.medley_id == medley_id)
            }>
                <span class="muted">{move || tr!("wishlist-feedback-empty")}</span>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistMedleyFeedbackForm(
    medley_id: uuid::Uuid,
    member_id: uuid::Uuid,
    feedbacks: RwSignal<Vec<WishlistMedleyFeedback>>,
    selected: RwSignal<Option<WishlistCategory>>,
    pending: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_save: Callback<leptos::ev::SubmitEvent>,
) -> AnyView {
    let has_own_feedback = Memo::new(move |_| {
        feedbacks
            .get()
            .iter()
            .any(|feedback| feedback.medley_id == medley_id && feedback.member_id == member_id)
    });
    view! {
        <div>
            <form class="wishlist-feedback-form" on:submit=move |event| on_save.run(event)>
                <label class="field">
                    <span class="field-label">{move || tr!("wishlist-medley-feedback-label")}</span>
                    <select
                        class="select"
                        prop:value=move || selected.get().map(|value| value.to_string()).unwrap_or_default()
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            selected.set(if value.is_empty() { None } else { value.parse().ok() });
                        }
                        disabled=move || pending.get()
                    >
                        <Show when=move || has_own_feedback.get()>
                            <option value="">{move || tr!("wishlist-feedback-none")}</option>
                        </Show>
                        <option value="song">{move || tr!("wishlist-category-song")}</option>
                        <option value="medley">{move || tr!("wishlist-category-medley")}</option>
                        <option value="try">{move || tr!("wishlist-category-try")}</option>
                        <option value="unplayable">{move || tr!("wishlist-category-unplayable")}</option>
                        <option value="other_ordering">{move || tr!("wishlist-category-other-ordering")}</option>
                        <option value="not_fit_medley">{move || tr!("wishlist-category-not-fit-medley")}</option>
                    </select>
                </label>
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    disabled=move || pending.get() || (selected.get().is_none() && !has_own_feedback.get())
                >
                    {move || if pending.get() {
                        tr!("wishlist-feedback-saving")
                    } else {
                        tr!("wishlist-feedback-save")
                    }}
                </button>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistFeedbackBadges(feedbacks: RwSignal<Vec<WishlistFeedback>>) -> AnyView {
    view! {
        <div class="wishlist-feedback-badges">
            <For
                each=move || feedbacks.get()
                key=|feedback| feedback.member_id
                children=move |feedback| {
                    let label = category_label(feedback.category);
                    let text = format!("{}: {}", feedback.member_name, label);
                    view! { <span class="wishlist-badge">{text}</span> }
                }
            />
            <Show when=move || feedbacks.get().is_empty()>
                <span class="muted">{move || tr!("wishlist-feedback-empty")}</span>
            </Show>
        </div>
    }
    .into_any()
}

#[component]
fn WishlistFeedbackForm(
    feedbacks: RwSignal<Vec<WishlistFeedback>>,
    member_id: uuid::Uuid,
    is_medley: bool,
    selected: RwSignal<Option<WishlistCategory>>,
    pending: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_save: Callback<leptos::ev::SubmitEvent>,
) -> AnyView {
    let has_own_feedback = Memo::new(move |_| {
        feedbacks
            .get()
            .iter()
            .any(|feedback| feedback.member_id == member_id)
    });
    view! {
        <div>
            <form
                class="wishlist-feedback-form"
                on:submit=move |event| on_save.run(event)
            >
                <label class="field">
                    <span class="field-label">{move || tr!("wishlist-feedback-label")}</span>
                    <select
                        class="select"
                        data-testid="wishlist-feedback-select"
                        prop:value=move || selected.get().map(|value| value.to_string()).unwrap_or_default()
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            selected.set(if value.is_empty() {
                                None
                            } else {
                                value.parse::<WishlistCategory>().ok()
                            });
                        }
                        disabled=move || pending.get()
                    >
                        <Show when=move || has_own_feedback.get()>
                            <option value="">{move || tr!("wishlist-feedback-none")}</option>
                        </Show>
                        <option value="song">{move || tr!("wishlist-category-song")}</option>
                        <Show when=move || !is_medley>
                            <option value="medley">{move || tr!("wishlist-category-medley")}</option>
                            <option value="try">{move || tr!("wishlist-category-try")}</option>
                        </Show>
                        <option value="unplayable">{move || tr!("wishlist-category-unplayable")}</option>
                        <Show when=move || is_medley>
                            <option value="other_ordering">{move || tr!("wishlist-category-other-ordering")}</option>
                            <option value="not_fit_medley">{move || tr!("wishlist-category-not-fit-medley")}</option>
                        </Show>
                    </select>
                </label>
                <button
                    class="btn btn-primary btn-sm"
                    type="submit"
                    data-testid="wishlist-feedback-save"
                    disabled=move || pending.get() || (selected.get().is_none() && !has_own_feedback.get())
                >
                    {move || if pending.get() {
                        tr!("wishlist-feedback-saving")
                    } else {
                        tr!("wishlist-feedback-save")
                    }}
                </button>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
    .into_any()
}

fn category_label(category: WishlistCategory) -> String {
    match category {
        WishlistCategory::Song => tr!("wishlist-category-song"),
        WishlistCategory::Medley => tr!("wishlist-category-medley"),
        WishlistCategory::Try => tr!("wishlist-category-try"),
        WishlistCategory::Unplayable => tr!("wishlist-category-unplayable"),
        WishlistCategory::OtherOrdering => tr!("wishlist-category-other-ordering"),
        WishlistCategory::NotFitMedley => tr!("wishlist-category-not-fit-medley"),
    }
}

fn link_label(link: &str) -> String {
    let host = link
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(link)
        .split('/')
        .next()
        .unwrap_or_default()
        .trim_start_matches("www.")
        .to_ascii_lowercase();
    if host == "youtube.com" || host == "youtu.be" || host.ends_with(".youtube.com") {
        "YouTube".to_string()
    } else {
        host.to_string()
    }
}

fn sort_wishlist_items(items: &mut [WishlistItem]) {
    let mut group_titles = std::collections::HashMap::new();
    for item in items.iter() {
        if let Some(medley_id) = item.medley_id {
            group_titles.entry(medley_id).or_insert_with(|| {
                item.medley_name
                    .clone()
                    .unwrap_or_else(|| item.title.clone())
            });
        }
    }
    items.sort_by(|left, right| {
        let left_group_title = left
            .medley_id
            .and_then(|id| group_titles.get(&id))
            .unwrap_or(&left.title);
        let right_group_title = right
            .medley_id
            .and_then(|id| group_titles.get(&id))
            .unwrap_or(&right.title);
        left_group_title
            .to_lowercase()
            .cmp(&right_group_title.to_lowercase())
            .then_with(|| match (left.medley_id, right.medley_id) {
                (Some(left_id), Some(right_id)) if left_id == right_id => {
                    left.medley_position.cmp(&right.medley_position)
                }
                (Some(left_id), Some(right_id)) => left_id.cmp(&right_id),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| left.title.cmp(&right.title))
    });
}

fn move_song(ids: &mut [uuid::Uuid], song_id: uuid::Uuid, delta: isize) {
    let Some(position) = ids.iter().position(|id| *id == song_id) else {
        return;
    };
    let next = position as isize + delta;
    if (0..ids.len() as isize).contains(&next) {
        ids.swap(position, next as usize);
    }
}
