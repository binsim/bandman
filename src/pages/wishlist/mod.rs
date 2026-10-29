mod modify_song;
mod wishlist_feedback_form;

use crate::{
    models::{
        Member, MemberRole, WISHLIST_MUSICAL_KEYS, WishlistCategory, WishlistFeedback, WishlistItem,
    },
    pages::wishlist::modify_song::EditSong,
};
use leptos::prelude::*;
use leptos_fluent::tr;

use crate::pages::wishlist::wishlist_feedback_form::WishlistFeedbackControl;
use modify_song::CreateSong;

#[server(WishlistList, "/api")]
async fn wishlist_list() -> Result<Vec<WishlistItem>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_member().await?;
    let items = WishlistItem::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(items)
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

#[component]
pub fn WishlistPage() -> impl IntoView {
    let member = expect_context::<Resource<Option<crate::models::Member>>>();
    let wishlist = Resource::new(|| (), |_| async move { wishlist_list().await });
    view! {
        <Suspense fallback=|| view! {
            <section>
                <h1>{move || tr!("nav-wishlist")}</h1>
                <p class="muted">{move || tr!("wishlist-loading")}</p>
            </section>
        }>
            {move || match member.get().flatten() {
                Some(member) => {
                    match wishlist.get() {
                        Some(Ok(items)) => view! {
                            <WishlistLoaded
                                initial_items=items
                                current_member=member
                            />
                        }.into_any(),
                        Some(Err(error)) => view! {
                            <section>
                                <h1>{tr!("nav-wishlist")}</h1>
                                <p class="from-error" role="alert">{error.to_string()}</p>
                            </section>
                        }.into_any(),
                        None => ().into_any(),
                    }
                }
                None => view! {
                    <section>
                        <h1>{tr!("nav-wishlist")}</h1>
                        <p>{tr!("wishlist-sign-in")}</p>
                        <a class="btn btn-primary" href="/login">{move || tr!("nav-login")}</a>
                    </section>
                }.into_any(),
            }}
        </Suspense>
    }
}

#[component]
fn WishlistLoaded(initial_items: Vec<WishlistItem>, current_member: Member) -> AnyView {
    let items = RwSignal::new(initial_items);
    let medley_management_active = RwSignal::new(false);
    provide_context(medley_management_active);
    let editing_medley_id = RwSignal::new(Option::<uuid::Uuid>::None);
    provide_context(editing_medley_id);
    let editing_medley_song_ids = RwSignal::new(Vec::<uuid::Uuid>::new());
    provide_context(editing_medley_song_ids);
    let show_needs_feedback = RwSignal::new(true);
    #[cfg(feature = "hydrate")]
    let filter_storage_key = format!("bandman-wishlist-needs-feedback-{0}", current_member.id);
    #[cfg(feature = "hydrate")]
    {
        let load_filter_storage_key = filter_storage_key.clone();
        Effect::new(move |_| {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
                && let Ok(Some(value)) = storage.get_item(&load_filter_storage_key)
            {
                show_needs_feedback.set(value != "false");
            }
        });
    }
    let on_filter_changed = Callback::new(move |show: bool| {
        show_needs_feedback.set(show);
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window()
                && let Ok(Some(storage)) = window.local_storage()
            {
                let _ = storage.set_item(&filter_storage_key, if show { "true" } else { "false" });
            }
        }
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
                        .any(|feedback| feedback.member_id == current_member.id)
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
                                .any(|feedback| feedback.member_id == current_member.id)
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
                                item.medley_created_by_id = Some(current_member.id);
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
        <section>
            <p class="eyebrow">{move || tr!("wishlist-eyebrow")}</p>
            <h1>{move || tr!("nav-wishlist")}</h1>
            <p class="lead">{move || tr!("wishlist-lead")}</p>
        </section>
        <CreateSong items on_created=on_song_created />
        <WishlistListing
            items
            visible_items
            current_member
            show_needs_feedback
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
    }
    .into_any()
}

#[derive(Clone, PartialEq, Eq)]
enum WishlistDisplayEntry {
    Song(Box<WishlistItem>),
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
            entries.push(WishlistDisplayEntry::Song(Box::new(item)));
            continue;
        };
        if let std::collections::hash_map::Entry::Vacant(entry) = medley_indices.entry(medley_id) {
            let index = entries.len();
            entry.insert(index);
            entries.push(WishlistDisplayEntry::Medley { id: medley_id });
        }
    }
    entries
}

#[component]
fn WishlistListing(
    items: RwSignal<Vec<WishlistItem>>,
    visible_items: Memo<Vec<WishlistItem>>,
    current_member: Member,
    show_needs_feedback: RwSignal<bool>,
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
        <section class="card">
            <header>
                <div style="display:flex; justify-content:space-between;width:100%">
                    <h2 id="wishlist-list-title">{move || tr!("wishlist-list-title")}
                        <span class="muted">{move || tr!("wishlist-count", {"count" => items.get().len().to_string()})}</span>
                    </h2>

                    <Show when=move || !selecting_medley.get()>
                        <button
                            class="btn btn-primary btn-sm"
                            type="button"
                            on:click=move |_| on_start_creating_medley.run(())
                        >
                            {move || tr!("wishlist-medley-start")}
                        </button>
                    </Show>
                </div>
            </header>

            <Show when=move || !selecting_medley.get()>
                <div class="wishlist-list-header-actions">
                    <div class="wishlist-filter" role="group" aria-label=move || tr!("wishlist-filter-label")>
                        <button
                            class="wishlist-filter-button"
                            class:is-active=move || show_needs_feedback.get()
                            type="button"
                            aria-pressed=move || show_needs_feedback.get().to_string()
                            on:click=move |_| on_filter_changed.run(true)
                        >
                            {move || tr!("wishlist-filter-needs-feedback")}
                        </button>
                        <button
                            class="wishlist-filter-button"
                            class:is-active=move || !show_needs_feedback.get()
                            type="button"
                            aria-pressed=move || (!show_needs_feedback.get()).to_string()
                            on:click=move |_| on_filter_changed.run(false)
                        >
                            {move || tr!("wishlist-filter-all")}
                        </button>
                    </div>
                </div>
            </Show>
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
            <Show
                when=move || !items.get().is_empty()
                fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-empty")}</p> }
            >
                <Show
                    clone:current_member
                    when=move || !visible_items.get().is_empty()
                    fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-no-feedback-pending")}</p> }
                >
                    <div class="wishlist-items">
                        <WishlistItemList
                            items
                            entries
                            current_member=current_member.clone()
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
        <Show when=move || selecting_medley.get()>
            <div class="wishlist-medley-controls">
                <div class="wishlist-medley-selection-actions">
                    <span class="muted">
                        {move || tr!("wishlist-medley-selected", {"count" => selected_songs.get().len().to_string()})}
                    </span>
                    <input
                        class="input wishlist-medley-name-input"
                        maxlength="120"
                        style="max-width:22rem"
                        aria-label=move || tr!("wishlist-medley-name")
                        placeholder=move || tr!("wishlist-medley-name")
                        prop:value=move || new_medley_name.get()
                        on:input=move |event| new_medley_name.set(event_target_value(&event))
                        disabled=move || medley_pending.get()
                    />
                </div>
                <Show when=move || selected_songs.get().len() < 2 >
                    <span class="form-error">
                        {move || tr!("wishlist-medley-create-hint")}
                    </span>
                </Show>
                <div class="wishlist-medley-selection-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        type="button"
                        on:click=move |_| on_create_medley.run(())
                        disabled=move || medley_pending.get() || selected_songs.get().len() < 2 || new_medley_name.get().trim().is_empty()
                    >
                        {move || if medley_pending.get() { tr!("wishlist-medley-saving") } else { tr!("wishlist-medley-create") }}
                    </button>
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
                </div>
            </div>
        </Show>
    }
    .into_any()
}

#[component]
fn WishlistItemList(
    items: RwSignal<Vec<WishlistItem>>,
    entries: Memo<Vec<WishlistDisplayEntry>>,
    current_member: Member,
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
                        .retain(|existing| existing.member_id != current_member.id);
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
                        current_member=current_member.clone()
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
    current_member: Member,
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
                item=*item
                current_member
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
                current_member
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
    current_member: Member,
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
    let songs = Memo::new(move |_| {
        let mut songs: Vec<_> = items
            .get()
            .into_iter()
            .filter(|item| item.medley_id == Some(id))
            .collect();
        songs.sort_by_key(|item| item.medley_position);
        songs
    });
    let medley_creator = Memo::new(move |_| {
        songs
            .get()
            .first()
            .and_then(|item| item.medley_created_by_id)
    });
    let can_manage =
        current_member.role == MemberRole::Admin || medley_creator.get() == Some(current_member.id);
    let member_store = StoredValue::new(current_member);
    let medley_management_active = expect_context::<RwSignal<bool>>();
    let medley_name = Memo::new(move |_| {
        songs
            .get()
            .first()
            .and_then(|item| item.medley_name.clone())
            .unwrap_or_default()
    });
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
            <header class="collapsable">
                <div style="display: flex; align-items:center; gap: var(--space-2) var(--space-3);width:100%">
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
                    <h3>{move || medley_name.get()}
                        <span class="muted">
                            {move || tr!("wishlist-medley-song-count", {"count" => song_count.get().to_string()})}
                        </span>
                    </h3>
                    <div class="wishlist-medley-header-actions">
                        // <Show
                        //     clone:current_member
                        //     when=move || !medley_management_active.get()
                        // >
                            // <WishlistFeedbackControl
                            //     item_id=id
                            //     current_member=current_member.clone()
                            //     is_medley=true
                            // />
                        // </Show>
                        <Show when=move || !medley_management_active.get() && can_manage && !selecting_medley.get()>
                            <WishlistMedleyActions
                                name=medley_name
                                pending=medley_pending
                                confirming=confirm_disband
                                on_edit=start_editing
                                on_confirm_disband=confirm_ungroup
                            />
                        </Show>
                    </div>
                </div>
            </header>
            <Show when=move || rename_error.get().is_some()>
                <p class="form-error" role="alert">{move || rename_error.get().unwrap_or_default()}</p>
            </Show>
            <Show
                when=move || editing_name.get()
            >
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
                                            current_member=member_store.read_value().clone()
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
            <Show
                when=move || expanded.get() && !editing_name.get()
            >
                <div class="wishlist-medley-songs">
                    <For
                        each=move || songs.get()
                        key=|item| (item.id, item.medley_position)
                        children=move |item| view! {
                            <WishlistListItem
                                item
                                current_member=member_store.read_value().clone()
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
    current_member: Member,
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
        <div class="wishlist-medley-song">
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
                current_member
                on_feedback_changed
                on_deleted
                on_updated
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
    current_member: Member,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_deleted: Callback<uuid::Uuid>,
    on_updated: Callback<WishlistItem>,
) -> AnyView {
    let item = RwSignal::new(item);

    let medley_management_active = expect_context::<RwSignal<bool>>();
    let member_id = current_member.id;

    view! {
        <article
            class="wishlist-item"
            class:needs-feedback=move || {
                !item.get().feedback.iter().any(|feedback| feedback.member_id == member_id)
            }
            data-testid="wishlist-item"
        >
            <div class="wishlist-item-line">
                <div class="wishlist-song-summary">
                    <div>
                        <h3>{move || item.get().title}</h3>
                        <Show when=move || item.get().artist.is_some()>
                            <span class="wishlist-artist">{move || item.get().artist}</span>
                        </Show>
                    </div>
                    <div>
                        <Show when=move || item.get().link_title.is_some()>
                            <a
                                class="wishlist-badge wishlist-link"
                                href=move || item.get().link
                                target="_blank"
                                rel="noopener noreferrer"
                                title=move || item.get().link_title.unwrap_or_default()
                            >
                                {move || item.get().link_title.unwrap_or_default()}
                            </a>
                        </Show>
                        <Show when=move || item.get().tempo.is_some()>
                            <span class="wishlist-badge wishlist-music-detail">
                                {move || tr!("wishlist-tempo-badge", {"tempo" => item.get().tempo.unwrap_or_default().to_string()})}
                            </span>
                        </Show>
                        <Show when=move || item.get().musical_key.is_some()>
                            <span class="wishlist-badge wishlist-music-detail">{move || item.get().musical_key.unwrap_or_default()}</span>
                        </Show>
                    </div>
                </div>
                <Show
                    clone:current_member
                    when=move || !medley_management_active.get()
                >
                    <div class="wishlist-item-actions">
                        <WishlistFeedbackControl
                            item
                            current_member=current_member.clone()
                            on_feedback_changed
                        />
                        <EditSong
                            item
                            on_updated
                            current_member=current_member.clone()
                            on_deleted
                        />
                    </div>
                </Show>
            </div>
        </article>
    }
    .into_any()
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
