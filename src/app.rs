use crate::components::layout::AppShell;
use crate::components::theme::{provide_theme, ThemeRoot};
use crate::pages::{
    admin::AdminPage, finance::FinancePage, login::LoginPage, plan::PlanPage, program::ProgramPage,
    wishlist::WishlistPage,
};
use leptos::prelude::*;
use leptos_fluent::leptos_fluent;
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
        children: children(),
        locales: "./locales",
        default_language: "en",
        sync_html_tag_lang: true,
        initial_language_from_local_storage: true,
        set_language_to_local_storage: true,
        initial_language_from_navigator: true,
    }
}
