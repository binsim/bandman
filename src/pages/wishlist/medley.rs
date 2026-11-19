use super::{
    api::wishlist_update_medley,
    grouping::sort_wishlist_items,
    listing::{WishlistListItem, WishlistRow},
    metadata::{SongMetadataBadges, SongMetadataFields},
};
use crate::models::{Member, MemberRole, SongMetadataAssignment, WishlistFeedback, WishlistItem};
use leptos::{ev, prelude::*};
use leptos_fluent::tr;

#[component]
pub(super) fn WishlistMedleyControls(
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
            <div class="wishlist-medley-controls" data-testid="wishlist-medley-create-form">
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
                        data-testid="medley-name-input"
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
                        data-testid="save-medley"
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
                        data-testid="cancel-medley"
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
pub(super) fn WishlistMedley(
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
    let can_delete =
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
    let medley_metadata = Signal::derive(move || {
        songs
            .get()
            .first()
            .map(|item| SongMetadataAssignment {
                category: item.medley_category.clone(),
                labels: item.medley_labels.clone(),
            })
            .unwrap_or(SongMetadataAssignment {
                category: None,
                labels: Vec::new(),
            })
    });
    let song_count = Memo::new(move |_| songs.get().len());
    let expanded = RwSignal::new(true);
    let editing_name = RwSignal::new(false);
    let edit_name = RwSignal::new(medley_name.get_untracked());
    let category_id = RwSignal::new(
        medley_metadata
            .get_untracked()
            .category
            .as_ref()
            .map(|category| category.id),
    );
    let label_ids = RwSignal::new(
        medley_metadata
            .get_untracked()
            .labels
            .iter()
            .map(|label| label.id)
            .collect::<Vec<_>>(),
    );
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
    let reset_metadata = move || {
        let metadata = medley_metadata.get();
        category_id.set(metadata.category.as_ref().map(|category| category.id));
        label_ids.set(metadata.labels.iter().map(|label| label.id).collect());
    };
    let start_editing = Callback::new(move |_: ()| {
        edit_name.set(medley_name.get());
        reset_metadata();
        edit_song_ids.set(songs.get().iter().map(|item| item.id).collect());
        rename_error.set(None);
        editing_medley_id.set(Some(id));
        editing_name.set(true);
        medley_management_active.set(true);
        expanded.set(true);
    });
    let cancel_editing = Callback::new(move |_: ()| {
        edit_name.set(medley_name.get());
        reset_metadata();
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
        let category_id = category_id.get();
        let label_ids = label_ids.get();
        leptos::task::spawn_local(async move {
            match wishlist_update_medley(
                id,
                ordered_ids.clone(),
                new_name.clone(),
                category_id,
                Some(label_ids),
            )
            .await
            {
                Ok(metadata) => {
                    let song_count = ordered_ids.len() as i64;
                    items.update(|items| {
                        for item in items.iter_mut().filter(|item| item.medley_id == Some(id)) {
                            if !ordered_ids.contains(&item.id) {
                                item.medley_id = None;
                                item.medley_position = None;
                                item.medley_size = None;
                                item.medley_created_by_id = None;
                                item.medley_name = None;
                                item.medley_category = None;
                                item.medley_labels.clear();
                            }
                        }
                        for (position, song_id) in ordered_ids.iter().enumerate() {
                            if let Some(item) = items.iter_mut().find(|item| item.id == *song_id) {
                                item.medley_id = Some(id);
                                item.medley_position = Some(position as i32);
                                item.medley_size = Some(song_count);
                                item.medley_created_by_id = creator_id;
                                item.medley_name = Some(new_name.clone());
                                item.medley_category = metadata.category.clone();
                                item.medley_labels = metadata.labels.clone();
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
            data-medley-name=move || medley_name
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
                        data-testid="medley-expand-button"
                    >
                        <span aria-hidden="true">{move || if expanded.get() { "▾" } else { "▸" }}</span>
                    </button>
                    <h3>{move || medley_name.get()}
                        <span class="muted">
                            {move || tr!("wishlist-medley-song-count", {"count" => song_count.get().to_string()})}
                        </span>
                    </h3>
                    <SongMetadataBadges assignment=medley_metadata />
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
                        <Show when=move || !medley_management_active.get() && !selecting_medley.get()>
                            <WishlistMedleyActions
                                can_delete
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
                            data-testid="medley-name-edit"
                        />
                    </header>
                    <details class="wishlist-metadata-editor" data-testid="wishlist-metadata-details">
                        <summary>{move || tr!("wishlist-metadata-edit")}</summary>
                        <SongMetadataFields
                            category_id
                            label_ids
                            disabled=Signal::derive(move || rename_pending.get())
                        />
                    </details>
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
                                                data-testid="song-added-to-medley-cb"
                                            />
                                        </label>
                                        <div class="wishlist-medley-order-controls">
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                type="button"
                                                aria-label=move || tr!("wishlist-medley-move-up")
                                                on:click=move |_| edit_song_ids.update(|ids| move_song(ids, song_id, -1))
                                                disabled=move || rename_pending.get() || edit_song_ids.get().first() == Some(&song_id)
                                                data-testid="medley-move-song-up-button"
                                            >"↑"</button>
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                type="button"
                                                aria-label=move || tr!("wishlist-medley-move-down")
                                                on:click=move |_| edit_song_ids.update(|ids| move_song(ids, song_id, 1))
                                                disabled=move || rename_pending.get() || edit_song_ids.get().last() == Some(&song_id)
                                                data-testid="medley-move-song-down-button"
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
                            data-testid="medley-save-button"
                        >
                            {move || if rename_pending.get() { tr!("wishlist-medley-saving-changes") } else { tr!("admin-save-changes") }}
                        </button>
                        <button
                            class="btn btn-ghost btn-sm"
                            type="button"
                            on:click=move |_| cancel_editing.run(())
                            disabled=move || rename_pending.get()
                            data-testid="medley-cancel-button"
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
    can_delete: bool,
    pending: RwSignal<bool>,
    confirming: RwSignal<bool>,
    on_edit: Callback<()>,
    on_confirm_disband: Callback<()>,
) -> AnyView {
    let menu_open = RwSignal::new(false);
    let pointer_listener = window_event_listener(ev::pointerdown, move |_| menu_open.set(false));
    on_cleanup(move || pointer_listener.remove());
    view! {
        <div
            class="wishlist-medley-menu"
            on:pointerdown=|ev| ev.stop_propagation()
            on:click=|ev| ev.stop_propagation()
        >
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                aria-expanded=move || menu_open.get().to_string()
                on:click=move |_| menu_open.update(|open| *open = !*open)
                data-testid="medley-manage-button"
            >
                {move || tr!("wishlist-medley-actions")}
                <span class="wishlist-medley-menu-arrow" aria-hidden="true">"▾"</span>
            </button>
            <Show when=move || menu_open.get()>
                <div class="wishlist-medley-menu-items">
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        on:click=move |_| {
                            menu_open.set(false);
                            on_edit.run(());
                        }
                        data-testid="medley-edit-button"
                    >
                        {move || tr!("wishlist-medley-edit")}
                    </button>
                    <Show when=move || can_delete>
                        <button
                            class="btn btn-ghost btn-danger btn-sm"
                            type="button"
                            on:click=move |_| {
                                menu_open.set(false);
                                confirming.set(true);
                            }
                            disabled=move || pending.get()
                            data-testid="medley-delete-button"
                        >
                            {move || tr!("wishlist-medley-disband")}
                        </button>
                    </Show>
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
                                data-testid="medley-confirm-delete-button"
                            >
                                {move || tr!("wishlist-medley-disband")}
                            </button>
                            <button
                                class="btn btn-ghost btn-sm"
                                type="button"
                                on:click=move |_| confirming.set(false)
                                disabled=move || pending.get()
                                data-testid="medley-cancel-delete-button"
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

fn move_song(ids: &mut [uuid::Uuid], song_id: uuid::Uuid, delta: isize) {
    let Some(position) = ids.iter().position(|id| *id == song_id) else {
        return;
    };
    let next = position as isize + delta;
    if (0..ids.len() as isize).contains(&next) {
        ids.swap(position, next as usize);
    }
}
