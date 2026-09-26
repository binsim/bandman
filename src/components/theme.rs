use leptos::prelude::*;
use leptos_meta::Html;

#[allow(dead_code)]
const STORAGE_KEY: &str = "bandman-theme";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "dark" => Self::Dark,
            _ => Self::Light,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ThemeCtx {
    pub theme: RwSignal<Theme>,
}

pub fn provide_theme() -> ThemeCtx {
    let initial = initial_theme();
    let theme = RwSignal::new(initial);
    let ctx = ThemeCtx { theme };
    provide_context(ctx);

    Effect::new(move |_| {
        let current = theme.get();
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(STORAGE_KEY, current.as_str());
                }
            }
        }
        let _ = current;
    });

    ctx
}

fn initial_theme() -> Theme {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(value)) = storage.get_item(STORAGE_KEY) {
                    return Theme::parse(&value);
                }
            }
            if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
                if mql.matches() {
                    return Theme::Dark;
                }
            }
        }
    }
    Theme::Light
}

/// Applies `data-theme` on the document root.
#[component]
pub fn ThemeRoot(children: Children) -> impl IntoView {
    let ThemeCtx { theme } = expect_context::<ThemeCtx>();

    view! {
        <Html attr:data-theme=move || theme.get().as_str() />
        {children()}
    }
}

#[component]
pub fn ThemeToggle() -> impl IntoView {
    use leptos_fluent::tr;

    let ThemeCtx { theme } = expect_context::<ThemeCtx>();

    view! {
        <button
            type="button"
            class="btn btn-ghost theme-toggle"
            data-testid="theme-toggle"
            aria-label=move || tr!("theme-toggle")
            on:click=move |_| theme.update(|t| *t = t.toggle())
        >
            {move || match theme.get() {
                Theme::Light => tr!("theme-dark"),
                Theme::Dark => tr!("theme-light"),
            }}
        </button>
    }
}
