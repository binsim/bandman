use crate::auth::{current_member, logout};
use crate::components::language::LanguageSwitcher;
use crate::components::theme::ThemeToggle;
use crate::models::{Member, MemberRole};
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
    provide_context(member);

    view! {
        <div class="app-shell">
            <header class="topbar" data-testid="topbar">
                <A href="/" attr:class="brand">
                    <span class="brand-mark">"♪"</span>
                    <span class="brand-name" data-testid="brand-name">"Bandman"</span>
                </A>
                <Suspense fallback=|| ()>
                    {move || match member.get() {
                        Some(member) => {
                            let role = member.as_ref().map(|m| m.role);
                            view! { <AppNavigation role /> }.into_any()
                        }
                        None => ().into_any(),
                    }}
                </Suspense>
                <div class="topbar-actions" data-testid="topbar-actions">
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
fn LoggedInControls(member: Member) -> impl IntoView {
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
        <div class="session-controls">
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
        </div>
    }
}

#[component]
fn AppNavigation(role: Option<MemberRole>) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);

    view! {
        {role.map(|role| view! {
            <nav class="topbar-nav" aria-label="Main">
                <A href="/">{move || tr!("nav-program")}</A>
                <A href="/wishlist">{move || tr!("nav-wishlist")}</A>
                <A href="/plan">{move || tr!("nav-plan")}</A>
                {if role == MemberRole::Participant || role == MemberRole::Admin {
                    view! { <A href="/finance">{move || tr!("nav-finance")}</A> }.into_any()
                } else {
                    ().into_any()
                }}
                {if role == MemberRole::Admin {
                    view! { <A href="/admin">{move || tr!("nav-admin")}</A> }.into_any()
                } else {
                    ().into_any()
                }}
            </nav>
        })}
        <div class=move || if menu_open.get() { "mobile-nav is-open" } else { "mobile-nav" } data-testid="mobile-navigation">
            <button
                type="button"
                class="brand mobile-nav-trigger"
                data-testid="mobile-navigation-trigger"
                aria-label="Bandman navigation"
                aria-expanded=move || menu_open.get().to_string()
                on:click=move |_| set_menu_open.update(|open| *open = !*open)
            >
                <span class="brand-mark">"♪"</span>
                <span class="brand-name" data-testid="mobile-brand-name">"Bandman"</span>
            </button>
            <Show when=move || menu_open.get()>
                <button
                    type="button"
                    class="mobile-nav-backdrop"
                    aria-label="Close navigation menu"
                    data-testid="mobile-navigation-backdrop"
                    on:click=move |_| set_menu_open.set(false)
                ></button>
            </Show>
            <nav class="mobile-nav-menu" aria-label="Main" data-testid="mobile-navigation-menu">
                    {if let Some(role) = role {
                        view! {
                            <A href="/" on:click=move |_| set_menu_open.set(false)>
                                {move || tr!("nav-program")}
                            </A>
                            <A href="/wishlist" on:click=move |_| set_menu_open.set(false)>
                                {move || tr!("nav-wishlist")}
                            </A>
                            <A href="/plan" on:click=move |_| set_menu_open.set(false)>
                                {move || tr!("nav-plan")}
                            </A>
                            {if role == MemberRole::Participant || role == MemberRole::Admin {
                                view! {
                                    <A href="/finance" on:click=move |_| set_menu_open.set(false)>
                                        {move || tr!("nav-finance")}
                                    </A>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                            {if role == MemberRole::Admin {
                                view! {
                                    <A href="/admin" on:click=move |_| set_menu_open.set(false)>
                                        {move || tr!("nav-admin")}
                                    </A>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                        }.into_any()
                    } else {
                        view! {
                            <A href="/login" on:click=move |_| set_menu_open.set(false)>
                                {move || tr!("nav-login")}
                            </A>
                        }.into_any()
                    }}
            </nav>
        </div>
    }
}
