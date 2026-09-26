# Wishlist page

**Status:** Not started (stub route only)

## Purpose

Every band member can enter **song wishes** and **vote** on them.

## Planned behaviour

- Add a wish (link to an existing song, or free-text title that can become a song later)
- Vote once per member per wish (toggle or upvote)
- Sort / filter by vote count and status (open / accepted / rejected)
- Support **medleys**: a wish may reference a medley song (combination of songs)
- Only active members; show who proposed each item

## Suggested data

- `wishlist_items` — song_id or free text, proposed_by, status
- `wishlist_votes` — unique `(item_id, member_id)`

## Out of scope for later

- Auto-promoting a wish into the rehearsal plan (nice-to-have)
