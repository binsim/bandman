use leptos::prelude::*;
use leptos_fluent::tr;

use crate::{models::Member, pages::wishlist::create_song::CreateSong};

mod create_song;

#[component]
pub fn WishlistPage() -> impl IntoView {
    let _current_member = expect_context::<Resource<Option<Member>>>();

    let on_song_created = Callback::new(|_| {});
    view! {
        <section class="page-stub">
            <h1>{move || tr!("nav-wishlist")}</h1>

            <CreateSong on_created=on_song_created />
        </section>
    }
}
