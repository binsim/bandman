use leptos::prelude::*;

use crate::components::layout::PopupModal;
use crate::models::{Member, WishlistCategory, WishlistFeedback, WishlistItem};
use leptos_fluent::tr;

pub fn category_label(category: WishlistCategory) -> String {
    match category {
        WishlistCategory::Song => tr!("wishlist-category-song"),
        WishlistCategory::Medley => tr!("wishlist-category-medley"),
        WishlistCategory::Try => tr!("wishlist-category-try"),
        WishlistCategory::Unplayable => tr!("wishlist-category-unplayable"),
        WishlistCategory::OtherOrdering => tr!("wishlist-category-other-ordering"),
        WishlistCategory::NotFitMedley => tr!("wishlist-category-not-fit-medley"),
    }
}

#[component]
pub fn WishlistFeedbackControl(
    item: RwSignal<WishlistItem>,
    current_member: Member,
    on_feedback_changed: Callback<(uuid::Uuid, Option<WishlistFeedback>)>,
) -> AnyView {
    let initial_item = item.get_untracked();
    let is_medley = initial_item.medley_id.is_some();
    let member_id = current_member.id;
    let current_member = StoredValue::new(current_member);
    let selected = RwSignal::new(
        initial_item
            .feedback
            .iter()
            .find(|feedback| feedback.member_id == member_id)
            .map(|feedback| feedback.category),
    );
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let feedback_open = RwSignal::new(false);
    let feedbacks = RwSignal::new(initial_item.feedback);
    let close_feedback = Callback::new(move |_: ()| {
        if !pending.get_untracked() {
            feedback_open.set(false);
        }
    });
    let save_feedback = Callback::new(move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let category = selected.get_untracked();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match super::api::wishlist_set_feedback(item.get_untracked().id, category).await {
                Ok(feedback) => {
                    item.update(|item| {
                        item.feedback
                            .retain(|existing| existing.member_id != member_id);
                        if let Some(feedback) = feedback.clone() {
                            item.feedback.push(feedback);
                            item.feedback.sort_by(|left, right| {
                                left.member_name
                                    .to_lowercase()
                                    .cmp(&right.member_name.to_lowercase())
                            });
                        }
                    });
                    feedbacks.set(item.get_untracked().feedback);
                    on_feedback_changed.run((item.get_untracked().id, feedback));
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
                data-testid="wishliste-feedback-edit"
            >
                {move || tr!("wishlist-feedback-summary", {"count" => item.get().feedback.len().to_string()})}
            </button>
            <Show when=move || feedback_open.get()>
                <PopupModal data_testid="wishlist-feedback-popup">
                    <button
                        class="wishlist-edit-backdrop"
                        type="button"
                        aria-label=move || tr!("wishlist-feedback-close")
                        on:click=move |_| close_feedback.run(())
                        disabled=move || pending.get()
                    ></button>
                    <div
                        class="wishlist-feedback-content popup-modal-panel"
                        role="dialog"
                        aria-modal="true"
                        aria-label=move || tr!("wishlist-feedback-label")
                        on:keydown=move |event: leptos::ev::KeyboardEvent| {
                            if event.key() == "Escape" {
                                close_feedback.run(());
                            }
                        }
                    >
                        <WishlistFeedbackBadges feedbacks />
                        <WishlistFeedbackForm
                            feedbacks
                            current_member=current_member.get_value()
                            is_medley
                            selected
                            pending
                            error
                            on_save=save_feedback
                            on_cancel=close_feedback
                        />
                    </div>
                </PopupModal>
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
    current_member: Member,
    is_medley: bool,
    selected: RwSignal<Option<WishlistCategory>>,
    pending: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    on_save: Callback<leptos::ev::SubmitEvent>,
    on_cancel: Callback<()>,
) -> AnyView {
    let has_own_feedback = Memo::new(move |_| {
        feedbacks
            .get()
            .iter()
            .any(|feedback| feedback.member_id == current_member.id)
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
                <div class="form-actions">
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
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        on:click=move |_| on_cancel.run(())
                        disabled=move || pending.get()
                    >
                        {move || tr!("admin-cancel")}
                    </button>
                </div>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
    .into_any()
}
