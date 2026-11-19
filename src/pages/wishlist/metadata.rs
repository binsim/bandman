use crate::models::{SongMetadataAssignment, SongMetadataOptions};
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub(super) fn SongMetadataBadges(assignment: Signal<SongMetadataAssignment>) -> impl IntoView {
    view! {
        <Show when=move || assignment.get().category.is_some()>
            <span class="wishlist-badge wishlist-category-badge">
                {move || assignment.get().category.map(|category| category.name).unwrap_or_default()}
            </span>
        </Show>
        <For
            each=move || assignment.get().labels
            key=|label| label.id
            children=move |label| view! {
                <span
                    class="wishlist-badge wishlist-label-badge"
                    style=move || format!("--label-color: {}", label.color)
                >
                    {label.name}
                </span>
            }
        />
    }
}

#[component]
pub(super) fn SongMetadataFields(
    category_id: RwSignal<Option<uuid::Uuid>>,
    label_ids: RwSignal<Vec<uuid::Uuid>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let options = expect_context::<RwSignal<SongMetadataOptions>>();

    view! {
        <div class="wishlist-metadata-fields">
            <label class="field">
                <span class="field-label">{move || tr!("wishlist-song-category")}</span>
                <select
                    class="select"
                    data-testid="wishlist-song-category-select"
                    prop:value=move || category_id.get().map(|id| id.to_string()).unwrap_or_default()
                    on:change=move |event| {
                        let value = event_target_value(&event);
                        category_id.set(uuid::Uuid::parse_str(&value).ok());
                    }
                    disabled=move || disabled.get()
                >
                    <option value="">{move || tr!("wishlist-uncategorized")}</option>
                    <For
                        each=move || options.get().categories
                        key=|category| category.id
                        children=move |category| view! {
                            <option value=category.id.to_string()>{category.name}</option>
                        }
                    />
                </select>
            </label>
            <Show when=move || !options.get().labels.is_empty()>
                <fieldset class="wishlist-label-options">
                    <legend>{move || tr!("wishlist-labels")}</legend>
                    <For
                        each=move || options.get().labels
                        key=|label| label.id
                        children=move |label| {
                            let label_id = label.id;
                            view! {
                                <label class="wishlist-label-option">
                                    <input
                                        type="checkbox"
                                        data-testid=format!("wishlist-label-{}", label.id)
                                        prop:checked=move || label_ids.get().contains(&label_id)
                                        on:change=move |_| label_ids.update(|ids| {
                                            if ids.contains(&label_id) {
                                                ids.retain(|id| *id != label_id);
                                            } else {
                                                ids.push(label_id);
                                            }
                                        })
                                        disabled=move || disabled.get()
                                    />
                                    <span
                                        class="wishlist-badge wishlist-label-badge"
                                        style=move || format!("--label-color: {}", label.color)
                                    >
                                        {label.name}
                                    </span>
                                </label>
                            }
                        }
                    />
                </fieldset>
            </Show>
        </div>
    }
}
