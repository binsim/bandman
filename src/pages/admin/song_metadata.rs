use crate::models::{SONG_LABEL_COLORS, SongCategory, SongLabel, SongMetadataOptions};
use leptos::prelude::*;
use leptos_fluent::tr;
use uuid::Uuid;

#[server(AdminSongMetadataOptions, "/api")]
async fn admin_song_metadata_options() -> Result<SongMetadataOptions, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    let categories = SongCategory::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let labels = SongLabel::list(&pool)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(SongMetadataOptions { categories, labels })
}

#[server(AdminCreateSongCategory, "/api")]
async fn admin_create_song_category(name: String) -> Result<SongCategory, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongCategory::create(&pool, &name)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(AdminUpdateSongCategory, "/api")]
async fn admin_update_song_category(id: Uuid, name: String) -> Result<SongCategory, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongCategory::update(&pool, id, &name)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(AdminDeleteSongCategory, "/api")]
async fn admin_delete_song_category(id: Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongCategory::delete(&pool, id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(AdminCreateSongLabel, "/api")]
async fn admin_create_song_label(name: String, color: String) -> Result<SongLabel, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongLabel::create(&pool, &name, &color)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(AdminUpdateSongLabel, "/api")]
async fn admin_update_song_label(
    id: Uuid,
    name: String,
    color: String,
) -> Result<SongLabel, ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongLabel::update(&pool, id, &name, &color)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server(AdminDeleteSongLabel, "/api")]
async fn admin_delete_song_label(id: Uuid) -> Result<(), ServerFnError> {
    let pool = expect_context::<crate::state::AppState>().pool;
    crate::auth::session::require_admin().await?;
    SongLabel::delete(&pool, id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[component]
pub(super) fn SongMetadataAdmin() -> impl IntoView {
    let options = Resource::new(
        || (),
        |_| async move { admin_song_metadata_options().await },
    );

    view! {
        <Suspense fallback=|| view! { <p class="muted">{move || tr!("admin-song-options-loading")}</p> }>
            {move || match options.get() {
                Some(Ok(options)) => view! { <SongMetadataLoaded options /> }.into_any(),
                Some(Err(error)) => view! {
                    <p class="form-error" role="alert">{error.to_string()}</p>
                }.into_any(),
                None => ().into_any(),
            }}
        </Suspense>
    }
}

#[component]
fn SongMetadataLoaded(options: SongMetadataOptions) -> impl IntoView {
    let categories = RwSignal::new(options.categories);
    let labels = RwSignal::new(options.labels);
    let category_error = RwSignal::new(Option::<String>::None);
    let label_error = RwSignal::new(Option::<String>::None);
    let on_category_created = Callback::new(move |category: SongCategory| {
        categories.update(|categories| {
            categories.push(category);
            categories.sort_by_cached_key(|category| category.name.to_lowercase());
        });
    });
    let on_category_updated = Callback::new(move |category: SongCategory| {
        categories.update(|categories| {
            if let Some(existing) = categories
                .iter_mut()
                .find(|existing| existing.id == category.id)
            {
                *existing = category;
            }
        });
    });
    let on_category_deleted = Callback::new(move |id| {
        categories.update(|categories| categories.retain(|category| category.id != id));
    });
    let on_label_created = Callback::new(move |label: SongLabel| {
        labels.update(|labels| {
            labels.push(label);
            labels.sort_by_cached_key(|label| label.name.to_lowercase());
        });
    });
    let on_label_updated = Callback::new(move |label: SongLabel| {
        labels.update(|labels| {
            if let Some(existing) = labels.iter_mut().find(|existing| existing.id == label.id) {
                *existing = label;
            }
        });
    });
    let on_label_deleted = Callback::new(move |id| {
        labels.update(|labels| labels.retain(|label| label.id != id));
    });

    view! {
        <section class="card admin-disclosure song-metadata-admin">
            <details data-testid="admin-song-categories-section">
                <summary>
                    <span>{move || tr!("admin-song-categories-title")}</span>
                    <span class="admin-disclosure-count">{move || categories.get().len()}</span>
                </summary>
                <SongCategoryForm on_created=on_category_created error=category_error />
                <Show when=move || category_error.get().is_some()>
                    <p class="form-error" role="alert">{move || category_error.get().unwrap_or_default()}</p>
                </Show>
                <div class="song-metadata-options-list">
                    <For
                        each=move || categories.get()
                        key=|category| category.id
                        children=move |category| view! {
                            <SongCategoryRow
                                category
                                on_updated=on_category_updated
                                on_deleted=on_category_deleted
                                error=category_error
                            />
                        }
                    />
                </div>
            </details>
        </section>
        <section class="card admin-disclosure song-metadata-admin">
            <details data-testid="admin-song-labels-section">
                <summary>
                    <span>{move || tr!("admin-song-labels-title")}</span>
                    <span class="admin-disclosure-count">{move || labels.get().len()}</span>
                </summary>
                <SongLabelForm on_created=on_label_created error=label_error />
                <Show when=move || label_error.get().is_some()>
                    <p class="form-error" role="alert">{move || label_error.get().unwrap_or_default()}</p>
                </Show>
                <div class="song-metadata-options-list">
                    <For
                        each=move || labels.get()
                        key=|label| label.id
                        children=move |label| view! {
                            <SongLabelRow
                                label
                                on_updated=on_label_updated
                                on_deleted=on_label_deleted
                                error=label_error
                            />
                        }
                    />
                </div>
            </details>
        </section>
    }
}

#[component]
fn SongCategoryForm(
    on_created: Callback<SongCategory>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_create_song_category(name.get()).await {
                Ok(category) => {
                    name.set(String::new());
                    on_created.run(category);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };

    view! {
        <form class="song-metadata-option-form" on:submit=create>
            <input
                class="input"
                maxlength="60"
                required
                aria-label=move || tr!("admin-song-category-name")
                placeholder=move || tr!("admin-song-category-name")
                prop:value=move || name.get()
                on:input=move |event| name.set(event_target_value(&event))
                disabled=move || pending.get()
                data-testid="admin-song-category-name"
            />
            <button class="btn btn-primary btn-sm" type="submit" data-testid="admin-song-category-add" disabled=move || pending.get() || name.get().trim().is_empty()>
                {move || tr!("admin-song-category-add")}
            </button>
        </form>
    }
}

#[component]
fn SongCategoryRow(
    category: SongCategory,
    on_updated: Callback<SongCategory>,
    on_deleted: Callback<Uuid>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = category.id;
    let name = RwSignal::new(category.name.clone());
    let original_name = StoredValue::new(category.name);
    let pending = RwSignal::new(false);
    let confirming_delete = RwSignal::new(false);
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_update_song_category(id, name.get()).await {
                Ok(updated) => {
                    name.set(updated.name.clone());
                    original_name.set_value(updated.name.clone());
                    on_updated.run(updated);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };
    let delete = move |_| {
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_delete_song_category(id).await {
                Ok(()) => on_deleted.run(id),
                Err(message) => error.set(Some(message.to_string())),
            }
            confirming_delete.set(false);
            pending.set(false);
        });
    };

    view! {
        <div class="song-metadata-option" data-testid="admin-song-category-row" data-category-id=id.to_string() data-category-name=move || name.get()>
            <form class="song-metadata-option-form" on:submit=save>
                <input
                    class="input"
                    maxlength="60"
                    required
                    aria-label=move || tr!("admin-song-category-name")
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                    disabled=move || pending.get()
                    data-testid="admin-song-category-row-name"
                />
                <button class="btn btn-ghost btn-sm" type="submit" data-testid="admin-song-category-save" disabled=move || pending.get() || name.get().trim().is_empty() || name.get().trim() == original_name.get_value()>
                    {move || tr!("admin-save-changes")}
                </button>
                <button class="btn btn-ghost btn-danger btn-sm" type="button" data-testid="admin-song-category-delete" on:click=move |_| confirming_delete.set(true) disabled=move || pending.get()>
                    {move || tr!("admin-delete")}
                </button>
            </form>
            <Show when=move || confirming_delete.get()>
                <div
                    class="song-metadata-confirm"
                    role="alertdialog"
                    aria-label=move || tr!("admin-song-category-delete-confirm", {"name" => name.get()})
                >
                    <p>{move || tr!("admin-song-category-delete-confirm", {"name" => name.get()})}</p>
                    <button class="btn btn-ghost btn-danger btn-sm" type="button" data-testid="admin-song-category-confirm-delete" on:click=delete disabled=move || pending.get()>
                        {move || tr!("admin-delete")}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" data-testid="admin-song-category-cancel-delete" on:click=move |_| confirming_delete.set(false) disabled=move || pending.get()>
                        {move || tr!("admin-cancel")}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SongLabelForm(
    on_created: Callback<SongLabel>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let color = RwSignal::new(SONG_LABEL_COLORS[0].to_string());
    let pending = RwSignal::new(false);
    let create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_create_song_label(name.get(), color.get()).await {
                Ok(label) => {
                    name.set(String::new());
                    on_created.run(label);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };

    view! {
        <form class="song-metadata-option-form" on:submit=create>
            <span
                class="wishlist-badge wishlist-label-badge admin-song-label-preview"
                style=move || format!("--label-color: {}", color.get())
                data-testid="admin-song-label-preview"
            >
                {move || {
                    if name.get().trim().is_empty() {
                        tr!("admin-label-preview")
                    } else {
                        name.get()
                    }
                }}
            </span>
            <input
                class="input"
                maxlength="40"
                required
                aria-label=move || tr!("admin-song-label-name")
                placeholder=move || tr!("admin-song-label-name")
                prop:value=move || name.get()
                on:input=move |event| name.set(event_target_value(&event))
                disabled=move || pending.get()
                data-testid="admin-song-label-name"
            />
            <SongLabelColorSelect color disabled=pending/>
            <button class="btn btn-primary btn-sm" type="submit" data-testid="admin-song-label-add" disabled=move || pending.get() || name.get().trim().is_empty()>
                {move || tr!("admin-song-label-add")}
            </button>
        </form>
    }
}

#[component]
fn SongLabelRow(
    label: SongLabel,
    on_updated: Callback<SongLabel>,
    on_deleted: Callback<Uuid>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = label.id;
    let name = RwSignal::new(label.name.clone());
    let original_name = StoredValue::new(label.name);
    let color = RwSignal::new(label.color.clone());
    let original_color = StoredValue::new(label.color);
    let pending = RwSignal::new(false);
    let confirming_delete = RwSignal::new(false);
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_update_song_label(id, name.get(), color.get()).await {
                Ok(updated) => {
                    name.set(updated.name.clone());
                    original_name.set_value(updated.name.clone());
                    color.set(updated.color.clone());
                    original_color.set_value(updated.color.clone());
                    on_updated.run(updated);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };
    let delete = move |_| {
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_delete_song_label(id).await {
                Ok(()) => on_deleted.run(id),
                Err(message) => error.set(Some(message.to_string())),
            }
            confirming_delete.set(false);
            pending.set(false);
        });
    };

    view! {
        <div class="song-metadata-option" data-testid="admin-song-label-row" data-label-id=id.to_string() data-label-name=move || name.get()>
            <form class="song-metadata-option-form" on:submit=save>
                <span
                    class="wishlist-badge wishlist-label-badge admin-song-label-preview"
                    style=move || format!("--label-color: {}", color.get())
                    data-testid="admin-song-label-preview"
                >
                    {move || name.get()}
                </span>
                <input
                    class="input"
                    maxlength="40"
                    required
                    aria-label=move || tr!("admin-song-label-name")
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                    disabled=move || pending.get()
                    data-testid="admin-song-label-row-name"
                />
                <SongLabelColorSelect color disabled=pending/>
                <button class="btn btn-ghost btn-sm" type="submit" data-testid="admin-song-label-save" disabled=move || pending.get() || name.get().trim().is_empty() || (name.get().trim() == original_name.get_value() && color.get() == original_color.get_value())>
                    {move || tr!("admin-save-changes")}
                </button>
                <button class="btn btn-ghost btn-danger btn-sm" type="button" data-testid="admin-song-label-delete" on:click=move |_| confirming_delete.set(true) disabled=move || pending.get()>
                    {move || tr!("admin-delete")}
                </button>
            </form>
            <Show when=move || confirming_delete.get()>
                <div
                    class="song-metadata-confirm"
                    role="alertdialog"
                    aria-label=move || tr!("admin-song-label-delete-confirm", {"name" => name.get()})
                >
                    <p>{move || tr!("admin-song-label-delete-confirm", {"name" => name.get()})}</p>
                    <button class="btn btn-ghost btn-danger btn-sm" type="button" data-testid="admin-song-label-confirm-delete" on:click=delete disabled=move || pending.get()>
                        {move || tr!("admin-delete")}
                    </button>
                    <button class="btn btn-ghost btn-sm" type="button" data-testid="admin-song-label-cancel-delete" on:click=move |_| confirming_delete.set(false) disabled=move || pending.get()>
                        {move || tr!("admin-cancel")}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SongLabelColorSelect(color: RwSignal<String>, disabled: RwSignal<bool>) -> impl IntoView {
    view! {
        <select
            class="select"
            aria-label=move || tr!("admin-song-label-color")
            prop:value=move || color.get()
            on:change=move |event| color.set(event_target_value(&event))
            disabled=move || disabled.get()
        >
            <option value="#2563EB">{move || tr!("admin-label-color-blue")}</option>
            <option value="#15803D">{move || tr!("admin-label-color-green")}</option>
            <option value="#C2410C">{move || tr!("admin-label-color-orange")}</option>
            <option value="#BE123C">{move || tr!("admin-label-color-red")}</option>
            <option value="#7E22CE">{move || tr!("admin-label-color-purple")}</option>
            <option value="#475569">{move || tr!("admin-label-color-slate")}</option>
        </select>
    }
}
