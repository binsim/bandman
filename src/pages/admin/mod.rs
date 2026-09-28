mod create_member;
mod server;

use self::server::{
    admin_delete_member, admin_list_members, admin_set_member_active, admin_update_member_name,
    admin_update_member_role,
};
use crate::{
    models::{Member, MemberRole},
    pages::admin::create_member::CreateMemberForm,
};
use leptos::prelude::*;
use leptos_fluent::tr;
use uuid::Uuid;

type MemberFeedback = Option<(Uuid, String)>;

#[component]
pub fn AdminPage() -> impl IntoView {
    let auth_revision = expect_context::<RwSignal<u64>>();
    let current_member = expect_context::<Resource<Option<Member>>>();
    let members = Resource::new(|| (), |_| async move { admin_list_members().await });
    let on_member_updated = Callback::new(move |member: Member| {
        if current_member
            .get_untracked()
            .flatten()
            .is_some_and(|current| current.id == member.id)
        {
            auth_revision.update(|value| *value += 1);
        }
    });

    view! {
        <section class="admin-page">
            <header class="admin-header">
                <p class="eyebrow">{move || tr!("admin-eyebrow")}</p>
                <h1>{move || tr!("admin-title")}</h1>
                <p class="lead">{move || tr!("admin-lead")}</p>
            </header>

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
                            current_member_id=current_member
                                .get_untracked()
                                .flatten()
                                .map(|member| member.id)
                            on_member_updated
                        />
                    }
                    .into_any(),
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn AdminContent(
    members: Vec<Member>,
    current_member_id: Option<Uuid>,
    on_member_updated: Callback<Member>,
) -> impl IntoView {
    let members = RwSignal::new(members);
    let feedback = RwSignal::new(MemberFeedback::None);
    let on_member_created = Callback::new(move |member: Member| {
        members.update(|members| {
            members.push(member);
            sort_members(members);
        });
    });
    let on_member_updated_locally = Callback::new(move |member: Member| {
        members.update(|members| {
            if let Some(existing) = members.iter_mut().find(|existing| existing.id == member.id) {
                *existing = member.clone();
                sort_members(members);
            }
        });
        on_member_updated.run(member);
    });
    let on_member_deleted = Callback::new(move |member_id| {
        members.update(|members| members.retain(|member| member.id != member_id));
    });

    view! {
        <div class="admin-content">
            {view! { <CreateMemberForm on_created=on_member_created /> }.into_any()}
            {
                view! {
                    <MembersTable
                        members
                        current_member_id
                        on_member_updated=on_member_updated_locally
                        on_member_deleted
                        feedback
                    />
                }
                .into_any()
            }
        </div>
    }
}

fn sort_members(members: &mut [Member]) {
    members.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
}

#[component]
fn MembersTable(
    members: RwSignal<Vec<Member>>,
    current_member_id: Option<Uuid>,
    on_member_updated: Callback<Member>,
    on_member_deleted: Callback<Uuid>,
    feedback: RwSignal<MemberFeedback>,
) -> impl IntoView {
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
                    <tbody>
                        <For
                            each=move || members.get()
                            key=|member| member.id
                            children=move |member| {
                                let is_current_member = current_member_id == Some(member.id);
                                view! {
                                    <MemberRow
                                        member
                                        is_current_member
                                        on_change=on_member_updated
                                        on_deleted=on_member_deleted
                                        feedback
                                    />
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </section>
    }
}

#[component]
fn MemberRow(
    member: Member,
    is_current_member: bool,
    on_change: Callback<Member>,
    on_deleted: Callback<Uuid>,
    feedback: RwSignal<MemberFeedback>,
) -> impl IntoView {
    let role = RwSignal::new(member.role.as_str().to_string());
    let name_input = RwSignal::new(member.name.clone());
    let original_name = RwSignal::new(member.name.clone());
    let original_role = RwSignal::new(member.role.as_str().to_string());
    let pending = RwSignal::new(false);
    let confirming_delete = RwSignal::new(false);
    let member_id = member.id;
    let name_dirty = Memo::new(move |_| name_input.get().trim() != original_name.get());
    let role_dirty = Memo::new(move |_| role.get() != original_role.get());
    let dirty = Memo::new(move |_| name_dirty.get() || role_dirty.get());
    let last_login = member
        .last_login
        .map(|time| time.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| tr!("admin-never"));
    let role_label = move || format!("{}: {}", tr!("admin-role-label"), name_input.get());
    let active = RwSignal::new(member.active);

    let save_name = Callback::new(move |_| {
        let name_value = name_input.get();
        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_update_member_name(member_id, name_value).await {
                Ok(updated_member) => {
                    name_input.set(updated_member.name.clone());
                    original_name.set(updated_member.name.clone());
                    feedback.set(None);
                    on_change.run(updated_member);
                }
                Err(error) => feedback.set(Some((member_id, error.to_string()))),
            }
            pending.set(false);
        });
    });

    let save_role = Callback::new(move |_| {
        let Some(role_value) = MemberRole::parse(&role.get()) else {
            feedback.set(Some((member_id, tr!("admin-error-role"))));
            return;
        };
        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_update_member_role(member_id, role_value).await {
                Ok(updated_member) => {
                    role.set(updated_member.role.as_str().to_string());
                    original_role.set(updated_member.role.as_str().to_string());
                    feedback.set(None);
                    on_change.run(updated_member);
                }
                Err(error) => feedback.set(Some((member_id, error.to_string()))),
            }
            pending.set(false);
        });
    });

    let reset_details = Callback::new(move |_| {
        name_input.set(original_name.get_untracked());
        role.set(original_role.get_untracked());
        feedback.set(None);
    });

    let toggle_active = Callback::new(move |_| {
        pending.set(true);
        feedback.set(None);
        let next_active = !active.get_untracked();
        leptos::task::spawn_local(async move {
            match admin_set_member_active(member_id, next_active).await {
                Ok(updated_member) => {
                    active.set(updated_member.active);
                    feedback.set(None);
                    on_change.run(updated_member);
                }
                Err(error) => feedback.set(Some((member_id, error.to_string()))),
            }
            pending.set(false);
        });
    });

    let delete_member = Callback::new(move |_| {
        pending.set(true);
        feedback.set(None);
        leptos::task::spawn_local(async move {
            match admin_delete_member(member_id).await {
                Ok(()) => {
                    feedback.set(None);
                    on_deleted.run(member_id);
                }
                Err(error) => feedback.set(Some((member_id, error.to_string()))),
            }
            confirming_delete.set(false);
            pending.set(false);
        });
    });

    view! {
        <tr
            data-testid="admin-member-row"
            data-member-id=member_id.to_string()
            data-member-name=move || name_input.get()
        >
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
                </div>
            </td>
            <td data-label=move || tr!("admin-column-status")>
                <span class=move || if active.get() {
                    "status-badge is-active"
                } else {
                    "status-badge"
                }>
                    {move || if active.get() { tr!("admin-active") } else { tr!("admin-inactive") }}
                </span>
            </td>
            <td data-label=move || tr!("admin-column-last-login")>{last_login}</td>
            <td class="admin-actions">
                <MemberRowActions
                    active
                    member_name=name_input
                    is_current_member
                    pending
                    name_dirty
                    role_dirty
                    dirty
                    confirming_delete
                    on_save_name=save_name
                    on_save_role=save_role
                    on_reset=reset_details
                    on_toggle_active=toggle_active
                    on_delete=delete_member
                />
                <MemberRowError member_id feedback />
            </td>
        </tr>
    }
}

#[component]
fn MemberRowActions(
    active: RwSignal<bool>,
    member_name: RwSignal<String>,
    is_current_member: bool,
    pending: RwSignal<bool>,
    name_dirty: Memo<bool>,
    role_dirty: Memo<bool>,
    dirty: Memo<bool>,
    confirming_delete: RwSignal<bool>,
    on_save_name: Callback<()>,
    on_save_role: Callback<()>,
    on_reset: Callback<()>,
    on_toggle_active: Callback<()>,
    on_delete: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="admin-role-control">
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                data-testid="admin-save-member"
                on:click=move |_| on_save_name.run(())
                disabled=move || pending.get() || !name_dirty.get()
            >
                {move || tr!("admin-save-name")}
            </button>
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                data-testid="admin-save-member-role"
                on:click=move |_| on_save_role.run(())
                disabled=move || pending.get() || !role_dirty.get()
            >
                {move || tr!("admin-save-role")}
            </button>
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                data-testid="admin-reset-member"
                on:click=move |_| on_reset.run(())
                disabled=move || pending.get() || !dirty.get()
            >
                {move || tr!("admin-reset")}
            </button>
            <button
                class="btn btn-ghost btn-sm"
                type="button"
                data-testid="admin-toggle-active"
                on:click=move |_| on_toggle_active.run(())
                disabled=move || pending.get()
            >
                {move || if active.get() { tr!("admin-deactivate") } else { tr!("admin-reactivate") }}
            </button>
            {if is_current_member {
                ().into_any()
            } else {
                view! {
                    <button
                        class="btn btn-ghost btn-danger btn-sm"
                        type="button"
                        data-testid="admin-delete-member"
                        on:click=move |_| confirming_delete.set(true)
                        disabled=move || pending.get() || confirming_delete.get()
                    >
                        {move || tr!("admin-delete")}
                    </button>
                }
                .into_any()
            }}
        </div>
        {view! {
            <MemberDeleteDialog
                confirming_delete
                member_name
                pending
                on_delete
            />
        }
        .into_any()}
    }
}

#[component]
fn MemberDeleteDialog(
    confirming_delete: RwSignal<bool>,
    member_name: RwSignal<String>,
    pending: RwSignal<bool>,
    on_delete: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || confirming_delete.get()>
            <div class="admin-delete-backdrop">
                <section
                    class="admin-delete-dialog"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || tr!("admin-delete-title")
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if event.key() == "Escape" {
                            confirming_delete.set(false);
                        }
                    }
                >
                    <div class="admin-delete-dialog-content">
                        <h2>{move || tr!("admin-delete-title")}</h2>
                        <p>{move || tr!("admin-confirm-delete", {"name" => member_name.get()})}</p>
                        <div class="admin-delete-dialog-actions">
                            <button
                                class="btn btn-ghost btn-danger btn-sm"
                                type="button"
                                data-testid="admin-confirm-delete"
                                on:click=move |_| on_delete.run(())
                                disabled=move || pending.get()
                            >
                                {move || tr!("admin-delete")}
                            </button>
                            <button
                                class="btn btn-ghost btn-sm"
                                type="button"
                                data-testid="admin-cancel-delete"
                                on:click=move |_| confirming_delete.set(false)
                                disabled=move || pending.get()
                                autofocus
                            >
                                {move || tr!("admin-cancel")}
                            </button>
                        </div>
                    </div>
                </section>
            </div>
        </Show>
    }
}

#[component]
fn MemberRowError(member_id: Uuid, feedback: RwSignal<MemberFeedback>) -> impl IntoView {
    view! {
        <Show when=move || feedback.get().is_some_and(|(id, _)| id == member_id)>
            <p
                class="form-error admin-row-feedback"
                role="status"
                data-testid="admin-row-error"
            >
                {move || {
                    feedback
                        .get()
                        .filter(|(id, _)| *id == member_id)
                        .map(|(_, text)| text)
                        .unwrap_or_default()
                }}
            </p>
        </Show>
    }
}
