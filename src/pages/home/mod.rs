use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <section class="home-page">
            <h1>{move || tr!("home-title")}</h1>
            <p class="lead">{move || tr!("home-lead")}</p>
            <div class="home-links">
                <A href="/login" attr:class="btn btn-primary">{move || tr!("nav-login")}</A>
                <A href="/wishlist" attr:class="btn btn-ghost">{move || tr!("nav-wishlist")}</A>
            </div>
        </section>
    }
}
