# Program page

**Status:** Not started (stub route only)

## Purpose

Show the songs currently being played in rehearsal/performance. Each member uploads **their own PDF** (chords, lyrics, etc.) for personal viewing. When someone changes the current song, **all other members switch too** (live sync).

## Planned behaviour

- Shared `program_state.current_song_id` (singleton or per rehearsal session)
- Song change broadcasts over **WebSocket** so every connected client updates
- Per-member PDF upload & viewer for the current song (`member_song_files`)
- Medleys: show medley title; optionally list ordered parts
- Mobile-friendly PDF viewing (scroll / pinch)

## Suggested data

- `program_state` — current_song_id, updated_by, updated_at
- `member_song_files` — song_id, member_id, file_path, mime, uploaded_at

## Technical notes

- Use Axum WebSocket (or `leptos-axum-socket`) for live sync
- Store PDFs on a Docker volume (`uploads/`)
