# Plan page (rehearsal preparation)

**Status:** Not started (stub route only)

## Purpose

Plan which songs should be prepared for the **next exercise / rehearsal meeting**.

## Planned behaviour

- Create or edit the upcoming rehearsal plan (date, optional notes)
- Ordered list of songs / medleys to prepare
- Add / remove / reorder songs
- Visible to all members; edits by any member or admin-only (decide when implementing)

## Suggested data

- `rehearsal_plans` — date, title/notes, created_by
- `rehearsal_plan_songs` — plan_id, song_id, position

## Related

- Wishlist can feed candidates into the plan later
- Program page is “now playing”; plan is “prepare next”
