use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn WishlistPage() -> impl IntoView {
    view! {
        <section class="page-stub">
            <h1>{move || tr!("nav-wishlist")}</h1>
            <p class="muted">{move || tr!("stub-coming-soon")}</p>
            <p class="hint">{move || tr!("stub-see-readme")}</p>
        </section>
    }
}
