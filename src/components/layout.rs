use crate::auth::{current_member, logout};
use crate::components::language::LanguageSwitcher;
use crate::components::theme::ThemeToggle;
use crate::models::MemberSummary;
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::components::A;

#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    let auth_revision = RwSignal::new(0_u64);
    provide_context(auth_revision);

    let member = Resource::new(
        move || auth_revision.get(),
        |_| async move { current_member().await.ok().flatten() },
    );

    view! {
        <div class="app-shell">
            <header class="topbar" data-testid="topbar">
                <A href="/" attr:class="brand">
                    <span class="brand-mark">"♪"</span>
                    <span class="brand-name" data-testid="brand-name">"Bandman"</span>
                </A>
                <Suspense fallback=|| ()>
                    {move || match member.get() {
                        Some(Some(m)) => view! {
                            <nav class="topbar-nav" aria-label="Main">
                                <A href="/wishlist">{move || tr!("nav-wishlist")}</A>
                                <A href="/plan">{move || tr!("nav-plan")}</A>
                                {if m.role.is_participant() {
                                    view! { <A href="/finance">{move || tr!("nav-finance")}</A> }.into_any()
                                } else {
                                    ().into_any()
                                }}
                                {if m.role.is_admin() {
                                    view! { <A href="/admin">{move || tr!("nav-admin")}</A> }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            </nav>
                        }
                        .into_any(),
                        _ => ().into_any(),
                    }}
                </Suspense>
                <div class="topbar-actions">
                    <LanguageSwitcher />
                    <ThemeToggle />
                    <Suspense fallback=|| ()>
                        {move || match member.get() {
                            Some(Some(m)) => view! { <LoggedInControls member=m /> }.into_any(),
                            _ => view! {
                                <A href="/login" attr:class="btn btn-primary btn-sm">
                                    {move || tr!("nav-login")}
                                </A>
                            }
                            .into_any(),
                        }}
                    </Suspense>
                </div>
            </header>
            <main class="app-main">
                {children()}
            </main>
        </div>
    }
}

#[component]
fn LoggedInControls(member: MemberSummary) -> impl IntoView {
    let name = member.name.clone();
    let auth_revision = expect_context::<RwSignal<u64>>();
    let logout_action = Action::new(move |_| {
        let auth_revision = auth_revision;
        async move {
            if logout().await.is_ok() {
                auth_revision.update(|revision| *revision += 1);
            }
        }
    });

    Effect::new(move |_| {
        if logout_action.version().get() > 0 {
            leptos_router::hooks::use_navigate()("/login", Default::default());
        }
    });

    view! {
        <span class="session-name" data-testid="session-name">{name}</span>
        <button
            type="button"
            class="btn btn-ghost btn-sm"
            data-testid="logout-button"
            on:click=move |_| {
                logout_action.dispatch(());
            }
        >
            {move || tr!("nav-logout")}
        </button>
    }
}
