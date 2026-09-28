use leptos::prelude::*;
use leptos::{
    callback::{Callable, Callback},
    component,
    reactive::{
        signal::RwSignal,
        traits::{Get, Set},
    },
    server,
    server_fn::ServerFnError,
    view, IntoView,
};
use leptos_fluent::tr;

use crate::models::{Member, MemberRole};

#[server(AdminCreateMember, "/api")]
async fn admin_create_member(name: String, role: MemberRole) -> Result<Member, ServerFnError> {
    crate::auth::session::require_admin().await?;
    let pool = expect_context::<crate::state::AppState>().pool;
    Member::create(&pool, &name, role)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))
}

#[component]
pub fn CreateMemberForm(on_created: Callback<Member>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let role = RwSignal::new(MemberRole::Member);
    let pending = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let on_create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let name_value = name.get();
        let role_value = role.get();

        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            match admin_create_member(name_value, role_value).await {
                Ok(member) => {
                    name.set(String::new());
                    on_created.run(member);
                }
                Err(message) => error.set(Some(message.to_string())),
            }
            pending.set(false);
        });
    };

    view! {
        <section class="admin-card">
            <h2>{move || tr!("admin-add-title")}</h2>
            <form class="admin-create-form" on:submit=on_create>
                <label class="field">
                    <span class="field-label">{move || tr!("admin-name-label")}</span>
                    <input
                        class="input"
                        data-testid="admin-member-name"
                        type="text"
                        maxlength="80"
                        required
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                </label>
                <label class="field">
                    <span class="field-label">{move || tr!("admin-role-label")}</span>
                    <select
                        class="select"
                        data-testid="admin-member-role"
                        prop:value=move || role.get().to_string()
                        on:change=move |event| {
                            match event_target_value(&event).parse::<MemberRole>() {
                                Ok(value) => role.set(value),
                                Err(_) => error.set(Some(tr!("admin-error-role"))),
                            }
                        }
                    >
                        <option value=MemberRole::Admin.to_string()>{move || tr!("admin-role-admin")}</option>
                        <option value=MemberRole::Member.to_string()>{move || tr!("admin-role-member")}</option>
                        <option value=MemberRole::Participant.to_string()>{move || tr!("admin-role-participant")}</option>
                    </select>
                </label>
                <button
                    class="btn btn-primary"
                    data-testid="admin-create-member"
                    type="submit"
                    disabled=move || pending.get()
                >
                    {move || tr!("admin-add-button")}
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
