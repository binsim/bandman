mod create_song;

use crate::models::{Member, WishlistCategory, WishlistFeedback, WishlistItem};
use leptos::prelude::*;
use leptos_fluent::tr;

use create_song::CreateSong;

#[server(WishlistList, "/api")]
async fn wishlist_list() -> Result<Vec<WishlistItem>, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_member().await?;
    WishlistItem::list(&pool)
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
                Some(Ok(items)) => view! {
                    <WishlistLoaded
                        initial_items=items
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
    member_id: uuid::Uuid,
    can_delete_all: bool,
) -> impl IntoView {
    let items = RwSignal::new(initial_items);
    let show_needs_feedback = RwSignal::new(true);
    let visible_items = Memo::new(move |_| {
        let items = items.get();
        if show_needs_feedback.get() {
            items
                .into_iter()
                .filter(|item| {
                    !item
                        .feedback
                        .iter()
                        .any(|feedback| feedback.member_id == member_id)
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

    view! {
        <section class="wishlist-page">
            <header class="wishlist-header">
                <p class="eyebrow">{move || tr!("wishlist-eyebrow")}</p>
                <h1>{move || tr!("nav-wishlist")}</h1>
                <p class="lead">{move || tr!("wishlist-lead")}</p>
            </header>

            <CreateSong items on_created=on_song_created />

            <section class="wishlist-list" aria-labelledby="wishlist-list-title">
                <div class="wishlist-list-header">
                    <h2 id="wishlist-list-title">{move || tr!("wishlist-list-title")}</h2>
                    <div class="wishlist-list-header-actions">
                        <span class="muted">{move || tr!("wishlist-count", {"count" => items.get().len().to_string()})}</span>
                        <div class="wishlist-filter" role="group" aria-label=move || tr!("wishlist-filter-label")>
                            <button
                                class="wishlist-filter-button"
                                class:is-active=move || show_needs_feedback.get()
                                type="button"
                                aria-pressed=move || show_needs_feedback.get().to_string()
                                on:click=move |_| show_needs_feedback.set(true)
                            >
                                {move || tr!("wishlist-filter-needs-feedback")}
                            </button>
                            <button
                                class="wishlist-filter-button"
                                class:is-active=move || !show_needs_feedback.get()
                                type="button"
                                aria-pressed=move || (!show_needs_feedback.get()).to_string()
                                on:click=move |_| show_needs_feedback.set(false)
                            >
                                {move || tr!("wishlist-filter-all")}
                            </button>
                        </div>
                    </div>
                </div>
                <Show
                    when=move || !items.get().is_empty()
                    fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-empty")}</p> }
                >
                    <Show
                        when=move || !visible_items.get().is_empty()
                        fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-no-feedback-pending")}</p> }
                    >
                    <div class="wishlist-items">
                        {
                            let on_feedback_changed = Callback::new(move |(item_id, feedback): (uuid::Uuid, Option<WishlistFeedback>)| {
                                items.update(|items| {
                                    if let Some(item) = items.iter_mut().find(|item| item.id == item_id) {
                                        item.feedback.retain(|existing| existing.member_id != member_id);
                                        if let Some(feedback) = feedback {
                                            item.feedback.push(feedback);
                                        }
                                    }
                                });
                            });
                            let on_updated = Callback::new(move |updated: WishlistItem| {
                                items.update(|items| {
                                    if let Some(item) = items.iter_mut().find(|item| item.id == updated.id) {
                                        let mut updated = updated.clone();
                                        updated.feedback = item.feedback.clone();
                                        *item = updated;
                                    }
                                    sort_wishlist_items(items);
                                });
                            });
                            view! {
                        <For
                            each=move || visible_items.get()
                            key=|item| item.id
                            children=move |item| {
                                let can_delete = can_delete_all || item.proposed_by_id == Some(member_id);
                                let on_deleted = Callback::new(move |item_id| {
                                    items.update(|items| items.retain(|item| item.id != item_id));
                                });
                                view! {
                                    <WishlistRow
                                        item
                                        member_id
                                        can_delete
                                        on_feedback_changed=on_feedback_changed
                                        on_deleted
                                        on_updated
                                    />
                                }
                            }
                        />
                            }
                        }
                    </div>
                    </Show>
                </Show>
            </section>
        </section>
    }
}

#[component]
fn WishlistRow(
    item: WishlistItem,
    member_id: uuid::Uuid,
    can_delete: bool,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_deleted: Callback<uuid::Uuid>,
    on_updated: Callback<WishlistItem>,
) -> impl IntoView {
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
    let selected = RwSignal::new(
        item.feedback
            .iter()
            .find(|feedback| feedback.member_id == member_id)
            .map(|feedback| feedback.category),
    );
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let feedback_open = RwSignal::new(false);
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
    let cancel_edit = move |_| {
        edit_title.set(title.get());
        edit_artist.set(artist.get());
        edit_link.set(link.get());
        edit_error.set(None);
        editing.set(false);
    };
    let save_feedback = move |event: leptos::ev::SubmitEvent| {
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
    };
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
                <div class="wishlist-item-actions">
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
                        ></button>
                        <div class="wishlist-feedback-content">
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
                            <form class="wishlist-feedback-form" on:submit=save_feedback>
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
                                        <Show when=move || {
                                            !feedbacks
                                                .get()
                                                .iter()
                                                .any(|feedback| feedback.member_id == member_id)
                                        }>
                                            <option value="">{move || tr!("wishlist-feedback-none")}</option>
                                        </Show>
                                        <option value="song">{move || tr!("wishlist-category-song")}</option>
                                        <option value="medley">{move || tr!("wishlist-category-medley")}</option>
                                        <option value="try">{move || tr!("wishlist-category-try")}</option>
                                        <option value="unplayable">{move || tr!("wishlist-category-unplayable")}</option>
                                    </select>
                                </label>
                                <button
                                    class="btn btn-primary btn-sm"
                                    type="submit"
                                    data-testid="wishlist-feedback-save"
                                    disabled=move || pending.get() || selected.get().is_none()
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
                        </Show>
                    </div>
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
                    }}
                </div>
            </div>
            <Show when=move || editing.get()>
                <form class="wishlist-edit-form" on:submit=update_song>
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
                        <button class="btn btn-ghost btn-sm" type="button" data-testid="wishlist-cancel-edit" on:click=cancel_edit disabled=move || edit_pending.get()>
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
}

fn category_label(category: WishlistCategory) -> String {
    match category {
        WishlistCategory::Song => tr!("wishlist-category-song"),
        WishlistCategory::Medley => tr!("wishlist-category-medley"),
        WishlistCategory::Try => tr!("wishlist-category-try"),
        WishlistCategory::Unplayable => tr!("wishlist-category-unplayable"),
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
    items.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.title.cmp(&right.title))
    });
}
