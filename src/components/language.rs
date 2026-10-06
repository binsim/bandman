use leptos::prelude::*;
use leptos_fluent::I18n;

#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let i18n = expect_context::<I18n>();

    let set_en = move |_| {
        if let Some(lang) = i18n.languages.iter().find(|l| l.id == "en") {
            i18n.language.set(*lang);
            crate::app::persist_language(lang.id);
        }
    };
    let set_de = move |_| {
        if let Some(lang) = i18n.languages.iter().find(|l| l.id == "de") {
            i18n.language.set(*lang);
            crate::app::persist_language(lang.id);
        }
    };

    let is_en = move || i18n.language.get().id == "en";
    let is_de = move || i18n.language.get().id == "de";

    view! {
        <div class="lang-switcher" role="group" aria-label="Language" data-testid="lang-switcher">
            <button
                type="button"
                data-testid="lang-en"
                class=move || {
                    if is_en() {
                        "btn btn-ghost lang-btn is-active"
                    } else {
                        "btn btn-ghost lang-btn"
                    }
                }
                on:click=set_en
            >
                "EN"
            </button>
            <button
                type="button"
                data-testid="lang-de"
                class=move || {
                    if is_de() {
                        "btn btn-ghost lang-btn is-active"
                    } else {
                        "btn btn-ghost lang-btn"
                    }
                }
                on:click=set_de
            >
                "DE"
            </button>
        </div>
    }
}
