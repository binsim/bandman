use super::{
    grouping::{
        WishlistCategoryGroup, WishlistDisplayEntry, group_wishlist_items_by_category,
        sort_wishlist_items,
    },
    medley::{WishlistMedley, WishlistMedleyControls},
    metadata::SongMetadataBadges,
    modify_song::EditSong,
    wishlist_feedback_form::WishlistFeedbackControl,
};
use crate::models::{Member, SongMetadataAssignment, WishlistFeedback, WishlistItem};
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub(super) fn WishlistListing(
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
    let category_groups = Memo::new(move |_| group_wishlist_items_by_category(visible_items.get()));
    let member_store = StoredValue::new(current_member);
    view! {
        <section class="card wishlist-listing">
            <header class="wishlist-listing-header">
                <h2 id="wishlist-list-title">{move || tr!("wishlist-list-title")}
                    <span class="muted">{move || tr!("wishlist-count", {"count" => items.get().len().to_string()})}</span>
                </h2>

                <Show when=move || !selecting_medley.get()>
                    <button
                        class="btn btn-primary btn-sm"
                        type="button"
                        on:click=move |_| on_start_creating_medley.run(())
                        data-testid="wishlist-create-medley"
                    >
                        {move || tr!("wishlist-medley-start")}
                    </button>
                </Show>
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
                            data-testid="filter-feedback-needed"
                        >
                            {move || tr!("wishlist-filter-needs-feedback")}
                        </button>
                        <button
                            class="wishlist-filter-button"
                            class:is-active=move || !show_needs_feedback.get()
                            type="button"
                            aria-pressed=move || (!show_needs_feedback.get()).to_string()
                            on:click=move |_| on_filter_changed.run(false)
                            data-testid="filter-all-songs"
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
                    when=move || !visible_items.get().is_empty()
                    fallback=|| view! { <p class="wishlist-empty">{move || tr!("wishlist-no-feedback-pending")}</p> }
                >
                    <div class="wishlist-category-groups">
                        <For
                            each=move || category_groups.get()
                            key=WishlistCategoryGroup::key
                            children=move |group| {
                                let group_key = group.key();
                                let entry_count = group.entries.len();
                                let category_name = group
                                    .category
                                    .map(|category| category.name)
                                    .unwrap_or_else(|| tr!("wishlist-uncategorized"));
                                let group_entries = Memo::new(move |_| {
                                    category_groups
                                        .get()
                                        .into_iter()
                                        .find(|group| group.key() == group_key)
                                        .map(|group| group.entries)
                                        .unwrap_or_default()
                                });
                                view! {
                                    <section class="wishlist-category-group" data-testid="wishlist-category-group">
                                        <div class="wishlist-category-header">
                                            <h3>{category_name}</h3>
                                            <span class="wishlist-category-count">{entry_count.to_string()}</span>
                                        </div>
                                        <div class="wishlist-items">
                                            <WishlistItemList
                                                items
                                                entries=group_entries
                                                current_member=member_store.read_value().clone()
                                                selecting_medley
                                                selected_songs
                                                medley_pending
                                                on_disband_medley
                                                on_medley_renamed
                                                on_remove_medley_item
                                                on_move_medley_item
                                            />
                                        </div>
                                    </section>
                                }
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
                updated.medley_category = item.medley_category.clone();
                updated.medley_labels = item.medley_labels.clone();
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
                        item.medley_category = None;
                        item.medley_labels.clear();
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
pub(super) fn WishlistListItem(
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
                        data-testid="song-added-to-medley-cb"
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
pub(super) fn WishlistRow(
    item: WishlistItem,
    current_member: Member,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
    on_deleted: Callback<uuid::Uuid>,
    on_updated: Callback<WishlistItem>,
) -> AnyView {
    let item = RwSignal::new(item);

    let medley_management_active = expect_context::<RwSignal<bool>>();
    let metadata = Signal::derive(move || SongMetadataAssignment {
        category: item.get().song_category.clone(),
        labels: item.get().labels.clone(),
    });
    let member_id = current_member.id;

    view! {
        <article
            class="wishlist-item"
            class:needs-feedback=move || {
                !item.get().feedback.iter().any(|feedback| feedback.member_id == member_id)
            }
            data-testid="wishlist-item"
            data-song-id=move || item.get().id.to_string()
            data-song-name=move || item.get().title
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
                    <SongMetadataBadges assignment=metadata />
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
