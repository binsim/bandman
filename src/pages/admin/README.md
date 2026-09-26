# Admin panel

**Status:** Not started (stub route only)

## Purpose

Manage **band members** (who can log in by name).

## Planned behaviour

- List members (name, role, active, last login)
- Create member by **name** (unique)
- Set role: `admin` | `member`
- Deactivate / reactivate members (soft delete via `active`)
- Protect route: only `MemberRole::Admin`
- Ensure at least one admin remains

## Existing foundation

- Table `members` already exists (see `migrations/`)
- Types in [`../../models/member.rs`](../../models/member.rs)
- Seeded admin: name `Admin`

## Optional later

- Shared band PIN on top of name login
- Rename members without breaking session history
