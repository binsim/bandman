use crate::models::WishlistItem;
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn CreateSong(
    items: RwSignal<Vec<WishlistItem>>,
    on_created: Callback<WishlistItem>,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let artist = RwSignal::new(String::new());
    let link = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let matching_title = Memo::new(move |_| {
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
    });

    let on_create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if matching_title.get() == Some(true) {
            return;
        }
        let title_value = title.get().trim().to_string();
        let artist_value = artist.get().trim().to_string();
        let link_value = link.get().trim().to_string();

        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match super::wishlist_create(title_value, artist_value, link_value).await {
                Ok(item) => {
                    title.set(String::new());
                    artist.set(String::new());
                    link.set(String::new());
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
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-song-label")}</span>
                        <input
                            class="input"
                            data-testid="add-song-name"
                            type="text"
                            maxlength="120"
                            placeholder=move || tr!("wishlist-song-placeholder")
                            required
                            prop:value=move || title.get()
                            on:input=move |event| title.set(event_target_value(&event))
                        />
                    </label>
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-artist-label")}</span>
                        <input
                            class="input"
                            data-testid="wishlist-artist"
                            type="text"
                            maxlength="120"
                            placeholder=move || tr!("wishlist-artist-placeholder")
                            prop:value=move || artist.get()
                            on:input=move |event| artist.set(event_target_value(&event))
                        />
                    </label>
                    <label class="field">
                        <span class="field-label">{move || tr!("wishlist-link-label")}</span>
                        <input
                            class="input"
                            data-testid="wishlist-link"
                            type="url"
                            maxlength="2048"
                            placeholder="https://"
                            prop:value=move || link.get()
                            on:input=move |event| link.set(event_target_value(&event))
                        />
                    </label>
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
