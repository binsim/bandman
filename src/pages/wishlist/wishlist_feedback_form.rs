use leptos::prelude::*;

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
    let selected = RwSignal::new(
        item.get()
            .feedback
            .iter()
            .find(|feedback| feedback.member_id == current_member.id)
            .map(|feedback| feedback.category),
    );
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let feedback_open = RwSignal::new(false);
    let feedbacks = RwSignal::new(item.get().feedback);
    let save_feedback = Callback::new(move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let category = selected.get();
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match super::wishlist_set_feedback(item.get().id, category).await {
                Ok(feedback) => {
                    item.update(|item| {
                        item.feedback
                            .retain(|existing| existing.member_id != current_member.id);
                        if let Some(feedback) = feedback.clone() {
                            item.feedback.push(feedback);
                            item.feedback.sort_by(|left, right| {
                                left.member_name
                                    .to_lowercase()
                                    .cmp(&right.member_name.to_lowercase())
                            });
                        }
                    });
                    feedbacks.set(item.get().feedback);
                    on_feedback_changed.run((item.get().id, feedback));
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
            >
                {move || tr!("wishlist-feedback-summary", {"count" => item.get().feedback.len().to_string()})}
            </button>
            <Show
                clone:current_member
                when=move || feedback_open.get()
            >
                <button
                    class="wishlist-feedback-backdrop"
                    type="button"
                    aria-label=move || tr!("wishlist-feedback-close")
                    on:click=move |_| feedback_open.set(false)
                >                </button>
                <div class="wishlist-feedback-content">
                    <WishlistFeedbackBadges feedbacks />
                    <WishlistFeedbackForm
                        feedbacks
                        current_member=current_member.clone()
                        is_medley=item.get().medley_id.is_some()
                        selected
                        pending
                        error
                        on_save=save_feedback
                    />
                </div>
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
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
    .into_any()
}
