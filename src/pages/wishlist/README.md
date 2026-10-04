# Wishlist page

**Status:** Initial implementation complete

## Purpose

Signed-in band members can suggest songs and give the band useful context for
considering and later planning them.

## Implemented

- Persist a song title, optional artist and HTTP(S) link.
- New wishes currently default to the regular song intent with no targets.
- Show wishes alphabetically with title, artist, and resolved link-title badges.
- Let each signed-in member set, update, or remove their own feedback for every wish.
- Show the saved feedback and its author on the wish.
- Allow the proposer to delete their wish and admins to delete any wish.
- Allow the proposer and admins to edit the song title, artist, and link.
- Require an active member session to view or add wishes.

## Remaining

- Edit wish intent (add as a song, add to a medley, try it, or mark as
  unplayable) and add multiple targets (for example party, slow dance, or
  discofox) after creating a wish.
- Sort or filter by feedback and review state.
- Connect targets and wish intents to the rehearsal-plan workflow.
