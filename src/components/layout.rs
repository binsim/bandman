use crate::auth::{current_member, logout};
use crate::components::language::LanguageSwitcher;
use crate::components::theme::ThemeToggle;
use crate::models::{Member, MemberRole};
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::components::A;

#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    let (nav_open, set_nav_open) = signal(false);

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
                <div class="brand" style="border: none" on:click=move |_| set_nav_open.update(|open| *open = !*open)>
                    <span class="brand-mark">"♪"</span>
                    <span class="brand-name" data-testid="brand-name">"Bandman"</span>
                </div>
                <Suspense fallback=|| ()>
                    {move || match member.get().flatten() {
                        Some(m) => view! {
                            <div class="topbar-nav-container" data-testid="topbar-navigation-container">
                                <nav class="topbar-nav" class:mobile-open=move || nav_open.get() aria-label="Main">
                                    <ul>
                                        <li><A href="/">{move || tr!("nav-program")}</A></li>
                                        <li><A href="/wishlist">{move || tr!("nav-wishlist")}</A></li>
                                        <li><A href="/plan">{move || tr!("nav-plan")}</A></li>
                                        {if m.role == MemberRole::Participant || m.role == MemberRole::Admin {
                                            view! { <li><A href="/finance">{move || tr!("nav-finance")}</A></li> }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                        {if m.role == MemberRole::Admin {
                                            view! { <li><A href="/admin">{move || tr!("nav-admin")}</A></li> }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                    </ul>
                                </nav>
                            </div>
                            <UserIcon member=m/>
                        }.into_any(),
                        None => view! {
                            <div class="user-icon-container">
                                <A href="/login" attr:class="btn btn-primary btn-sm" attr:data-testid="login-btn">
                                    {move || tr!("nav-login")}
                                </A>
                            </div>
                        }.into_any(),
                    }}
                </Suspense>
            </header>
            <main class="app-main">
                {children()}
            </main>
        </div>
    }
}
#[component]
fn UserIcon(member: Member) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);

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
        <div class="user-icon-container">
        <button class="user-icon btn btn-ghost" data-testid="user-icon" on:click=move |_| set_menu_open.update(|open| *open = !*open)>{get_user_icon_content(&member.name)}
        </button>
        <Show when=move || menu_open.get()>
            <div class="user-card">
            // <section> Logged in as<br/> {member_name}</section>
            <hr/>
            <section>
                <LanguageSwitcher />
                <ThemeToggle />
            </section>
            <hr/>
            <section>
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
            </section>
            </div>
            </Show>
        </div>
    }.into_any()
}

fn get_user_icon_content(member_name: &str) -> String {
    let mut split = member_name.split_whitespace();
    let mut result_string;
    if let Some(first_word) = split.next()
        && let Some(first_char) = first_word.chars().next()
    {
        result_string = String::from(first_char);
    } else {
        return "".to_string();
    }

    if let Some(second) = split.next()
        && let Some(first_char) = second.chars().next()
    {
        result_string.push(first_char);
    }
    result_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_without_spaces_has_one_digit() {
        assert_eq!(get_user_icon_content("Admin"), "A");
    }

    #[test]
    fn user_with_space_has_two_digits() {
        assert_eq!(get_user_icon_content("Admin Admin"), "AA");
    }
}
