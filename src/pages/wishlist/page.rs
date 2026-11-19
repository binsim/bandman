use super::{
    api::{
        wishlist_create_medley, wishlist_disband_medley, wishlist_list, wishlist_move_medley_item,
        wishlist_remove_medley_item,
    },
    grouping::sort_wishlist_items,
    listing::WishlistListing,
    modify_song::CreateSong,
};
use crate::models::{Member, WishlistData, WishlistItem};
use leptos::prelude::*;
use leptos_fluent::tr;

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
                        Some(Ok(data)) => view! {
                            <WishlistLoaded
                                initial_data=data
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
fn WishlistLoaded(initial_data: WishlistData, current_member: Member) -> AnyView {
    let items = RwSignal::new(initial_data.items);
    provide_context(RwSignal::new(initial_data.metadata));
    let split_by_category = RwSignal::new(false);
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
                                item.medley_category = None;
                                item.medley_labels.clear();
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
                                item.medley_category = None;
                                item.medley_labels.clear();
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
                                    item.medley_category = None;
                                    item.medley_labels.clear();
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
                            item.medley_category = None;
                            item.medley_labels.clear();
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
            split_by_category
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
