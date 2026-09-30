use leptos::prelude::*;

#[component]
pub fn CreateSong(on_created: Callback<()>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let on_create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        pending.set(true);
        on_created.run(())
    };

    view! {
        <section>
            <h2>Create song wish</h2>
            <form class="admin-create-form" on:submit=on_create>
                <label class="field" for="add-song-name">
                    <span class="label">Song Name</span>
                    <input
                        class="input"
                        data-testid="add-song-name"
                        type="text"
                        maxlength="80"
                        required
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                </label>
                <button
                    class="btn btn-primary"
                    type="submit"
                    data-testid="create-song"
                    disabled=move || pending.get()
                >
                    "Create"
                </button>
            </form>
            <Show when=move || error.get().is_some()>
                <p class="form-error" role="alert" data-testid="admin-create-error">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </section>
    }
}
