//! Login page — name-based member picker.

use crate::auth::{list_members, login_as_member};
use crate::models::Member;
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LoginPage() -> impl IntoView {
    let members = Resource::new(|| (), |_| async move { list_members().await });
    let selected = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let pending = RwSignal::new(false);
    let navigate = use_navigate();
    let auth_revision = expect_context::<RwSignal<u64>>();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = selected.get();
        if name.trim().is_empty() {
            error.set(Some("empty".into()));
            return;
        }
        pending.set(true);
        error.set(None);
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match login_as_member(name).await {
                Ok(_) => {
                    pending.set(false);
                    auth_revision.update(|revision| *revision += 1);
                    navigate("/", Default::default());
                }
                Err(e) => {
                    pending.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    view! {
        <section class="login-page">
            <div class="login-card">
                <p class="eyebrow">{move || tr!("login-eyebrow")}</p>
                <h1 class="login-title" data-testid="login-title">{move || tr!("login-title")}</h1>
                <p class="login-lead">{move || tr!("login-lead")}</p>

                <form class="login-form" data-testid="login-form" on:submit=on_submit>
                    <label class="field">
                        <span class="field-label">{move || tr!("login-name-label")}</span>
                        <Suspense fallback=move || {
                            view! { <p class="muted">{move || tr!("login-loading")}</p> }
                        }>
                            {move || match members.get() {
                                None => view! { <p class="muted">{move || tr!("login-loading")}</p> }.into_any(),
                                Some(Err(_)) => view! {
                                    <p class="form-error" data-testid="login-error">{move || tr!("login-load-error")}</p>
                                }
                                .into_any(),
                                Some(Ok(list)) if list.is_empty() => view! {
                                    <p class="form-error" data-testid="login-error">{move || tr!("login-empty")}</p>
                                }
                                .into_any(),
                                Some(Ok(list)) => view! {
                                    <MemberSelect members=list selected=selected />
                                }
                                .into_any(),
                            }}
                        </Suspense>
                    </label>

                    <Show when=move || error.get().is_some()>
                        <p class="form-error" data-testid="login-error">
                            {move || match error.get().as_deref() {
                                Some("empty") => tr!("login-error-empty"),
                                Some(_) => tr!("login-error-generic"),
                                None => "".into(),
                            }}
                        </p>
                    </Show>

                    <button
                        type="submit"
                        class="btn btn-primary btn-block"
                        data-testid="login-submit"
                        disabled=move || pending.get()
                    >
                        {move || {
                            if pending.get() {
                                tr!("login-submitting")
                            } else {
                                tr!("login-submit")
                            }
                        }}
                    </button>
                </form>
            </div>
        </section>
    }
}

#[component]
fn MemberSelect(members: Vec<Member>, selected: RwSignal<String>) -> impl IntoView {
    let options = members
        .into_iter()
        .map(|m| {
            let name = m.name.clone();
            let value = m.name.clone();
            view! {
                <option value=value>{name}</option>
            }
        })
        .collect_view();

    view! {
        <select
            class="select"
            data-testid="login-member-select"
            prop:value=move || selected.get()
            on:change=move |ev| {
                selected.set(event_target_value(&ev));
            }
        >
            <option value="">
                {move || tr!("login-placeholder")}
            </option>
            {options}
        </select>
    }
}
