use crate::auth::{
    admin_create_member, admin_list_members, admin_set_member_active, admin_set_member_name,
    admin_set_member_role,
};
use crate::models::{Member, MemberRole};
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn AdminPage() -> impl IntoView {
    let revision = RwSignal::new(0_u64);
    let auth_revision = expect_context::<RwSignal<u64>>();
    let members = Resource::new(
        move || revision.get(),
        |_| async move { admin_list_members().await },
    );
    let feedback = RwSignal::new(Option::<(bool, String)>::None);
    let on_member_created = Callback::new(move |_| {
        revision.update(|value| *value += 1);
    });
    let on_members_changed = Callback::new(move |_| {
        revision.update(|value| *value += 1);
        auth_revision.update(|value| *value += 1);
    });

    view! {
        <section class="admin-page">
            <AdminHeader />

            <Suspense fallback=move || {
                view! { <p class="muted">{move || tr!("admin-loading")}</p> }
            }>
                {move || match members.get() {
                    None => view! { <p class="muted">{move || tr!("admin-loading")}</p> }.into_any(),
                    Some(Err(error)) => view! {
                        <section class="admin-card" data-testid="admin-access-denied">
                            <h2>{move || tr!("admin-access-denied-title")}</h2>
                            <p class="form-error">{error.to_string()}</p>
                        </section>
                    }
                    .into_any(),
                    Some(Ok(member_list)) => view! {
                        <AdminContent
                            members=member_list
                            on_member_created
                            on_members_changed
                            feedback
                        />
                    }
                    .into_any(),
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn AdminHeader() -> impl IntoView {
    view! {
        <header class="admin-header">
            <p class="eyebrow">{move || tr!("admin-eyebrow")}</p>
            <h1>{move || tr!("admin-title")}</h1>
            <p class="lead">{move || tr!("admin-lead")}</p>
        </header>
    }
}

#[component]
fn AdminContent(
    members: Vec<Member>,
    on_member_created: Callback<()>,
    on_members_changed: Callback<()>,
    feedback: RwSignal<Option<(bool, String)>>,
) -> impl IntoView {
    view! {
        <div class="admin-content">
            {view! { <CreateMemberForm on_created=on_member_created feedback /> }.into_any()}
            {view! { <MembersTable members on_members_changed feedback /> }.into_any()}
        </div>
    }
}

#[component]
fn CreateMemberForm(
    on_created: Callback<()>,
    feedback: RwSignal<Option<(bool, String)>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let role = RwSignal::new(MemberRole::Member.as_str().to_string());
    let pending = RwSignal::new(false);
    let on_create = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let name_value = name.get();
        let Some(role_value) = MemberRole::parse(&role.get()) else {
            feedback.set(Some((false, tr!("admin-error-role"))));
            return;
        };

        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_create_member(name_value, role_value).await {
                Ok(_) => {
                    name.set(String::new());
                    feedback.set(Some((true, tr!("admin-member-created"))));
                    on_created.run(());
                }
                Err(error) => feedback.set(Some((false, error.to_string()))),
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
                        prop:value=move || role.get()
                        on:change=move |event| role.set(event_target_value(&event))
                    >
                        <option value="admin">{move || tr!("admin-role-admin")}</option>
                        <option value="member">{move || tr!("admin-role-member")}</option>
                        <option value="participant">{move || tr!("admin-role-participant")}</option>
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

            <Show when=move || feedback.get().is_some()>
                <p
                    class=move || if feedback.get().is_some_and(|(success, _)| success) {
                        "admin-feedback"
                    } else {
                        "form-error"
                    }
                    role="status"
                    data-testid="admin-feedback"
                >
                    {move || feedback.get().map(|(_, text)| text).unwrap_or_default()}
                </p>
            </Show>
        </section>
    }
}

#[component]
fn MembersTable(
    members: Vec<Member>,
    on_members_changed: Callback<()>,
    feedback: RwSignal<Option<(bool, String)>>,
) -> impl IntoView {
    let rows = members
        .into_iter()
        .map(|member| view! { <MemberRow member on_change=on_members_changed feedback /> })
        .collect_view();

    view! {
        <section class="admin-card">
            <h2>{move || tr!("admin-members-title")}</h2>
            <div class="admin-table-wrap">
                <table class="admin-table">
                    <thead>
                        <tr>
                            <th>{move || tr!("admin-column-name")}</th>
                            <th>{move || tr!("admin-column-role")}</th>
                            <th>{move || tr!("admin-column-status")}</th>
                            <th>{move || tr!("admin-column-last-login")}</th>
                            <th><span class="sr-only">{move || tr!("admin-column-actions")}</span></th>
                        </tr>
                    </thead>
                    <tbody>{rows}</tbody>
                </table>
            </div>
        </section>
    }
}

#[component]
fn MemberRow(
    member: Member,
    on_change: Callback<()>,
    feedback: RwSignal<Option<(bool, String)>>,
) -> impl IntoView {
    let role = RwSignal::new(member.role.as_str().to_string());
    let name_input = RwSignal::new(member.name.clone());
    let pending = RwSignal::new(false);
    let member_id = member.id;
    let original_name = member.name.clone();
    let original_role = member.role.as_str().to_string();
    let unchanged_role = original_role.clone();
    let last_login = member
        .last_login
        .map(|time| time.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| tr!("admin-never"));
    let name = member.name.clone();
    let role_label = format!("{}: {}", tr!("admin-role-label"), name);
    let active = member.active;

    let save_name = move |_| {
        let name_value = name_input.get();
        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_set_member_name(member_id, name_value).await {
                Ok(()) => {
                    feedback.set(Some((true, tr!("admin-name-updated"))));
                    on_change.run(());
                }
                Err(error) => feedback.set(Some((false, error.to_string()))),
            }
            pending.set(false);
        });
    };

    let save_role = move |_| {
        let Some(role_value) = MemberRole::parse(&role.get()) else {
            feedback.set(Some((false, tr!("admin-error-role"))));
            return;
        };
        pending.set(true);
        feedback.set(None);
        let original_role = original_role.clone();
        leptos::task::spawn_local(async move {
            match admin_set_member_role(member_id, role_value).await {
                Ok(()) => {
                    feedback.set(Some((true, tr!("admin-role-updated"))));
                    on_change.run(());
                }
                Err(error) => {
                    role.set(original_role);
                    feedback.set(Some((false, error.to_string())));
                }
            }
            pending.set(false);
        });
    };

    let toggle_active = move |_| {
        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_set_member_active(member_id, !active).await {
                Ok(()) => {
                    feedback.set(Some((true, tr!("admin-status-updated"))));
                    on_change.run(());
                }
                Err(error) => feedback.set(Some((false, error.to_string()))),
            }
            pending.set(false);
        });
    };

    view! {
        <tr data-testid="admin-member-row" data-member-name=name>
            <td data-label=move || tr!("admin-column-name")>
                <div class="admin-role-control">
                    <input
                        class="input"
                        data-testid="admin-member-name-input"
                        type="text"
                        maxlength="80"
                        aria-label=move || tr!("admin-name-label")
                        prop:value=move || name_input.get()
                        on:input=move |event| name_input.set(event_target_value(&event))
                        disabled=move || pending.get()
                    />
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        data-testid="admin-save-name"
                        on:click=save_name
                        disabled=move || {
                            pending.get() || name_input.get().trim() == original_name
                        }
                    >
                        {move || tr!("admin-rename")}
                    </button>
                </div>
            </td>
            <td data-label=move || tr!("admin-column-role")>
                <div class="admin-role-control">
                    <select
                        class="select admin-role-select"
                        aria-label=role_label
                        prop:value=move || role.get()
                        on:change=move |event| role.set(event_target_value(&event))
                        disabled=move || pending.get()
                    >
                        <option value="admin">{move || tr!("admin-role-admin")}</option>
                        <option value="member">{move || tr!("admin-role-member")}</option>
                        <option value="participant">{move || tr!("admin-role-participant")}</option>
                    </select>
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        data-testid="admin-save-role"
                        on:click=save_role
                        disabled=move || pending.get() || role.get() == unchanged_role
                    >
                        {move || tr!("admin-save")}
                    </button>
                </div>
            </td>
            <td data-label=move || tr!("admin-column-status")>
                <span class=if active { "status-badge is-active" } else { "status-badge" }>
                    {if active { tr!("admin-active") } else { tr!("admin-inactive") }}
                </span>
            </td>
            <td data-label=move || tr!("admin-column-last-login")>{last_login}</td>
            <td class="admin-actions">
                <button
                    class="btn btn-ghost btn-sm"
                    type="button"
                    data-testid="admin-toggle-active"
                    on:click=toggle_active
                    disabled=move || pending.get()
                >
                    {if active { tr!("admin-deactivate") } else { tr!("admin-reactivate") }}
                </button>
            </td>
        </tr>
    }
}
