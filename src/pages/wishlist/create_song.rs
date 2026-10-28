use super::WISHLIST_MUSICAL_KEYS;
use crate::models::{Member, MemberRole, WishlistCategory, WishlistItem, WishlistSongDetails};
use leptos::prelude::*;
use leptos_fluent::tr;

#[server(WishlistCreate, "/api")]
async fn wishlist_create(
    title: String,
    artist: String,
    link: String,
    tempo: String,
    musical_key: String,
) -> Result<WishlistItem, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    let link_title = resolve_link_title(&link).await;
    let tempo = parse_wishlist_tempo(&tempo)?;
    WishlistItem::create_with_music_details(
        &pool,
        WishlistSongDetails {
            title: &title,
            artist: Some(&artist),
            link: Some(&link),
            link_title: link_title.as_deref(),
            tempo,
            musical_key: Some(&musical_key),
        },
        &[],
        WishlistCategory::Song,
        member.id,
        &member.name,
    )
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
    tempo: String,
    musical_key: String,
) -> Result<WishlistItem, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    let member = crate::auth::session::require_member().await?;
    let link_title = resolve_link_title(&link).await;
    let tempo = parse_wishlist_tempo(&tempo)?;
    WishlistItem::update_details_with_music(
        &pool,
        item_id,
        member.id,
        member.role == crate::models::MemberRole::Admin,
        WishlistSongDetails {
            title: &title,
            artist: Some(&artist),
            link: Some(&link),
            link_title: link_title.as_deref(),
            tempo,
            musical_key: Some(&musical_key),
        },
    )
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
}
#[component]
pub fn WishlistSongFields(
    title: RwSignal<String>,
    artist: RwSignal<String>,
    link: RwSignal<String>,
    tempo: RwSignal<String>,
    musical_key: RwSignal<String>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] test_id_prefix: Option<&'static str>,
    #[prop(optional)] with_placeholders: bool,
) -> impl IntoView {
    let is_disabled = disabled.unwrap_or_else(|| Signal::derive(|| false));
    let title_test_id = match test_id_prefix {
        Some(prefix) => format!("{prefix}-title"),
        None => "add-song-name".to_string(),
    };
    let artist_test_id = match test_id_prefix {
        Some(prefix) => format!("{prefix}-artist"),
        None => "wishlist-artist".to_string(),
    };
    let link_test_id = match test_id_prefix {
        Some(prefix) => format!("{prefix}-link"),
        None => "wishlist-link".to_string(),
    };
    let tempo_test_id = match test_id_prefix {
        Some(prefix) => format!("{prefix}-tempo"),
        None => "wishlist-tempo".to_string(),
    };
    let key_test_id = match test_id_prefix {
        Some(prefix) => format!("{prefix}-key"),
        None => "wishlist-key".to_string(),
    };
    let key_options = WISHLIST_MUSICAL_KEYS
        .iter()
        .map(|key| view! { <option value=*key>{*key}</option> })
        .collect_view();

    view! {
        <label class="field">
            <span class="field-label">{move || tr!("wishlist-song-label")}</span>
            <input
                class="input"
                data-testid=title_test_id
                type="text"
                maxlength="120"
                placeholder=move || {
                    if with_placeholders {
                        tr!("wishlist-song-placeholder")
                    } else {
                        String::new()
                    }
                }
                required
                prop:value=move || title.get()
                on:input=move |event| title.set(event_target_value(&event))
                disabled=move || is_disabled.get()
            />
        </label>
        <label class="field">
            <span class="field-label">{move || tr!("wishlist-artist-label")}</span>
            <input
                class="input"
                data-testid=artist_test_id
                type="text"
                maxlength="120"
                placeholder=move || {
                    if with_placeholders {
                        tr!("wishlist-artist-placeholder")
                    } else {
                        String::new()
                    }
                }
                prop:value=move || artist.get()
                on:input=move |event| artist.set(event_target_value(&event))
                disabled=move || is_disabled.get()
            />
        </label>
        <label class="field">
            <span class="field-label">{move || tr!("wishlist-link-label")}</span>
            <input
                class="input"
                data-testid=link_test_id
                type="url"
                maxlength="2048"
                placeholder=move || {
                    if with_placeholders {
                        "https://".to_string()
                    } else {
                        String::new()
                    }
                }
                prop:value=move || link.get()
                on:input=move |event| link.set(event_target_value(&event))
                disabled=move || is_disabled.get()
            />
        </label>
        <label class="field">
            <span class="field-label">{move || tr!("wishlist-tempo-label")}</span>
            <input
                class="input"
                data-testid=tempo_test_id
                type="number"
                min="20"
                max="300"
                step="1"
                placeholder="120"
                prop:value=move || tempo.get()
                on:input=move |event| tempo.set(event_target_value(&event))
                disabled=move || is_disabled.get()
            />
        </label>
        <label class="field">
            <span class="field-label">{move || tr!("wishlist-key-label")}</span>
            <select
                class="input"
                data-testid=key_test_id
                prop:value=move || musical_key.get()
                on:change=move |event| musical_key.set(event_target_value(&event))
                disabled=move || is_disabled.get()
            >
                <option value="">{move || tr!("wishlist-key-unknown")}</option>
                {key_options}
            </select>
        </label>
    }
}

#[component]
pub fn EditSong(
    item: RwSignal<WishlistItem>,
    current_member: Member,
    on_updated: Callback<WishlistItem>,
    on_deleted: Callback<uuid::Uuid>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let edit_pending = RwSignal::new(false);

    if !editing.get() {
        return view! {
            <button
                class="btn btn-ghost btn-sm wishlist-edit-button"
                type="button"
                data-testid="wishlist-edit"
                on:click=move |_| {
                    if editing.get() {
                        editing.set(false);
                    } else {
                        editing.set(true);
                    }
                }
                disabled=move || edit_pending.get()
            >
                {move || if editing.get() { tr!("wishlist-close-edit") } else { tr!("wishlist-edit") }}
            </button>
        }.into_any();
    }

    let confirm_delete = RwSignal::new(false);
    if confirm_delete.get() {
        return view! { <DeleteConfirmationForm
        item_id=item.get().id
        item_name = item.get().title
        confirm_delete
        on_deleted
        /> }
        .into_any();
    }

    let can_delete = current_member.role == MemberRole::Admin
        || item.get().proposed_by_id == Some(current_member.id);

    let item_id = item.get().id;
    let title = RwSignal::new(item.get().title.clone());
    let artist = RwSignal::new(item.get().artist.clone().unwrap_or_default());
    let link = RwSignal::new(item.get().link.clone().unwrap_or_default());
    let tempo = RwSignal::new(item.get().tempo.map(|i| i.to_string()).unwrap_or_default());
    let musical_key = RwSignal::new(item.get().musical_key.clone().unwrap_or_default());

    let error = RwSignal::new(Option::<String>::None);

    let update_song = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        edit_pending.set(true);
        error.set(None);

        leptos::task::spawn_local(async move {
            match wishlist_update(
                item_id,
                title.get(),
                artist.get(),
                link.get(),
                tempo.get(),
                musical_key.get(),
            )
            .await
            {
                Ok(updated) => {
                    title.set(updated.title.clone());
                    artist.set(updated.artist.clone().unwrap_or_default());
                    link.set(updated.link.clone().unwrap_or_default());
                    tempo.set(updated.tempo.map(|i| i.to_string()).unwrap_or_default());
                    musical_key.set(updated.musical_key.clone().unwrap_or_default());
                    on_updated.run(updated);
                    edit_pending.set(false);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            edit_pending.set(false);
        });
    };
    let cancel_edit = Callback::new(move |_: ()| {
        title.set(item.get().title.clone());
        artist.set(item.get().artist.clone().unwrap_or_default());
        link.set(item.get().link.clone().unwrap_or_default());
        tempo.set(item.get().tempo.map(|i| i.to_string()).unwrap_or_default());
        musical_key.set(item.get().musical_key.clone().unwrap_or_default());
        error.set(None);
        editing.set(false);
    });

    view! {
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
                            on:click=move |_| confirm_delete.set(true)
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
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert" data-testid="wishlist-edit-error">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </form>
    }.into_any()
}

#[component]
pub fn DeleteConfirmationForm(
    item_id: uuid::Uuid,
    item_name: String,
    confirm_delete: RwSignal<bool>,
    on_deleted: Callback<uuid::Uuid>,
) -> impl IntoView {
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let delete_item = move |_| {
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_delete(item_id).await {
                Ok(()) => on_deleted.run(item_id),
                Err(message) => error.set(Some(message.to_string())),
            }
            confirm_delete.set(false);
            pending.set(false);
        });
    };

    view! {
        <div class="admin-delete-backdrop">
            <section
                class="admin-delete-dialog"
                role="dialog"
                aria-modal="true"
                aria-label=move || tr!("wishlist-delete-title")
                on:keydown=move |event: leptos::ev::KeyboardEvent| {
                    if event.key() == "Escape" && !pending.get() {
                        confirm_delete.set(false);
                    }
                }
            >
                <div class="admin-delete-dialog-content">
                    <h2>{move || tr!("wishlist-delete-title")}</h2>
                    <p>{move || tr!("wishlist-confirm-delete", {"name" => item_name.clone()})}</p>
                    <Show when=move || error.get().is_some()>
                        <p class="form-error" role="alert">{move || error.get().unwrap_or_default()}</p>
                    </Show>
                    <div class="admin-delete-dialog-actions">
                        <button
                            class="btn btn-ghost btn-danger btn-sm"
                            type="button"
                            data-testid="wishlist-confirm-delete"
                            on:click=delete_item
                            disabled=move || pending.get()
                        >
                            {move || if pending.get() {
                                tr!("wishlist-deleting")
                            } else {
                                tr!("wishlist-delete")
                            }}
                        </button>
                        <button
                            class="btn btn-ghost btn-sm"
                            type="button"
                            data-testid="wishlist-cancel-delete"
                            on:click=move |_| confirm_delete.set(false)
                            disabled=move || pending.get()
                        >
                            {move || tr!("admin-cancel")}
                        </button>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
pub fn CreateSong(
    items: RwSignal<Vec<WishlistItem>>,
    on_created: Callback<WishlistItem>,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let artist = RwSignal::new(String::new());
    let link = RwSignal::new(String::new());
    let tempo = RwSignal::new(String::new());
    let musical_key = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let matching_title = Memo::new(move |_| has_matching_title(title, artist, items));

    let on_create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if matching_title.get() == Some(true) {
            return;
        }
        let title_value = title.get().trim().to_string();
        let artist_value = artist.get().trim().to_string();
        let link_value = link.get().trim().to_string();
        let tempo_value = tempo.get();
        let musical_key_value = musical_key.get();

        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match wishlist_create(
                title_value,
                artist_value,
                link_value,
                tempo_value,
                musical_key_value,
            )
            .await
            {
                Ok(item) => {
                    title.set(String::new());
                    artist.set(String::new());
                    link.set(String::new());
                    tempo.set(String::new());
                    musical_key.set(String::new());
                    on_created.run(item);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };

    view! {
        <section class="wishlist-form-card">
            <h2>{move || tr!("wishlist-add-title")}</h2>
            <form class="wishlist-form" on:submit=on_create>
                <div class="wishlist-form-fields">
                    <WishlistSongFields
                        title
                        artist
                        link
                        tempo
                        musical_key
                        with_placeholders=true
                    />
                </div>

                <Show when=move || matching_title.get().is_some()>
                    <p class="wishlist-duplicate-warning" role="status" data-testid="wishlist-duplicate-warning">
                        {move || if matching_title.get() == Some(true) {
                            tr!("wishlist-duplicate-exact")
                        } else {
                            tr!("wishlist-duplicate-title")
                        }}
                    </p>
                </Show>

                <div class="wishlist-form-actions">
                    <button
                        class="btn btn-primary"
                        type="submit"
                        data-testid="create-song"
                        disabled=move || pending.get() || matching_title.get() == Some(true)
                    >
                        {move || if pending.get() {
                            tr!("wishlist-saving")
                        } else {
                            tr!("wishlist-add-button")
                        }}
                    </button>
                </div>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert" data-testid="wishlist-create-error">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </section>
    }
    .into_any()
}

fn has_matching_title(
    title: RwSignal<String>,
    artist: RwSignal<String>,
    items: RwSignal<Vec<WishlistItem>>,
) -> Option<bool> {
    let title = title.get().trim().to_lowercase();
    if title.is_empty() {
        return None;
    }
    let artist = artist.get().trim().to_lowercase();
    let items = items.get();
    if items.iter().any(|item| {
        item.title.trim().to_lowercase() == title
            && item
                .artist
                .as_deref()
                .unwrap_or_default()
                .trim()
                .to_lowercase()
                == artist
    }) {
        Some(true)
    } else if items
        .iter()
        .any(|item| item.title.trim().to_lowercase() == title)
    {
        Some(false)
    } else {
        None
    }
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
            use crate::pages::wishlist::YoutubeOembed;

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

#[cfg(feature = "ssr")]
fn parse_wishlist_tempo(value: &str) -> Result<Option<i32>, ServerFnError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<i32>()
        .map(Some)
        .map_err(|_| ServerFnError::new("Tempo must be a whole number of BPM"))
}
