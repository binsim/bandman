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
- Let members give separate feedback on the medley as a whole, without changing
  the feedback status of its individual songs.
- Select multiple ungrouped songs to create a named, ordered medley. Its creator
  and admins can rename it, add or remove songs, or dissolve it without
  deleting the songs. They can also move songs up or down to set the medley
  order; medley members are never alphabetically sorted.
- Give medley-specific feedback when a different ordering would work better or
  when a song does not fit in the medley.
- Allow the proposer to delete their wish and admins to delete any wish.
- Allow the proposer and admins to edit the song title, artist, and link.
- Remember each member's choice of showing all wishes or only wishes needing
  their feedback.
- Collapse and expand medley groups while keeping their ordered songs intact.
- Require an active member session to view or add wishes.

## Remaining

- Edit wish intent (add as a song, add to a medley, try it, or mark as
  unplayable) and add multiple targets (for example party, slow dance, or
  discofox) after creating a wish.
- Sort or filter by feedback and review state.
- Connect targets and wish intents to the rehearsal-plan workflow.
