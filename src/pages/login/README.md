# Login page

**Status:** Implemented (foundation)

## Purpose

Name-based login for band members. No email, passwords, or magic links.

## Planned / current behaviour

1. Show a list (select) of **active** members from the database.
2. Member picks their **name** and submits.
3. Server sets an HTTP-only signed session cookie (`bandman_session`).
4. Redirect to home (`/`).
5. Logout clears the cookie (top bar).

## Seeded user

After migrations: member **Admin** (role `admin`) is available for first login.

## Related files

- [`mod.rs`](mod.rs) — UI
- [`../../auth/mod.rs`](../../auth/mod.rs) — server functions
- [`../../models/member.rs`](../../models/member.rs) — types
