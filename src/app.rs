use crate::components::layout::AppShell;
use crate::components::theme::{provide_theme, ThemeRoot};
use crate::pages::{
    admin::AdminPage, finance::FinancePage, login::LoginPage, plan::PlanPage, program::ProgramPage,
    wishlist::WishlistPage,
};
use leptos::prelude::*;
use leptos_fluent::{leptos_fluent, I18n};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
                <link
                    href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600&family=Sora:wght@500;600;700&display=swap"
                    rel="stylesheet"
                />
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_theme();

    view! {
        <Stylesheet id="leptos" href="/pkg/bandman.css"/>
        <Title text="Bandman"/>
        <ThemeRoot>
            <I18nProvider>
                <Router>
                    <AppShell>
                        <Routes fallback=|| {
                            view! {
                                <section class="page-stub">
                                    <h1>"404"</h1>
                                </section>
                            }
                        }>
                            <Route path=StaticSegment("") view=ProgramPage/>
                            <Route path=StaticSegment("login") view=LoginPage/>
                            <Route path=StaticSegment("wishlist") view=WishlistPage/>
                            <Route path=StaticSegment("plan") view=PlanPage/>
                            <Route path=StaticSegment("finance") view=FinancePage/>
                            <Route path=StaticSegment("admin") view=AdminPage/>
                        </Routes>
                    </AppShell>
                </Router>
            </I18nProvider>
        </ThemeRoot>
    }
}

#[component]
fn I18nProvider(children: Children) -> impl IntoView {
    leptos_fluent! {
        children: view! {
            <LanguagePreferenceSync/>
            {children()}
        },
        locales: "./locales",
        default_language: "en",
        sync_html_tag_lang: true,
        initial_language_from_cookie: true,
        initial_language_from_accept_language_header: true,
        cookie_name: "lf-lang",
        cookie_attrs: "Path=/; Max-Age=31536000; SameSite=Lax",
        local_storage_key: "lang",
    }
}

#[component]
fn LanguagePreferenceSync() -> impl IntoView {
    let i18n = expect_context::<I18n>();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if leptos_fluent::cookie::get("lf-lang").is_some() {
            return;
        }
        let Some(language_id) = leptos_fluent::local_storage::get("lang") else {
            return;
        };
        if let Some(language) = i18n
            .languages
            .iter()
            .find(|language| language.id == language_id)
        {
            i18n.language.set(*language);
            persist_language(language.id);
        }
    });

    let _ = i18n;
    ().into_view()
}

pub(crate) fn persist_language(language_id: &str) {
    #[cfg(feature = "hydrate")]
    {
        leptos_fluent::local_storage::set("lang", language_id);
        leptos_fluent::cookie::set(
            "lf-lang",
            language_id,
            "Path=/; Max-Age=31536000; SameSite=Lax",
        );
    }
    let _ = language_id;
}
