# Admin panel

**Status:** Member management implemented

## Purpose

Manage **band members** (who can log in by name).

## Implemented behaviour

- List members (name, role, active, last login)
- Create member by **name** (unique)
- Rename members without changing their IDs or session history
- Set role: `admin` | `member` | `participant`
- Deactivate / reactivate members (soft delete via `active`)
- Protect member-management server functions: only active `MemberRole::Admin` accounts
- Prevent demoting or deactivating the last active admin

## Existing foundation

- Table `members` already exists (see `migrations/`)
- Types in [`../../models/member.rs`](../../models/member.rs)
- Seeded admin: name `Admin`

## Optional later

- Shared band PIN on top of name login
